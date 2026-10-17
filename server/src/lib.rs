//! mdkernel-server — the `mdk` CLI, MCP server and HTTP API (docs/PACKAGES.md
//! `server`; GOAL P1-D). Depends on `core` and `index` only.

pub mod config;
pub mod conformance;
pub mod error;
pub mod http;
pub mod mcp;
pub mod paths;
pub mod propose;
pub mod queue;
pub mod serve;
pub mod token;
pub mod tools;
