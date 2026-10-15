//! P1-D policy extension (GOAL "Politika ve kimlik"): `[promotion] policy`
//! table form, `[server]` and `[index]` tables validated by the server; an
//! unknown field, a wrong type or a value out of bounds is an error (never
//! clamped); the P1-B forms are unchanged.

use mdkcore::{Policy, PolicySource, Promotion};
use mdkserver::config::{default_recipe, ServerConfig};

const BASE: &str =
    "[version_trigger]\nidle_ms = 1000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 24\n";

fn parse(text: &str) -> anyhow::Result<Policy> {
    Policy::parse(text, PolicySource::Builtin)
}

fn cfg(text: &str) -> anyhow::Result<ServerConfig> {
    ServerConfig::from_policy(&parse(text)?)
}

#[test]
fn promotion_string_and_table_forms() {
    let a = parse(&format!("promotion = \"human_only\"\n{BASE}")).unwrap();
    let b = parse(&format!("{BASE}\n[promotion]\npolicy = \"human_only\"\n")).unwrap();
    assert_eq!(a.promotion, Promotion::HumanOnly);
    assert_eq!(b.promotion, Promotion::HumanOnly);
    assert_eq!(a.to_json(), b.to_json());
    for (text, pat) in [
        (
            format!("{BASE}\n[promotion]\npolicy = \"implicit\"\n"),
            "unknown promotion policy",
        ),
        (
            format!("{BASE}\n[promotion]\nname = \"deny\"\n"),
            "unknown field `name`",
        ),
        (format!("{BASE}\n[promotion]\n"), "missing field `policy`"),
        (
            format!("{BASE}\n[promotion]\npolicy = 1\n"),
            "must be a string",
        ),
        (format!("promotion = 1\n{BASE}"), "invalid policy.toml"),
    ] {
        let e = format!("{:#}", parse(&text).unwrap_err());
        assert!(e.contains(pat), "{text}: {e}");
    }
}

#[test]
fn server_and_index_tables() {
    let p = format!("promotion = \"deny\"\n{BASE}");
    // absent tables: defaults; to_json of the kernel policy is unchanged
    let c = cfg(&p).unwrap();
    assert_eq!(c, ServerConfig::default());
    let full = format!(
        "{p}\n[server]\nhttp_port = 7357\ntoken_rotate_days = 7\nmax_propose_bytes = 2048\n\n[index]\ncompact_every_s = 60\n"
    );
    let c = cfg(&full).unwrap();
    assert_eq!(c.http_port, Some(7357));
    assert_eq!(c.token_rotate_days, 7);
    assert_eq!(c.max_propose_bytes, 2048);
    assert_eq!(c.compact_every_s, 60);
    assert_eq!(
        parse(&full).unwrap().to_json(),
        parse(&p).unwrap().to_json()
    );
    for (extra, pat) in [
        ("[server]\nhttp_port = 0\n", "outside the bounds"),
        ("[server]\nhttp_port = 70000\n", "outside the bounds"),
        ("[server]\ntoken_rotate_days = 0\n", "outside the bounds"),
        ("[server]\nmax_propose_bytes = 0\n", "outside the bounds"),
        (
            "[server]\nmax_propose_bytes = 67108865\n",
            "outside the bounds",
        ),
        ("[server]\nhttp_port = \"80\"\n", "must be an integer"),
        ("[server]\nhost = \"0.0.0.0\"\n", "unknown field `host`"),
        ("[index]\ncompact_every_s = 0\n", "outside the bounds"),
        ("[index]\ncompact_every_s = 86401\n", "outside the bounds"),
        ("[index]\nbm25_k1 = 1.5\n", "unknown field `bm25_k1`"),
    ] {
        let e = format!("{:#}", cfg(&format!("{p}\n{extra}")).unwrap_err());
        assert!(e.contains(pat), "{extra}: {e}");
    }
    // an unknown top-level table is still a kernel error
    let e = format!(
        "{:#}",
        parse(&format!("{p}\n[client]\nx = 1\n")).unwrap_err()
    );
    assert!(e.contains("unknown field"), "{e}");
}

#[test]
fn default_recipe_is_valid_and_human_only() {
    let p = parse(&default_recipe()).unwrap();
    assert_eq!(p.promotion, Promotion::HumanOnly);
    let b = Policy::builtin();
    assert_eq!(p.trigger, b.trigger);
    assert_eq!(p.retention, b.retention);
    assert_eq!(
        ServerConfig::from_policy(&p).unwrap(),
        ServerConfig::default()
    );
}
