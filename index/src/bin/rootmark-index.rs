//! `rootmark-index`: the index commands of the `rootmark` CLI as a standalone binary
//! (`rootmark-index index …`, `rootmark-index search …`, `rootmark-index models verify`).
//! The logic is `rootmarkindex::cli`; since P1-D the `rootmark` binary (`server`)
//! calls it directly. Every command prints JSON (`--pretty` indents it);
//! any failure is a non-zero exit with the error on stderr.

use clap::Parser;
use rootmarkcore::MatcherVersion;
use rootmarkindex::cli::{run, Cmd};

#[derive(Parser)]
#[command(
    name = "rootmark-index",
    about = "rootmark index CLI (`rootmark index …`, `rootmark search …`)"
)]
struct Cli {
    /// Indent JSON output
    #[arg(long, global = true)]
    pretty: bool,
    /// Lineage matcher version for `index watch` ingest (v2|v3; default:
    /// $ROOTMARK_MATCHER, else the vault's lock, else v3) — as `rootmark`
    #[arg(long, global = true, value_parser = ["v2", "v3"])]
    matcher: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

fn main() {
    let cli = Cli::parse();
    // as `rootmark`: the flag, else $ROOTMARK_MATCHER (validated for every command)
    let matcher = cli
        .matcher
        .clone()
        .or_else(|| std::env::var("ROOTMARK_MATCHER").ok())
        .map(|m| MatcherVersion::parse(&m));
    let res = match matcher.transpose() {
        Ok(m) => run(cli.cmd, m),
        Err(e) => Err(e),
    };
    match res {
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
