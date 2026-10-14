//! P1-B B-10: a seeded random sequence of propose (agent ingest) / human
//! ingest / promote / demote / prune / refused requests; after it the layer
//! state held in memory equals the state rebuilt by replaying the log
//! (`Vault::open`), and every refused request left the log untouched.
//!
//! `MDK_FUZZ_SEED` (u64, default 20261014) and `MDK_FUZZ_STEPS` (default
//! 1000) select the sequence; `MDK_FUZZ_OUT` receives a JSON summary.

use mdkcore::{
    Actor, ActorKind, AuthorKind, Deny, HumanOnly, Layer, MatcherVersion, Provenance,
    TransitionKind, Vault,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// splitmix64 (no external RNG: the sequence is fixed by the seed alone)
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const WORDS: [&str; 40] = [
    "espresso",
    "grind",
    "water",
    "temperature",
    "pressure",
    "basket",
    "tamp",
    "crema",
    "milk",
    "steam",
    "latte",
    "roast",
    "bean",
    "origin",
    "acidity",
    "body",
    "sweet",
    "bitter",
    "dose",
    "yield",
    "ratio",
    "filter",
    "paper",
    "kettle",
    "pour",
    "bloom",
    "time",
    "scale",
    "burr",
    "blade",
    "storage",
    "fresh",
    "stale",
    "aroma",
    "cup",
    "tasting",
    "notes",
    "cherry",
    "farm",
    "wash",
];

fn paragraph(r: &mut Rng) -> String {
    let n = 8 + r.below(8);
    let mut w: Vec<&str> = (0..n).map(|_| WORDS[r.below(WORDS.len())]).collect();
    w[0] = "Note";
    format!("{}.", w.join(" "))
}

fn render(title: &str, paras: &[String]) -> String {
    let mut s = format!("# {title}\n");
    for p in paras {
        s.push('\n');
        s.push_str(p);
        s.push('\n');
    }
    s
}

fn edit(r: &mut Rng, paras: &mut Vec<String>) {
    match r.below(6) {
        0 if !paras.is_empty() => {
            // light rewrite of one paragraph
            let i = r.below(paras.len());
            let mut w: Vec<String> = paras[i]
                .trim_end_matches('.')
                .split(' ')
                .map(str::to_string)
                .collect();
            let j = 1 + r.below(w.len() - 1);
            w[j] = WORDS[r.below(WORDS.len())].to_string();
            paras[i] = format!("{}.", w.join(" "));
        }
        1 if paras.len() > 1 => {
            paras.remove(r.below(paras.len()));
        }
        2 if paras.len() > 1 => {
            // merge two adjacent paragraphs
            let i = r.below(paras.len() - 1);
            let b = paras.remove(i + 1);
            paras[i] = format!("{} {}", paras[i], b);
        }
        _ => {
            let at = r.below(paras.len() + 1);
            paras.insert(at, paragraph(r));
        }
    }
}

type LayerState = BTreeMap<String, (String, Layer, bool)>;

fn layer_state(v: &Vault) -> LayerState {
    let mut out = BTreeMap::new();
    for d in v.doc_paths() {
        let s = v.snapshot(&d).unwrap().unwrap();
        for b in v.snapshot_blocks(&s).unwrap() {
            let l = b.lineage_id.clone();
            assert_eq!(v.layer_of(&l).unwrap(), b.layer);
            out.insert(l.clone(), (d.clone(), b.layer, v.is_inactive(&l).unwrap()));
        }
    }
    out
}

fn env_u64(k: &str, d: u64) -> u64 {
    std::env::var(k)
        .ok()
        .map(|s| {
            s.parse()
                .unwrap_or_else(|_| panic!("{k}={s:?} is not a u64"))
        })
        .unwrap_or(d)
}

#[test]
fn b10_layer_state_equals_replay() {
    let seed = env_u64("MDK_FUZZ_SEED", 20261014);
    let steps = env_u64("MDK_FUZZ_STEPS", 1000);
    let root: PathBuf =
        std::env::temp_dir().join(format!("mdk-fuzz-{}-{seed}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let mut v = Vault::open(&root).unwrap();
    let mut r = Rng(seed);
    let mut docs: Vec<Vec<String>> = (0..3)
        .map(|_| (0..3).map(|_| paragraph(&mut r)).collect())
        .collect();
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    let human = Actor::new(ActorKind::Human, "reviewer").unwrap();
    let agent = Actor::new(ActorKind::Agent, "claude").unwrap();
    for step in 0..steps {
        let op = r.below(8);
        let log_before = fs::read(v.events_path()).unwrap_or_default();
        let live: Vec<String> = layer_state(&v).keys().cloned().collect();
        let pick = |r: &mut Rng| live[r.below(live.len())].clone();
        let name = match op {
            0..=2 => {
                let d = r.below(docs.len());
                edit(&mut r, &mut docs[d]);
                let kind = if op == 2 {
                    AuthorKind::Human
                } else {
                    AuthorKind::Agent
                };
                let p = Provenance {
                    author_kind: kind,
                    source: format!("fuzz-{step}"),
                };
                v.ingest(
                    &format!("d{d}.md"),
                    &render(&format!("Doc {d}"), &docs[d]),
                    p,
                    MatcherVersion::V3,
                )
                .unwrap();
                if op == 2 {
                    "ingest_human"
                } else {
                    "propose"
                }
            }
            _ if live.is_empty() => "skip",
            3 | 4 => {
                let l = pick(&mut r);
                let cur = v.layer_of(&l).unwrap();
                let req = v
                    .request(
                        TransitionKind::Promote,
                        &l,
                        Layer::Canonical,
                        human.clone(),
                        "fuzz",
                    )
                    .unwrap();
                match v.promote(&req, &HumanOnly) {
                    Ok(_) => {
                        assert_eq!(cur, Layer::Proposal);
                        assert_eq!(v.layer_of(&l).unwrap(), Layer::Canonical);
                        "promote_ok"
                    }
                    Err(_) => {
                        assert!(cur != Layer::Proposal || v.is_inactive(&l).unwrap());
                        assert_eq!(fs::read(v.events_path()).unwrap(), log_before);
                        "promote_refused_state"
                    }
                }
            }
            5 => {
                let l = pick(&mut r);
                let cur = v.layer_of(&l).unwrap();
                let to = if cur == Layer::Canonical {
                    Layer::Proposal
                } else {
                    Layer::Derived
                };
                let req = v
                    .request(TransitionKind::Demote, &l, to, human.clone(), "fuzz")
                    .unwrap();
                match v.demote(&req, &HumanOnly) {
                    Ok(_) => {
                        assert_eq!(v.layer_of(&l).unwrap(), to);
                        "demote_ok"
                    }
                    Err(_) => {
                        assert!(cur == Layer::Derived || v.is_inactive(&l).unwrap());
                        assert_eq!(fs::read(v.events_path()).unwrap(), log_before);
                        "demote_refused_state"
                    }
                }
            }
            6 => {
                let l = pick(&mut r);
                match v.prune_inactive(&l, human.clone(), "fuzz", &HumanOnly) {
                    Ok(_) => {
                        assert!(v.is_inactive(&l).unwrap());
                        "prune_ok"
                    }
                    Err(_) => {
                        assert_eq!(fs::read(v.events_path()).unwrap(), log_before);
                        "prune_refused_state"
                    }
                }
            }
            _ => {
                // refused by the hook: agent actor under HumanOnly, anyone under Deny
                let l = pick(&mut r);
                let req = v
                    .request(
                        TransitionKind::Promote,
                        &l,
                        Layer::Canonical,
                        agent.clone(),
                        "fuzz",
                    )
                    .unwrap();
                assert!(v.promote(&req, &HumanOnly).is_err());
                let mut req = req;
                req.actor = human.clone();
                assert!(v.promote(&req, &Deny).is_err());
                assert_eq!(fs::read(v.events_path()).unwrap(), log_before);
                "hook_refused"
            }
        };
        *counts.entry(name).or_default() += 1;
        if step % 100 == 99 {
            assert_eq!(
                layer_state(&Vault::open(&root).unwrap()),
                layer_state(&v),
                "step {step}"
            );
        }
    }
    let mem = layer_state(&v);
    let replayed = layer_state(&Vault::open(&root).unwrap());
    assert_eq!(mem, replayed);
    let by_layer = |want: Layer| mem.values().filter(|(_, l, _)| *l == want).count();
    let summary = json!({
        "seed": seed,
        "steps": steps,
        "ops": counts,
        "live_lineages": mem.len(),
        "canonical": by_layer(Layer::Canonical),
        "proposal": by_layer(Layer::Proposal),
        "derived": by_layer(Layer::Derived),
        "inactive": mem.values().filter(|(_, _, i)| *i).count(),
        "events_hash": v.events_hash().unwrap(),
        "replay_equal": true,
    });
    println!("{summary}");
    if let Ok(p) = std::env::var("MDK_FUZZ_OUT") {
        fs::write(p, format!("{summary}\n")).unwrap();
    }
    fs::remove_dir_all(&root).unwrap();
}
