//! The server's part of the policy recipe (GOAL P1-D "Politika ve kimlik"):
//! `[server]` and `[index]` of `.mdkernel/policy.toml`, kept as written by
//! the kernel (`mdkcore::Policy::{server, index}`) and validated here.
//!
//! ```toml
//! [promotion]
//! policy = "human_only"
//!
//! [server]
//! http_port = 7357            # 1..=65535; used by `--http 127.0.0.1`
//! token_rotate_days = 30      # 1..=365
//! max_propose_bytes = 1048576 # 1..=67108864 (64 MiB)
//!
//! [index]
//! compact_every_s = 300       # 1..=86400
//! auto_rebuild = true         # a stale index is rebuilt at start (K-E0.3)
//!
//! [identity]
//! human_id = "ada"            # the human behind the HTTP token (K-D1.3)
//! ```
//!
//! `[identity] human_id` is the actor of every HTTP call (`human:<id>`);
//! without it the HTTP write tools answer 403 `denied`. It must be a
//! non-empty string without whitespace at either end and without control
//! characters.
//!
//! Each field is optional and takes the default above (`http_port`: none);
//! an unknown field, a wrong type or a value outside its bounds is an error,
//! never clamped (ARCHITECTURE §4.4, K21-T).

use anyhow::{bail, Result};
use mdkcore::Policy;
use serde_json::{json, Value};

pub const DEFAULT_TOKEN_ROTATE_DAYS: u32 = 30;
pub const DEFAULT_MAX_PROPOSE_BYTES: u64 = 1_048_576;
pub const MAX_PROPOSE_BYTES_BOUND: u64 = 64 * 1_048_576;
pub const DEFAULT_COMPACT_EVERY_S: u64 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub http_port: Option<u16>,
    pub token_rotate_days: u32,
    pub max_propose_bytes: u64,
    pub compact_every_s: u64,
    /// `[index] auto_rebuild` (K-E0.3): `mdk serve` rebuilds an index of
    /// another format at start; without it, searches answer `index_stale`
    pub auto_rebuild: bool,
    /// `[identity] human_id` (K-D1.3)
    pub human_id: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            http_port: None,
            token_rotate_days: DEFAULT_TOKEN_ROTATE_DAYS,
            max_propose_bytes: DEFAULT_MAX_PROPOSE_BYTES,
            compact_every_s: DEFAULT_COMPACT_EVERY_S,
            auto_rebuild: true,
            human_id: None,
        }
    }
}

fn int_in(table: &str, key: &str, v: &toml::Value, lo: i64, hi: i64) -> Result<i64> {
    let Some(n) = v.as_integer() else {
        bail!("[{table}] {key} must be an integer");
    };
    if n < lo || n > hi {
        bail!("[{table}] {key} = {n} outside the bounds [{lo}, {hi}]");
    }
    Ok(n)
}

impl ServerConfig {
    /// The server settings of `policy` (defaults where a field is absent).
    pub fn from_policy(policy: &Policy) -> Result<ServerConfig> {
        let mut c = ServerConfig::default();
        if let Some(t) = &policy.server {
            for (k, v) in t {
                match k.as_str() {
                    "http_port" => c.http_port = Some(int_in("server", k, v, 1, 65535)? as u16),
                    "token_rotate_days" => {
                        c.token_rotate_days = int_in("server", k, v, 1, 365)? as u32
                    }
                    "max_propose_bytes" => {
                        c.max_propose_bytes =
                            int_in("server", k, v, 1, MAX_PROPOSE_BYTES_BOUND as i64)? as u64
                    }
                    _ => bail!(
                        "[server]: unknown field `{k}` (http_port, token_rotate_days, max_propose_bytes)"
                    ),
                }
            }
        }
        if let Some(t) = &policy.index {
            for (k, v) in t {
                match k.as_str() {
                    "compact_every_s" => {
                        c.compact_every_s = int_in("index", k, v, 1, 86_400)? as u64
                    }
                    "auto_rebuild" => {
                        let Some(b) = v.as_bool() else {
                            bail!("[index] auto_rebuild must be a boolean");
                        };
                        c.auto_rebuild = b;
                    }
                    _ => bail!("[index]: unknown field `{k}` (compact_every_s, auto_rebuild)"),
                }
            }
        }
        if let Some(t) = &policy.identity {
            for (k, v) in t {
                match k.as_str() {
                    "human_id" => {
                        let Some(id) = v.as_str() else {
                            bail!("[identity] human_id must be a string");
                        };
                        if id.is_empty() || id != id.trim() || id.chars().any(char::is_control) {
                            bail!("[identity] human_id {id:?}: empty, padded with whitespace or holding control characters");
                        }
                        c.human_id = Some(id.to_string());
                    }
                    _ => bail!("[identity]: unknown field `{k}` (human_id)"),
                }
            }
        }
        Ok(c)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "server": {
                "http_port": self.http_port,
                "token_rotate_days": self.token_rotate_days,
                "max_propose_bytes": self.max_propose_bytes,
            },
            "index": {"compact_every_s": self.compact_every_s},
            "identity": {"human_id": self.human_id},
        })
    }
}

/// The recipe `mdk serve` writes into a vault that has none (GOAL P1-D:
/// the phase-1 default promotion policy is human approval only, K12/K16).
/// The trigger and journal values are the kernel's built-in ones.
pub fn default_recipe() -> String {
    let b = Policy::builtin();
    format!(
        "# written by `mdk serve` (no policy.toml was present)\n\
         [promotion]\n\
         policy = \"human_only\"\n\
         \n\
         [edit]\n\
         policy = \"append_only\"\n\
         \n\
         [version_trigger]\n\
         idle_ms = {}\n\
         min_sim_change = {:?}\n\
         \n\
         [journal]\n\
         retention_hours = {}\n\
         \n\
         [server]\n\
         token_rotate_days = {DEFAULT_TOKEN_ROTATE_DAYS}\n\
         max_propose_bytes = {DEFAULT_MAX_PROPOSE_BYTES}\n\
         \n\
         [index]\n\
         compact_every_s = {DEFAULT_COMPACT_EVERY_S}\n\
         \n\
         # the human behind the HTTP token; HTTP writes are refused without it\n\
         # [identity]\n\
         # human_id = \"<your id>\"\n",
        b.trigger.idle_ms(),
        b.trigger.min_sim_change(),
        b.retention.hours(),
    )
}
