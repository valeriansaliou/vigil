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
use crate::prober::mode::Mode;

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
    pub workers: usize,

    pub manager_token: String,
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

    #[serde(default = "defaults::metrics_poll_parallelism")]
    pub poll_parallelism: u16,

    #[serde(default = "defaults::metrics_push_delay_dead")]
    pub push_delay_dead: u64,

    #[serde(default = "defaults::metrics_push_system_cpu_sick_above")]
    pub push_system_cpu_sick_above: f32,

    #[serde(default = "defaults::metrics_push_system_ram_sick_above")]
    pub push_system_ram_sick_above: f32,

    #[serde(default = "defaults::metrics_script_interval")]
    pub script_interval: u64,

    #[serde(default = "defaults::script_parallelism")]
    pub script_parallelism: u16,

    #[serde(default = "defaults::metrics_local_delay_dead")]
    pub local_delay_dead: u64,
}

#[derive(Deserialize)]
pub struct ConfigNotify {
    #[serde(default = "defaults::notify_startup_notification")]
    pub startup_notification: bool,

    pub reminder_interval: Option<u64>,

    #[serde(default = "defaults::notify_reminder_backoff_function")]
    pub reminder_backoff_function: ConfigNotifyReminderBackoffFunction,

    #[serde(default = "defaults::notify_reminder_backoff_limit")]
    pub reminder_backoff_limit: u16,

    pub email: Option<ConfigNotifyEmail>,
    pub twilio: Option<ConfigNotifyTwilio>,
    pub slack: Option<ConfigNotifySlack>,
    pub zulip: Option<ConfigNotifyZulip>,
    pub telegram: Option<ConfigNotifyTelegram>,
    pub pushover: Option<ConfigNotifyPushover>,
    pub gotify: Option<ConfigNotifyGotify>,
    pub xmpp: Option<ConfigNotifyXMPP>,
    pub matrix: Option<ConfigNotifyMatrix>,
    pub webex: Option<ConfigNotifyWebEx>,
    pub webhook: Option<ConfigNotifyWebHook>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ConfigNotifyReminderBackoffFunction {
    #[serde(rename = "none")]
    None = 0,

    #[serde(rename = "linear")]
    Linear = 1,

    #[serde(rename = "square")]
    Square = 2,

    #[serde(rename = "cubic")]
    Cubic = 3,
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
    pub queue_ready_dead_above: u32,
    pub queue_nack_dead_above: u32,
    pub queue_loaded_retry_delay: Option<u64>,
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

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyTwilio {
    pub to: Vec<String>,
    pub service_sid: String,
    pub account_sid: String,
    pub auth_token: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifySlack {
    pub hook_url: SerdeUrl,

    #[serde(default = "defaults::notify_slack_mention_channel")]
    pub mention_channel: bool,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyZulip {
    pub bot_email: String,
    pub bot_api_key: String,
    pub channel: String,
    pub api_url: SerdeUrl,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyTelegram {
    pub bot_token: String,
    pub chat_id: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyPushover {
    pub app_token: String,
    pub user_keys: Vec<String>,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyGotify {
    pub app_url: SerdeUrl,
    pub app_token: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyXMPP {
    pub to: String,
    pub from: String,
    pub xmpp_password: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyMatrix {
    pub homeserver_url: SerdeUrl,
    pub access_token: String,
    pub room_id: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyWebEx {
    pub endpoint_url: SerdeUrl,
    pub token: String,
    pub room_id: String,

    #[serde(default = "defaults::notify_generic_reminders_only")]
    pub reminders_only: bool,
}

#[derive(Deserialize)]
pub struct ConfigNotifyWebHook {
    pub hook_url: SerdeUrl,
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
    pub scripts: Option<Vec<String>>,

    #[serde(default)]
    #[serde(with = "http_serde::header_map")]
    pub http_headers: http::HeaderMap,

    pub http_method: Option<ConfigProbeServiceNodeHTTPMethod>,
    pub http_body: Option<String>,
    pub http_body_healthy_match: Option<Regex>,

    #[serde(default = "defaults::probe_service_node_reveal_replica_name")]
    pub reveal_replica_name: bool,

    pub rabbitmq_queue: Option<String>,
    pub rabbitmq_queue_nack_healthy_below: Option<u32>,
    pub rabbitmq_queue_nack_dead_above: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigProbeServiceNodeHTTPMethod {
    #[serde(rename = "HEAD")]
    Head,

    #[serde(rename = "GET")]
    Get,

    #[serde(rename = "POST")]
    Post,

    #[serde(rename = "PUT")]
    Put,

    #[serde(rename = "PATCH")]
    Patch,
}
