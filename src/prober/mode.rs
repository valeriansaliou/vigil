// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Mode {
    #[serde(rename = "poll")]
    Poll,

    #[serde(rename = "push")]
    Push,
}
