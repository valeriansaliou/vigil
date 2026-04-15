// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rmcp::schemars;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, schemars::JsonSchema)]
pub enum Status {
    #[serde(rename = "healthy")]
    Healthy,

    #[serde(rename = "sick")]
    Sick,

    #[serde(rename = "dead")]
    Dead,

    #[serde(rename = "partial")]
    Partial,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            &Status::Healthy => "healthy",
            &Status::Sick => "sick",
            &Status::Dead => "dead",
            &Status::Partial => "partial",
        }
    }

    pub fn as_icon(&self) -> &'static str {
        match self {
            &Status::Dead => "\u{274c}",
            &Status::Sick => "\u{26a0}",
            &Status::Healthy => "\u{2705}",
            &Status::Partial => "\u{26a0}",
        }
    }
}
