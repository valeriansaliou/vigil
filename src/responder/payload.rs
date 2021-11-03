// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::prober::status::Status as HealthStatus;

#[derive(Deserialize)]
pub struct ReporterPayload {
    pub replica: String,
    pub interval: u64,
    pub health: Option<HealthStatus>,
    pub load: Option<ReporterPayloadLoad>,
}

#[derive(Deserialize)]
pub struct ReporterPayloadLoad {
    pub cpu: f32,
    pub ram: f32,
}
