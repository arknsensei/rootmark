//! Local HTTP API (GOAL P1-D "HTTP API"): the MCP tools as `POST
//! /v1/<tool>` (JSON object body = the tool's arguments; empty body = `{}`)
//! and `GET /v1/status`. Only `127.0.0.1` is served. Every request, whatever
//! its path, needs `Authorization: Bearer <token>` (`crate::token`; missing
//! or wrong → 401) and then a non-empty `X-Consumer` (→ 400), which is the
//! consumer recorded in traces.
//!
//! Status codes: 200 tool result (JSON); 400 bad request (consumer, body not
//! a JSON object); 401 token; 404 unknown path or tool; 405 wrong method;
//! 413 body over the limit; 422 the tool failed (`{"error": "…"}`).

use crate::tools::{self, Kernel};
use anyhow::{bail, Context, Result};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Router;
use serde_json::{json, Value};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    kernel: Arc<Kernel>,
    token: Arc<String>,
}

/// `--http` value: `127.0.0.1:PORT` (port 0: any free port), or `127.0.0.1`
/// with the port from `[server] http_port`. Any other address is refused:
/// the API never listens on an external interface.
pub fn parse_addr(s: &str, policy_port: Option<u16>) -> Result<SocketAddr> {
    let addr: SocketAddr = match s.parse::<SocketAddr>() {
        Ok(a) => a,
        Err(_) => match s.parse::<std::net::IpAddr>() {
            Ok(ip) => {
                let Some(p) = policy_port else {
                    bail!("--http {s}: no port given and no [server] http_port in policy.toml");
                };
                SocketAddr::new(ip, p)
            }
            Err(_) => bail!("--http {s}: expected 127.0.0.1:PORT"),
        },
    };
    if addr.ip() != std::net::IpAddr::V4(Ipv4Addr::LOCALHOST) {
        bail!(
            "--http {s}: refused, the HTTP API listens on 127.0.0.1 only (no external interface)"
        );
    }
    Ok(addr)
}

fn reply(code: StatusCode, v: Value) -> Response {
    (
        code,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec(&v).expect("JSON output"),
    )
        .into_response()
}

fn err(code: StatusCode, msg: impl Into<String>) -> Response {
    reply(code, json!({"error": msg.into()}))
}

/// Byte comparison whose time does not depend on where the inputs differ.
fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn handle(
    State(st): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    match auth {
        Some(t) if same(t.as_bytes(), st.token.as_bytes()) => {}
        Some(_) => return err(StatusCode::UNAUTHORIZED, "wrong token"),
        None => {
            return err(
                StatusCode::UNAUTHORIZED,
                "Authorization: Bearer <token> required",
            )
        }
    }
    let consumer = match headers.get("x-consumer").map(|v| v.to_str()) {
        Some(Ok(c)) if !c.trim().is_empty() => c.to_string(),
        Some(Ok(_)) => return err(StatusCode::BAD_REQUEST, "X-Consumer is empty"),
        Some(Err(_)) => return err(StatusCode::BAD_REQUEST, "X-Consumer is not visible ASCII"),
        None => return err(StatusCode::BAD_REQUEST, "X-Consumer header required"),
    };
    let Some(tool) = uri.path().strip_prefix("/v1/") else {
        return err(StatusCode::NOT_FOUND, format!("no route {}", uri.path()));
    };
    if !tools::specs().iter().any(|s| s.name == tool) {
        return err(StatusCode::NOT_FOUND, format!("unknown tool {tool:?}"));
    }
    let args = match (&method, tool) {
        (&Method::GET, "status") => Value::Null,
        (&Method::POST, _) => {
            if body.is_empty() {
                Value::Null
            } else {
                match serde_json::from_slice::<Value>(&body) {
                    Ok(v @ Value::Object(_)) => v,
                    Ok(_) => return err(StatusCode::BAD_REQUEST, "body must be a JSON object"),
                    Err(e) => {
                        return err(StatusCode::BAD_REQUEST, format!("body is not JSON: {e}"))
                    }
                }
            }
        }
        _ => {
            return err(
                StatusCode::METHOD_NOT_ALLOWED,
                format!("{method} /v1/{tool}: use POST (GET only for status)"),
            )
        }
    };
    let kernel = st.kernel.clone();
    let tool = tool.to_string();
    let res =
        tokio::task::spawn_blocking(move || kernel.call(&tool, &args, Some(consumer.as_str())))
            .await;
    match res {
        Ok(Ok(v)) => reply(StatusCode::OK, v),
        Ok(Err(e)) => err(StatusCode::UNPROCESSABLE_ENTITY, format!("{e:#}")),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("tool task failed: {e}"),
        ),
    }
}

/// The router (tests drive it without a socket).
pub fn router(kernel: Arc<Kernel>, token: String, body_limit: usize) -> Router {
    Router::new()
        .fallback(handle)
        .layer(DefaultBodyLimit::max(body_limit))
        .with_state(AppState {
            kernel,
            token: Arc::new(token),
        })
}

/// Binds `addr` (127.0.0.1 only, `parse_addr`) and returns the listener and
/// the bound address.
pub async fn bind(addr: SocketAddr) -> Result<(tokio::net::TcpListener, SocketAddr)> {
    if addr.ip() != std::net::IpAddr::V4(Ipv4Addr::LOCALHOST) {
        bail!("refused to bind {addr}: 127.0.0.1 only");
    }
    let l = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    let a = l.local_addr()?;
    Ok((l, a))
}
