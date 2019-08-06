// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

pub fn server_log_level() -> String {
    "error".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn server_workers() -> u16 {
    4
}

pub fn assets_path() -> PathBuf {
    PathBuf::from("./res/assets/")
}

pub fn branding_page_title() -> String {
    "Status Page".to_string()
}

pub fn metrics_poll_interval() -> u64 {
    120
}

pub fn metrics_poll_retry() -> u64 {
    2
}

pub fn metrics_poll_http_status_healthy_above() -> u16 {
    200
}

pub fn metrics_poll_http_status_healthy_below() -> u16 {
    400
}

pub fn metrics_poll_delay_dead() -> u64 {
    30
}

pub fn metrics_poll_delay_sick() -> u64 {
    10
}

pub fn metrics_push_delay_dead() -> u64 {
    20
}

pub fn metrics_push_system_cpu_sick_above() -> f32 {
    0.99
}

pub fn metrics_push_system_ram_sick_above() -> f32 {
    0.99
}

pub fn notify_email_smtp_host() -> String {
    "localhost".to_string()
}

pub fn notify_email_smtp_port() -> u16 {
    587
}

pub fn notify_email_smtp_encrypt() -> bool {
    true
}

pub fn notify_email_reminders_only() -> bool {
    false
}

pub fn notify_twilio_reminders_only() -> bool {
    false
}

pub fn notify_slack_mention_channel() -> bool {
    false
}

pub fn notify_slack_reminders_only() -> bool {
    false
}

pub fn notify_pushover_reminders_only() -> bool {
    false
}

pub fn notify_xmpp_reminders_only() -> bool {
    false
}
