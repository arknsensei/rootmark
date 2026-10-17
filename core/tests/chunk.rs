use rootmarkcore::chunk::{count_tokens, FIXED_OVERLAP, MAX_TOKENS};
use rootmarkcore::{chunk, hash32, MatcherVersion, Provenance, Strategy, Vault};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("rootmark-chunk-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn mini_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault")
}

fn chunks_of(text: &str, strategy: Strategy, name: &str) -> (Vault, Vec<rootmarkcore::Chunk>) {
    let mut v = Vault::open(tmp(name)).unwrap();
    let r = v
        .ingest("d.md", text, Provenance::default(), MatcherVersion::V2)
        .unwrap();
    let c = chunk(&r.snapshot, &v, strategy).unwrap();
    (v, c)
}

fn words(n: usize, tag: &str) -> String {
    (0..n)
        .map(|i| format!("{tag}{i}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn structural_never_crosses_a_heading() {
    let text = "# A\n\npara a1\n\npara a2\n\n## B\n\npara b1\n\n# C\n\npara c1\n";
    let (v, cs) = chunks_of(text, Strategy::Structural, "hb");
    let snap = v.snapshot("d.md").unwrap().unwrap();
    let blocks = v.snapshot_blocks(&snap).unwrap();
    assert_eq!(cs.len(), 3);
    for c in &cs {
        // every non-heading block of a chunk shares the chunk's heading path
        for vid in &c.version_ids {
            let b = blocks.iter().find(|b| &b.version_id == vid).unwrap();
            if b.block_type != rootmarkcore::BlockType::Heading {
                assert_eq!(b.heading_path, c.heading_path);
            }
        }
    }
    assert_eq!(cs[0].heading_path, vec!["A"]);
    assert_eq!(cs[1].heading_path, vec!["A", "B"]);
    assert!(cs[1].text.starts_with("A > B\n\n## B"));
}

#[test]
fn structural_respects_512_and_keeps_oversized_block_whole() {
    let big = words(700, "w");
    assert!(count_tokens(&big) > MAX_TOKENS as u32);
    let small: Vec<String> = (0..8).map(|i| words(90, &format!("s{i}x"))).collect();
    let text = format!("# H\n\n{}\n\n{big}\n\nafter\n", small.join("\n\n"));
    let (_, cs) = chunks_of(&text, Strategy::Structural, "512");
    let big_chunks: Vec<_> = cs.iter().filter(|c| c.text.contains("w699")).collect();
    assert_eq!(big_chunks.len(), 1, "oversized block is not split");
    assert!(big_chunks[0].text.contains("w0 "));
    assert_eq!(
        big_chunks[0].version_ids.len(),
        1,
        "oversized block is alone"
    );
    for c in cs.iter().filter(|c| !c.text.contains("w699")) {
        assert!(c.token_count as usize <= MAX_TOKENS, "{}", c.token_count);
    }
    for c in &cs {
        assert_eq!(c.token_count, count_tokens(&c.text));
    }
}

#[test]
fn chunk_id_is_hash_of_version_ids_and_unique_per_doc() {
    let text = "# X\n\nsame body\n\n# X\n\nsame body\n\n# Y\n\nother\n";
    let (_, cs) = chunks_of(text, Strategy::Structural, "ids");
    assert_eq!(cs.len(), 3);
    assert_eq!(cs[0].chunk_id, hash32(&cs[0].version_ids.join("\n")));
    assert_eq!(cs[0].version_ids, cs[1].version_ids);
    assert_eq!(
        cs[1].chunk_id,
        hash32(&format!("{}\n#1", cs[1].version_ids.join("\n")))
    );
    assert_eq!(cs[2].chunk_id, hash32(&cs[2].version_ids.join("\n")));
}

#[test]
fn fixed_windows_overlap_and_cover_lineages() {
    let paras: Vec<String> = (0..12).map(|i| words(150, &format!("p{i}q"))).collect();
    let (v, cs) = chunks_of(&paras.join("\n\n"), Strategy::Fixed, "fixed");
    assert!(cs.len() >= 3);
    for c in &cs[..cs.len() - 1] {
        assert!(!c.text.is_empty());
        assert!(
            c.token_count as usize >= MAX_TOKENS - 2 && c.token_count as usize <= MAX_TOKENS + 2
        );
    }
    // consecutive windows share the overlap region
    let tail: String = cs[0]
        .text
        .chars()
        .rev()
        .take(40)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    assert!(
        cs[1].text.contains(tail.trim()),
        "overlap of {FIXED_OVERLAP} tokens"
    );
    let snap = v.snapshot("d.md").unwrap().unwrap();
    let all: std::collections::HashSet<_> = cs.iter().flat_map(|c| c.lineage_ids.clone()).collect();
    assert_eq!(all.len(), snap.blocks.len());
    assert!(cs.iter().all(|c| c.strategy == "fixed"));
}

#[test]
fn chunking_changes_no_lineage_and_writes_no_event() {
    let root = tmp("noevent");
    let mut v = Vault::open(&root).unwrap();
    v.ingest_dir(&mini_vault(), MatcherVersion::V2).unwrap();
    let h = v.events_hash().unwrap();
    let before: Vec<_> = v
        .doc_paths()
        .iter()
        .map(|d| v.snapshot(d).unwrap().unwrap())
        .collect();
    for s in &before {
        let a = chunk(s, &v, Strategy::Structural).unwrap();
        let b = chunk(s, &v, Strategy::Structural).unwrap();
        assert_eq!(a, b);
        chunk(s, &v, Strategy::Fixed).unwrap();
    }
    assert_eq!(v.events_hash().unwrap(), h);
    let after: Vec<_> = v
        .doc_paths()
        .iter()
        .map(|d| v.snapshot(d).unwrap().unwrap())
        .collect();
    assert_eq!(before, after);
}
