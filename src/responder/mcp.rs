// Vigil
//
// Microservices Status Page
// Copyright: 2025, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rmcp::{
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};

use super::payload::StatusReportResponsePayload;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ProbesRequest {}

#[derive(Clone)]
pub struct Probes {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Probes {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Gets the general health report for all Vigil probes")]
    fn get_report(
        &self,
        Parameters(_): Parameters<ProbesRequest>,
    ) -> Json<StatusReportResponsePayload> {
        Json(StatusReportResponsePayload::build())
    }
}

#[tool_handler]
impl ServerHandler for Probes {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Vigil Probes".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
