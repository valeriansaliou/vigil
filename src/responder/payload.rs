// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
pub struct ManagerProberAlertsIgnoredListResponsePayload {
    pub probe: String,
    pub node: String,
    pub replica: String,
}
