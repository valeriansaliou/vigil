// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::{Host, Url};

#[derive(Serialize, Debug, Clone)]
pub enum ReplicaURL {
    ICMP(String),
    TCP(String, u16),
    SSH(String, u16),
    HTTP(String),
    HTTPS(String),
}

impl ReplicaURL {
    pub fn parse_from(raw_url: &str) -> Result<ReplicaURL, ()> {
        match Url::parse(raw_url) {
            Ok(url) => match url.scheme() {
                "icmp" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), None, None) => Ok(ReplicaURL::ICMP(Self::host_string(host))),
                    _ => Err(()),
                },
                "tcp" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), Some(port), None) => {
                        Ok(ReplicaURL::TCP(Self::host_string(host), port))
                    }
                    _ => Err(()),
                },
                "ssh" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), Some(port), None) => {
                        Ok(ReplicaURL::SSH(Self::host_string(host), port))
                    }
                    _ => Err(()),
                },
                "http" => Ok(ReplicaURL::HTTP(url.into())),
                "https" => Ok(ReplicaURL::HTTPS(url.into())),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    fn host_string(host: Host<&str>) -> String {
        // Convert internal host value into string. This is especially useful for IPv6 addresses, \
        //   which we need returned in '::1' format; as they would otherwise be returned in \
        //   '[::1]' format using built-in top-level 'to_string()' method on the 'Host' trait. The \
        //   underlying address parser does not accept IPv6 addresses formatted as '[::1]', so \
        //   this seemingly overkill processing is obviously needed.
        match host {
            Host::Domain(domain_value) => domain_value.to_string(),
            Host::Ipv4(ipv4_value) => ipv4_value.to_string(),
            Host::Ipv6(ipv6_value) => ipv6_value.to_string(),
        }
    }
}
