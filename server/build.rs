// Embeds spec/fixtures into the `rootmark` binary for `rootmark conformance` (P1-E/E5,
// GOAL E-7: every release binary runs the conformance suite on its own).
// The list is the fixture tree as `spec/fixtures/MANIFEST.json` covers it
// (every file but `.rootmark/` and `.DS_Store`, MANIFEST.json included);
// the bytes are `include_bytes!`, so the binary holds exactly the frozen
// fixtures it was built from (the run still checks them against the
// manifest).
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        if name == ".rootmark" || name == ".DS_Store" {
            continue;
        }
        let p = e.path();
        if e.file_type().expect("file type").is_dir() {
            walk(base, &p, out);
        } else {
            let rel = p
                .strip_prefix(base)
                .expect("under base")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel, p));
        }
    }
}

fn main() {
    // the manifest directory of this build (read at run time: a build
    // script compiled once may run for another copy of the sources)
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let root = Path::new(&manifest).join("../spec/fixtures");
    let root = root.canonicalize().expect("spec/fixtures");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    assert!(
        files.iter().any(|(r, _)| r == "MANIFEST.json"),
        "spec/fixtures/MANIFEST.json missing"
    );
    let mut src = String::from("/// spec/fixtures, embedded at build time (build.rs).\npub static FIXTURES: &[(&str, &[u8])] = &[\n");
    for (rel, p) in &files {
        println!("cargo:rerun-if-changed={}", p.display());
        writeln!(
            src,
            "    ({rel:?}, include_bytes!({:?})),",
            p.display().to_string()
        )
        .unwrap();
    }
    src.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("fixtures.rs");
    std::fs::write(out, src).expect("write fixtures.rs");
}
