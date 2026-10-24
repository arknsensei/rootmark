//! rootmark-server — the `rootmark` CLI, MCP server and HTTP API (docs/PACKAGES.md
//! `server`; GOAL P1-D). Depends on `core` and `index` only.

pub mod attest;
pub mod attest_log;
pub mod config;
pub mod conformance;
pub mod error;
pub mod http;
pub mod mcp;
pub mod migrate;
pub mod paths;
pub mod propose;
pub mod queue;
pub mod rules;
pub mod serve;
pub mod token;
pub mod tools;
