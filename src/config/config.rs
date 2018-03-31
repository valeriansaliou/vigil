// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

use url_serde::SerdeUrl;

use super::defaults;
use super::regex::Regex;
use prober::mode::Mode;

#[derive(Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub assets: ConfigAssets,
    pub branding: ConfigBranding,
    pub metrics: ConfigMetrics,
    pub plugins: Option<ConfigPlugins>,
    pub notify: Option<ConfigNotify>,
    pub probe: ConfigProbe,
}

#[derive(Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_log_level")]
    pub log_level: String,

    #[serde(default = "defaults::server_inet")]
    pub inet: SocketAddr,

    #[serde(default = "defaults::server_workers")]
    pub workers: u16,

    pub reporter_token: String,
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

    pub page_url: SerdeUrl,
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
pub struct ConfigMetrics {
    #[serde(default = "defaults::metrics_poll_interval")]
    pub poll_interval: u64,

    #[serde(default = "defaults::metrics_poll_retry")]
    pub poll_retry: u64,

    #[serde(default = "defaults::metrics_poll_http_status_healthy_above")]
    pub poll_http_status_healthy_above: u16,

    #[serde(default = "defaults::metrics_poll_http_status_healthy_below")]
    pub poll_http_status_healthy_below: u16,

    #[serde(default = "defaults::metrics_poll_delay_dead")]
    pub poll_delay_dead: u64,

    #[serde(default = "defaults::metrics_poll_delay_sick")]
    pub poll_delay_sick: u64,

    #[serde(default = "defaults::metrics_push_delay_dead")]
    pub push_delay_dead: u64,

    #[serde(default = "defaults::metrics_push_system_cpu_sick_above")]
    pub push_system_cpu_sick_above: f32,

    #[serde(default = "defaults::metrics_push_system_ram_sick_above")]
    pub push_system_ram_sick_above: f32,
}

#[derive(Deserialize)]
pub struct ConfigNotify {
    pub email: Option<ConfigNotifyEmail>,
    pub twilio: Option<ConfigNotifyTwilio>,
    pub slack: Option<ConfigNotifySlack>,
    pub xmpp: Option<ConfigNotifyXMPP>,
}

#[derive(Deserialize)]
pub struct ConfigPlugins {
    pub rabbitmq: Option<ConfigPluginsRabbitMQ>,
}

#[derive(Deserialize)]
pub struct ConfigPluginsRabbitMQ {
    pub api_url: SerdeUrl,
    pub auth_username: String,
    pub auth_password: String,
    pub virtualhost: String,
    pub queue_ready_healthy_below: u32,
    pub queue_nack_healthy_below: u32,
}

#[derive(Deserialize)]
pub struct ConfigNotifyEmail {
    pub to: String,
    pub from: String,

    #[serde(default = "defaults::notify_email_smtp_host")]
    pub smtp_host: String,

    #[serde(default = "defaults::notify_email_smtp_port")]
    pub smtp_port: u16,

    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,

    #[serde(default = "defaults::notify_email_smtp_encrypt")]
    pub smtp_encrypt: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyTwilio {
    pub to: String,
    pub from: String,
    pub account_sid: String,
    pub auth_token: String,
}

#[derive(Deserialize)]
pub struct ConfigNotifySlack {
    pub hook_url: SerdeUrl,
}

#[derive(Deserialize)]
pub struct ConfigNotifyXMPP {
    pub to: String,
    pub from: String,
    pub xmpp_password: String,

    #[serde(default = "defaults::notify_xmpp_xmpp_encrypt")]
    pub xmpp_encrypt: bool,
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
    pub mode: Mode,
    pub replicas: Option<Vec<String>>,
    pub http_body_healthy_match: Option<Regex>,
    pub rabbitmq_queue: Option<String>,
}
