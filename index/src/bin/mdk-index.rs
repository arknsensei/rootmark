//! `mdk-index`: the index half of the `mdk` CLI (GOAL P1-C "CLI"). `core`'s
//! `mdk` cannot link `index` (dependency direction core ← index), so `mdk
//! index …` / `mdk search …` execute this binary, installed next to `mdk`.
//! Every command prints JSON. Moves to `server` with `mdk` in P1-D.

use anyhow::Result;
use clap::{Parser, Subcommand};
use mdkindex::models;
use serde_json::json;

#[derive(Parser)]
#[command(name = "mdk-index", about = "mdkernel index CLI")]
struct Cli {
    /// Indent JSON output
    #[arg(long, global = true)]
    pretty: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Model files (models/MANIFEST.json)
    Models {
        #[command(subcommand)]
        cmd: ModelsCmd,
    },
}

#[derive(Subcommand)]
enum ModelsCmd {
    /// Check every model file against the manifest (size, blake3)
    Verify,
}

fn main() {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(v) => {
            let s = if cli.pretty {
                serde_json::to_string_pretty(&v)
            } else {
                serde_json::to_string(&v)
            };
            println!("{}", s.expect("JSON output"));
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

fn run(cli: &Cli) -> Result<serde_json::Value> {
    match &cli.cmd {
        Cmd::Models {
            cmd: ModelsCmd::Verify,
        } => {
            let root = models::models_root();
            let m = models::verify(&root)?;
            Ok(json!({
                "ok": true,
                "root": root.display().to_string(),
                "name": m.manifest.name,
                "revision": m.manifest.revision,
                "files": m.manifest.files.len(),
                "embedder_id": m.embedder_id(),
            }))
        }
    }
}
