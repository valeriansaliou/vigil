// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::Url;

#[derive(Serialize, Debug, Clone)]
pub enum ReplicaURL {
    TCP(String, u16),
    HTTP(String),
    HTTPS(String),
}

impl ReplicaURL {
    pub fn parse_from(raw_url: &str) -> Result<ReplicaURL, ()> {
        match Url::parse(raw_url) {
            Ok(parsed_url) => match parsed_url.scheme() {
                "tcp" => match (parsed_url.host_str(), parsed_url.port()) {
                    (Some(host), Some(port)) => Ok(ReplicaURL::TCP(host.to_string(), port)),
                    _ => Err(()),
                },
                "http" => Ok(ReplicaURL::HTTP(parsed_url.into_string())),
                "https" => Ok(ReplicaURL::HTTPS(parsed_url.into_string())),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
