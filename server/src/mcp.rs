//! MCP server over stdio (GOAL P1-D; the official Rust SDK `rmcp`). The
//! tools are `crate::tools`; this module only maps them onto MCP:
//! `tools/list` gives every `ToolSpec` (name, description, JSON Schema),
//! `tools/call` runs the tool and returns its JSON as structured content
//! (and as text). A failing tool is a tool result with `isError: true` and
//! the error message — never a silent empty result.
//!
//! The consumer recorded in traces is the client's identity from the
//! `initialize` handshake, `<clientInfo.name>/<clientInfo.version>`. Protocol
//! versions without that handshake are not offered, and a tool that records
//! a trace refuses to run for a client that has not identified itself.

use crate::tools::{self, Kernel};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig,
    Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct McpServer {
    kernel: Arc<Kernel>,
}

impl McpServer {
    pub fn new(kernel: Arc<Kernel>) -> McpServer {
        McpServer { kernel }
    }
}

/// `<name>/<version>` of the client's `initialize` request; `None` when the
/// name or the version is empty or whitespace (an unidentified client, as an
/// empty HTTP `X-Consumer`), so trace-recording tools refuse to run.
pub fn consumer_of(info: &Implementation) -> Option<String> {
    if info.name.trim().is_empty() || info.version.trim().is_empty() {
        return None;
    }
    Some(format!("{}/{}", info.name, info.version))
}

fn tool_list() -> Vec<Tool> {
    tools::specs()
        .into_iter()
        .map(|s| {
            let schema = match s.input_schema {
                Value::Object(m) => m,
                _ => unreachable!("tool schemas are objects (tools::object_schema)"),
            };
            Tool::new(s.name, s.description, Arc::new(schema))
        })
        .collect()
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("mdk", tools::MDK_VERSION);
        info.instructions = Some(format!(
            "mdkernel {} (spec {}): block-level lineage, provenance, layers, checkpoints and \
             consumption traces over the Markdown vault {}.",
            tools::MDK_VERSION,
            tools::SPEC_VERSION,
            self.kernel.vault().display()
        ));
        info
    }

    /// Only revisions with the `initialize` handshake: the consumer
    /// identity comes from it.
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tool_list()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        tool_list().into_iter().find(|t| t.name == name)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let name = request.name.to_string();
        if !tools::specs().iter().any(|s| s.name == name) {
            return Err(McpError::invalid_params(
                format!("unknown tool {name:?}"),
                None,
            ));
        }
        let consumer = context
            .peer
            .peer_info()
            .and_then(|p| consumer_of(&p.client_info));
        let args = request.arguments.map(Value::Object).unwrap_or(Value::Null);
        let kernel = self.kernel.clone();
        let res =
            tokio::task::spawn_blocking(move || kernel.call(&name, &args, consumer.as_deref()))
                .await
                .map_err(|e| McpError::internal_error(format!("tool task failed: {e}"), None))?;
        Ok(match res {
            Ok(v) => CallToolResult::structured(v),
            Err(e) => CallToolResult::error(vec![ContentBlock::text(format!("error: {e:#}"))]),
        }
        .into())
    }
}

/// Serves `kernel` over stdin/stdout until the client closes the stream.
pub async fn serve_stdio(kernel: Arc<Kernel>) -> anyhow::Result<()> {
    let service = McpServer::new(kernel)
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| anyhow::anyhow!("MCP initialize failed: {e}"))?;
    service.waiting().await?;
    Ok(())
}
