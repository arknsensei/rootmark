//! MCP server over stdio (GOAL P1-D; the official Rust SDK `rmcp`). The
//! tools are `crate::tools`; this module only maps them onto MCP:
//! `tools/list` gives every `ToolSpec` (name, description, JSON Schema),
//! `tools/call` runs the tool and returns its JSON as structured content
//! (and as text). A failing tool is a tool result with `isError: true`, the
//! error message as text and `structuredContent: {"error": {"code",
//! "message"}}` with a stable code (K-D1.5) — never a silent empty result.
//!
//! MCP is the agent channel (K-D1.3): every call's actor is
//! `agent:<client>`. Writing calls take their writer-queue ticket when the
//! transport reads the request (`Sequenced`), so a pipelining client's calls
//! run in the order it sent them (K-D1.4); read-only calls run in parallel.
//!
//! The consumer recorded in traces is the client's identity from the
//! `initialize` handshake, `<clientInfo.name>/<clientInfo.version>`. Protocol
//! versions without that handshake are not offered, and a tool that records
//! a trace refuses to run for a client that has not identified itself.

use crate::queue::Ticket;
use crate::tools::{self, Caller, Kernel};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ClientRequest, ContentBlock,
    Implementation, JsonRpcMessage, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    RequestId, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::model::{ClientNotification, ServerResult};
use rmcp::service::{
    NotificationContext, RequestContext, RxJsonRpcMessage, Service, ServiceRole, TxJsonRpcMessage,
};
use rmcp::transport::Transport;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Writer-queue tickets of the `tools/call` requests read but not yet run.
type Tickets = Arc<Mutex<HashMap<RequestId, Ticket>>>;

#[derive(Clone)]
pub struct McpServer {
    kernel: Arc<Kernel>,
    tickets: Tickets,
}

impl McpServer {
    pub fn new(kernel: Arc<Kernel>) -> McpServer {
        McpServer {
            kernel,
            tickets: Arc::default(),
        }
    }
}

/// A transport that takes a writer-queue ticket for every writing
/// `tools/call` as it reads it (reading is sequential: arrival order), and
/// drops a ticket whose request was answered without running its tool (an
/// argument error of the SDK), so it never blocks the queue. A cancelled
/// call (`notifications/cancelled`) keeps its ticket: the SDK still runs it,
/// in its arrival place.
pub struct Sequenced<T> {
    inner: T,
    kernel: Arc<Kernel>,
    tickets: Tickets,
}

impl<T: Transport<RoleServer>> Transport<RoleServer> for Sequenced<T> {
    type Error = T::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleServer>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        let id = match &item {
            JsonRpcMessage::Response(r) => Some(r.id.clone()),
            JsonRpcMessage::Error(e) => e.id.clone(),
            _ => None,
        };
        if let Some(id) = id {
            self.tickets
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
        }
        self.inner.send(item)
    }

    fn receive(&mut self) -> impl Future<Output = Option<RxJsonRpcMessage<RoleServer>>> + Send {
        let kernel = self.kernel.clone();
        let tickets = self.tickets.clone();
        let next = self.inner.receive();
        async move {
            let m = next.await?;
            // `notifications/cancelled` leaves the ticket alone: rmcp still
            // runs the cancelled call, and it must run in its arrival place
            // (a cancelled write is neither undone nor moved; K-D1.4)
            if let JsonRpcMessage::Request(r) = &m {
                if let ClientRequest::CallToolRequest(c) = &r.request {
                    if tools::is_writer(&c.params.name) {
                        tickets
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .insert(r.id.clone(), kernel.ticket());
                    }
                }
            }
            Some(m)
        }
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.inner.close()
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
        let ticket = self
            .tickets
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&context.id);
        let res = tokio::task::spawn_blocking(move || {
            kernel.run(ticket, &name, &args, Caller::mcp(consumer.as_deref()))
        })
        .await
        .map_err(|e| McpError::internal_error(format!("tool task failed: {e}"), None))?;
        Ok(match res {
            Ok(v) => CallToolResult::structured(v),
            Err(e) => {
                let mut r =
                    CallToolResult::error(vec![ContentBlock::text(format!("error: {e:#}"))]);
                r.structured_content = Some(crate::error::error_json(&e));
                r
            }
        }
        .into())
    }
}

/// The served service: `McpServer`, and after every request handler
/// returns — whatever its outcome (a tool result, an SDK error before the
/// tool ran, a cancelled call whose response the SDK drops) — the request's
/// ticket, if its tool did not take it, is dropped (skipped), so no ticket
/// outlives its request (D1.2 review 2).
pub struct Ordered {
    server: McpServer,
}

impl Service<RoleServer> for Ordered {
    async fn handle_request(
        &self,
        request: <RoleServer as ServiceRole>::PeerReq,
        context: RequestContext<RoleServer>,
    ) -> Result<ServerResult, McpError> {
        let id = context.id.clone();
        let r = Service::handle_request(&self.server, request, context).await;
        self.server
            .tickets
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&id);
        r
    }

    async fn handle_notification(
        &self,
        notification: ClientNotification,
        context: NotificationContext<RoleServer>,
    ) -> Result<(), McpError> {
        Service::handle_notification(&self.server, notification, context).await
    }

    fn get_info(&self) -> <RoleServer as ServiceRole>::Info {
        Service::get_info(&self.server)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Service::supported_protocol_versions(&self.server)
    }
}

/// Serves `kernel` over stdin/stdout until the client closes the stream.
pub async fn serve_stdio(kernel: Arc<Kernel>) -> anyhow::Result<()> {
    let server = McpServer::new(kernel.clone());
    let (stdin, stdout) = rmcp::transport::stdio();
    let transport = Sequenced {
        inner: rmcp::transport::async_rw::AsyncRwTransport::new(stdin, stdout),
        kernel,
        tickets: server.tickets.clone(),
    };
    let service = Ordered { server }
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("MCP initialize failed: {e}"))?;
    service.waiting().await?;
    Ok(())
}
