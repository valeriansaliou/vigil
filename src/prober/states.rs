// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ordermap::OrderMap;

use super::status::Status;
use super::mode::Mode;

#[derive(Serialize)]
pub struct ServiceStates {
    pub status: Status,
    pub date: Option<String>,
    pub probes: OrderMap<String, ServiceStatesProbe>,
}

#[derive(Serialize)]
pub struct ServiceStatesProbe {
    pub status: Status,
    pub label: String,
    pub nodes: OrderMap<String, ServiceStatesProbeNode>,
}

#[derive(Serialize)]
pub struct ServiceStatesProbeNode {
    pub status: Status,
    pub label: String,
    pub mode: Mode,
    pub replicas: OrderMap<String, Status>,
}
