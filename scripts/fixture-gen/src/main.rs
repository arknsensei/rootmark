//! Writes `expected.json` for every conformance case and `MANIFEST.json`
//! (scripts/gen_fixtures.sh). Built against the v3-final core only.
//!
//!     rootmark-fixture-gen <spec/fixtures> <v3-final sha>

#[path = "eval.rs"]
mod eval;

use anyhow::{bail, Result};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        bail!("usage: rootmark-fixture-gen <fixtures-dir> <v3-final-sha>");
    }
    let root = PathBuf::from(&args[1]);
    let cases = eval::case_dirs(&root)?;
    if cases.is_empty() {
        bail!("no case.json under {}", root.display());
    }
    for c in &cases {
        let v = eval::evaluate(c)?;
        let mut text = serde_json::to_string_pretty(&v)?;
        text.push('\n');
        fs::write(c.join("expected.json"), text)?;
        eprintln!("wrote {}", c.join("expected.json").display());
    }
    let manifest = json!({
        "generated_by": "scripts/gen_fixtures.sh",
        "core": {"tag": "v3-final", "sha": args[2]},
        "files": eval::fixture_manifest(&root)?,
    });
    let mut text = serde_json::to_string_pretty(&manifest)?;
    text.push('\n');
    fs::write(root.join("MANIFEST.json"), text)?;
    Ok(())
}
