use crate::prober::status::Status as HealthStatus;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct ReporterData {
    pub replica: String,
    pub interval: u64,
    pub health: Option<HealthStatus>,
    pub load: Option<ReporterDataLoad>,
}

#[derive(Deserialize)]
pub struct ReporterDataLoad {
    pub cpu: f32,
    pub ram: f32,
}
