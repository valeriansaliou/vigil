// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::Url;

#[derive(Serialize, Clone)]
pub enum ReplicaURL {
    TCP(String, u16),
    HTTP(String, u16)
}

impl ReplicaURL {
    pub fn parse_from(raw_url: &str) -> Result<ReplicaURL, ()> {
        match Url::parse(raw_url) {
            Ok(parsed_url) => {
                match (parsed_url.host_str(), parsed_url.port()) {
                    (Some(host), Some(port)) => {
                        match parsed_url.scheme() {
                            "tcp" => Ok(ReplicaURL::TCP(host.to_string(), port)),
                            "http" | "https" => Ok(ReplicaURL::HTTP(host.to_string(), port)),
                            _ => Err(()),
                        }
                    },
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}
