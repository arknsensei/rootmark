use rootmarkcore::{extract_wikilinks, normalize, parse_document, version_id, BlockType, WikiLink};
use std::path::PathBuf;

fn types(text: &str) -> Vec<BlockType> {
    parse_document(text, "t.md")
        .unwrap()
        .0
        .iter()
        .map(|b| b.block_type)
        .collect()
}

#[test]
fn every_block_type() {
    let md = "# Title\n\nA paragraph.\n\n- one\n- two\n\n```rust\nfn x() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n> quoted\n> text\n\n---\n\n<div>html</div>\n\nRef[^1].\n\n[^1]: A footnote.\n";
    assert_eq!(
        types(md),
        vec![
            BlockType::Heading,
            BlockType::Paragraph,
            BlockType::ListItem,
            BlockType::ListItem,
            BlockType::Code,
            BlockType::Table,
            BlockType::Blockquote,
            BlockType::Other, // rule
            BlockType::Other, // html
            BlockType::Paragraph,
            BlockType::Other, // footnote definition
        ]
    );
}

#[test]
fn heading_levels_and_paths() {
    let md = "# A\n\npa\n\n## B\n\npb\n\n### C\n\npc\n\n## D\n\npd\n";
    let (b, _) = parse_document(md, "t.md").unwrap();
    let lv: Vec<_> = b.iter().map(|x| x.heading_level).collect();
    assert_eq!(
        lv,
        vec![Some(1), None, Some(2), None, Some(3), None, Some(2), None]
    );
    assert!(b[0].heading_path.is_empty());
    assert_eq!(b[2].heading_path, vec!["A"]); // own path excludes itself
    assert_eq!(b[5].heading_path, vec!["A", "B", "C"]);
    assert_eq!(b[7].heading_path, vec!["A", "D"]);
}

#[test]
fn frontmatter_is_split_and_produces_no_block() {
    let md = "---\ntitle: x\ntags: [a]\n---\n\n# H\n\nBody.\n";
    let (b, fm) = parse_document(md, "t.md").unwrap();
    assert_eq!(fm.as_deref(), Some("title: x\ntags: [a]\n"));
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].block_type, BlockType::Heading);
    // a rule later in the document is not frontmatter
    let (b2, fm2) = parse_document("Text\n\n---\n\nMore\n", "t.md").unwrap();
    assert!(fm2.is_none());
    assert_eq!(b2.len(), 3);
}

#[test]
fn nested_list_belongs_to_parent_item() {
    let md = "1. first\n   - nested a\n   - nested b\n2. second\n";
    let (b, _) = parse_document(md, "t.md").unwrap();
    assert_eq!(b.len(), 2);
    assert!(b.iter().all(|x| x.block_type == BlockType::ListItem));
    assert!(b[0].text.contains("nested a") && b[0].text.contains("nested b"));
    assert_eq!(b[0].normalized_text, "1. first - nested a - nested b");
}

#[test]
fn four_wikilink_forms() {
    let t = "See [[alpha]], [[beta|Beta]], [[gamma#Intro]], [[ dir/delta.md #Usage | label ]] and [[]].";
    assert_eq!(
        extract_wikilinks(t),
        vec![
            WikiLink {
                target: "alpha".into(),
                heading: None
            },
            WikiLink {
                target: "beta".into(),
                heading: None
            },
            WikiLink {
                target: "gamma".into(),
                heading: Some("Intro".into())
            },
            WikiLink {
                target: "delta".into(),
                heading: Some("Usage".into())
            },
        ]
    );
}

#[test]
fn nfc_and_whitespace() {
    // "é" decomposed (e + U+0301) vs precomposed
    let a = normalize("caf\u{65}\u{301}  au\tlait\r\n", false);
    let b = normalize("  caf\u{e9} au lait", false);
    assert_eq!(a, b);
    assert_eq!(a, "café au lait");
    // emphasis markers kept
    assert_eq!(normalize("**bold**  _it_", false), "**bold** _it_");
}

#[test]
fn code_block_whitespace_preserved() {
    let md = "```\nfn  x() {\r\n    let  y = 1;\n}\n```\n";
    let (b, _) = parse_document(md, "t.md").unwrap();
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].block_type, BlockType::Code);
    assert!(b[0].normalized_text.contains("fn  x() {\n    let  y = 1;"));
    assert_eq!(normalize("  a  \r\n", true), "  a  \n");
}

#[test]
fn empty_paragraph_produces_no_block() {
    let (b, _) = parse_document("\n\n   \n\n\u{a0}\n\nreal\n", "t.md").unwrap();
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].normalized_text, "real");
}

#[test]
fn same_text_same_version_id() {
    let (a, _) = parse_document("Hello   world\nagain\n", "a.md").unwrap();
    let (b, _) = parse_document("Hello world again", "b.md").unwrap();
    assert_eq!(a[0].version_id, b[0].version_id);
    assert_eq!(
        a[0].version_id,
        version_id(BlockType::Paragraph, "Hello world again")
    );
    assert_eq!(a[0].version_id.len(), 32);
    // same text, different type → different id
    assert_ne!(
        version_id(BlockType::Paragraph, "x"),
        version_id(BlockType::Other, "x")
    );
}

#[test]
fn mini_vault_parses() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/fixtures/mini_vault");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".md"))
        .collect();
    names.sort();
    assert_eq!(names.len(), 12);
    let mut total = 0;
    for n in &names {
        let text = std::fs::read_to_string(dir.join(n)).unwrap();
        let (b, _) = parse_document(&text, n).unwrap();
        assert!(!b.is_empty(), "{n}");
        println!("BLOCKS {n} {}", b.len());
        total += b.len();
    }
    println!("BLOCKS TOTAL {total}");
}

#[test]
fn heading_titles_atx_and_setext() {
    use rootmarkcore::parse::heading_title;
    assert_eq!(heading_title("## Stages ##"), "Stages");
    assert_eq!(heading_title("# C#"), "C#");
    assert_eq!(heading_title("Title ==="), "Title");
    assert_eq!(heading_title("Sub ---"), "Sub");
    let (b, _) = parse_document("Top\n===\n\ntext\n", "t.md").unwrap();
    assert_eq!(b[0].heading_level, Some(1));
    assert_eq!(b[1].heading_path, vec!["Top"]);
}

#[test]
fn line_spans_count_from_document_start() {
    let md =
        "---\na: 1\n---\n\n# H\n\npara one\ncontinues\n\n- item\n  - nested\n\n```\ncode\n```\n";
    let (b, _) = parse_document(md, "t.md").unwrap();
    let spans: Vec<_> = b.iter().map(|x| (x.line_start, x.line_end)).collect();
    assert_eq!(spans, vec![(5, 5), (7, 8), (10, 11), (13, 15)]);
}

#[test]
fn line_spans_with_crlf() {
    let md = "# H\r\n\r\npara\r\nmore\r\n\r\n- x\r\n";
    let (b, _) = parse_document(md, "t.md").unwrap();
    let spans: Vec<_> = b.iter().map(|x| (x.line_start, x.line_end)).collect();
    assert_eq!(spans, vec![(1, 1), (3, 4), (6, 6)]);
}
