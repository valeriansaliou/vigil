// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rmcp::schemars;

use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::status::Status as HealthStatus;

#[derive(Deserialize)]
pub struct ReporterRequestPayload {
    pub replica: String,
    pub interval: u64,
    pub health: Option<HealthStatus>,
    pub load: Option<ReporterRequestPayloadLoad>,
}

#[derive(Deserialize)]
pub struct ReporterRequestPayloadLoad {
    pub cpu: f32,
    pub ram: f32,
}

#[derive(Deserialize)]
pub struct ManagerAnnouncementInsertRequestPayload {
    pub title: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct ManagerProberAlertsIgnoredResolveRequestPayload {
    pub reminders_seconds: Option<u16>,
}

#[derive(Serialize)]
pub struct ManagerAnnouncementsResponsePayload {
    pub id: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct ManagerAnnouncementInsertResponsePayload {
    pub id: String,
}

#[derive(Serialize, Default)]
pub struct ManagerProberAlertsResponsePayload {
    pub dead: Vec<ManagerProberAlertsResponsePayloadEntry>,
    pub sick: Vec<ManagerProberAlertsResponsePayloadEntry>,
}

#[derive(Serialize)]
pub struct ManagerProberAlertsResponsePayloadEntry {
    pub probe: String,
    pub node: String,
    pub replica: String,
}

#[derive(Serialize)]
pub struct ManagerProberAlertsIgnoredResolveResponsePayload {
    pub reminders_seconds: Option<u16>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct StatusReportResponsePayload {
    health: HealthStatus,
    probes: Vec<StatusReportResponsePayloadProbe>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct StatusReportResponsePayloadProbe {
    pub name: String,
    pub status: HealthStatus,
    pub nodes: Vec<StatusReportResponsePayloadProbeNode>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct StatusReportResponsePayloadProbeNode {
    pub name: String,
    pub status: HealthStatus,
    pub replicas: Vec<HealthStatus>,
}

impl StatusReportResponsePayload {
    pub fn build() -> Self {
        let states = &PROBER_STORE.read().unwrap().states;

        StatusReportResponsePayload {
            health: states.status.clone(),
            probes: states
                .probes
                .iter()
                .map(|(_, probe)| StatusReportResponsePayloadProbe {
                    name: probe.label.clone(),
                    status: probe.status.clone(),
                    nodes: probe
                        .nodes
                        .iter()
                        .map(|(_, node)| StatusReportResponsePayloadProbeNode {
                            name: node.label.clone(),
                            status: node.status.clone(),
                            replicas: node
                                .replicas
                                .iter()
                                .map(|(_, replica)| replica.status.clone())
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
