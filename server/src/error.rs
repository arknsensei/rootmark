//! Stable error codes of the tools (K-D1.5; `docs/CLIENT_CONTRACT.md`). A
//! failing tool returns its code in MCP `structuredContent.error.code` and
//! as the HTTP status below; the message is for people, the code for
//! programs. An error nobody classified is `internal` — a bug to report,
//! never a silent success.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    /// arguments do not match the tool's schema or rules
    InvalidArguments,
    /// an empty or whitespace-only query (spec §9)
    EmptyQuery,
    /// a document path that leaves the vault or its documents
    PathOutsideVault,
    /// the thing to create exists already (document, checkpoint name)
    Exists,
    /// an unknown lineage, document, checkpoint or tool
    NotFound,
    /// over a size limit (`[server] max_propose_bytes`, request body)
    TooLarge,
    /// `replace_section` of a heading the document lacks (or holds twice)
    HeadingNotFound,
    /// a policy or the channel refuses the call (`EditPolicy`, an HTTP
    /// write without `[identity] human_id`, a missing or wrong token)
    Denied,
    /// the vault's `policy.toml` is invalid
    PolicyViolation,
    /// anything else: a failure of the server or the vault
    Internal,
}

impl Code {
    pub const ALL: [Code; 10] = [
        Code::InvalidArguments,
        Code::EmptyQuery,
        Code::PathOutsideVault,
        Code::Exists,
        Code::NotFound,
        Code::TooLarge,
        Code::HeadingNotFound,
        Code::Denied,
        Code::PolicyViolation,
        Code::Internal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Code::InvalidArguments => "invalid_arguments",
            Code::EmptyQuery => "empty_query",
            Code::PathOutsideVault => "path_outside_vault",
            Code::Exists => "exists",
            Code::NotFound => "not_found",
            Code::TooLarge => "too_large",
            Code::HeadingNotFound => "heading_not_found",
            Code::Denied => "denied",
            Code::PolicyViolation => "policy_violation",
            Code::Internal => "internal",
        }
    }

    /// The HTTP status of a tool error with this code (401 is the
    /// transport's: a missing or wrong token, code `denied`).
    pub fn http_status(self) -> u16 {
        match self {
            Code::InvalidArguments | Code::EmptyQuery | Code::PathOutsideVault => 400,
            Code::Denied => 403,
            Code::NotFound => 404,
            Code::Exists => 409,
            Code::TooLarge => 413,
            Code::HeadingNotFound | Code::PolicyViolation => 422,
            Code::Internal => 500,
        }
    }
}

/// An error with its code; the root of the `anyhow` chain of a classified
/// tool error (`code_of` finds it under any added context).
#[derive(Debug)]
pub struct Coded {
    pub code: Code,
    pub message: String,
}

impl std::fmt::Display for Coded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Coded {}

/// A new error with `code`.
pub fn coded(code: Code, message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(Coded {
        code,
        message: message.into(),
    })
}

/// `e` (with its whole message) under `code`, unless it already has one.
pub fn recode(code: Code, e: anyhow::Error) -> anyhow::Error {
    if code_in(&e).is_some() {
        return e;
    }
    coded(code, format!("{e:#}"))
}

fn code_in(e: &anyhow::Error) -> Option<Code> {
    e.chain()
        .find_map(|c| c.downcast_ref::<Coded>().map(|c| c.code))
}

/// The code of a tool error; unclassified → `internal`.
pub fn code_of(e: &anyhow::Error) -> Code {
    code_in(e).unwrap_or(Code::Internal)
}

/// `Result` → the same result with its error under `code` (if it has none).
pub trait CodeExt<T> {
    fn code(self, code: Code) -> anyhow::Result<T>;
}

impl<T, E: Into<anyhow::Error>> CodeExt<T> for Result<T, E> {
    fn code(self, code: Code) -> anyhow::Result<T> {
        self.map_err(|e| recode(code, e.into()))
    }
}

/// `bail!` with a code.
#[macro_export]
macro_rules! bail_code {
    ($code:expr, $($arg:tt)*) => {
        return Err($crate::error::coded($code, format!($($arg)*)))
    };
}

/// The JSON of a tool error: `{"error": {"code", "message"}}` (MCP
/// `structuredContent`, HTTP body).
pub fn error_json(e: &anyhow::Error) -> serde_json::Value {
    serde_json::json!({"error": {"code": code_of(e).as_str(), "message": format!("{e:#}")}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_survives_context_and_defaults_to_internal() {
        let e = coded(Code::Exists, "x exists").context("propose");
        assert_eq!(code_of(&e), Code::Exists);
        assert_eq!(code_of(&anyhow::anyhow!("boom")), Code::Internal);
        let r: anyhow::Result<()> = Err(anyhow::anyhow!("bad")).code(Code::NotFound);
        assert_eq!(code_of(&r.unwrap_err()), Code::NotFound);
        // an existing code is kept
        let r: anyhow::Result<()> = Err(coded(Code::TooLarge, "big")).code(Code::Internal);
        assert_eq!(code_of(&r.unwrap_err()), Code::TooLarge);
        let names: Vec<&str> = Code::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(names.len(), 10);
        for c in Code::ALL {
            assert_eq!(serde_json::to_value(c).unwrap(), c.as_str());
        }
    }
}
