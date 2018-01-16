// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

use url_serde::SerdeUrl;

use super::defaults;

#[derive(Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub assets: ConfigAssets,
    pub branding: ConfigBranding,
    pub probe: ConfigProbe,
}

#[derive(Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_log_level")]
    pub log_level: String,

    #[serde(default = "defaults::server_inet")]
    pub inet: SocketAddr,
}

#[derive(Deserialize)]
pub struct ConfigAssets {
    #[serde(default = "defaults::assets_path")]
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct ConfigBranding {
    #[serde(default = "defaults::branding_page_title")]
    pub page_title: String,

    pub company_name: String,
    pub icon_color: String,
    pub icon_url: SerdeUrl,
    pub logo_color: String,
    pub logo_url: SerdeUrl,
    pub website_url: SerdeUrl,
    pub support_url: SerdeUrl,
    pub custom_html: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigProbe {
    pub service: Vec<ConfigProbeService>,
}

#[derive(Deserialize)]
pub struct ConfigProbeService {
    pub id: String,
    pub label: String,
    pub node: Vec<ConfigProbeServiceNode>,
}

#[derive(Deserialize)]
pub struct ConfigProbeServiceNode {
    pub id: String,
    pub label: String,
    pub mode: ConfigProbeServiceNodeMode,
    pub replicas: Vec<String>,
}

#[derive(Deserialize)]
pub enum ConfigProbeServiceNodeMode {
    #[serde(rename = "poll")]
    Poll,

    #[serde(rename = "push")]
    Push,
}
