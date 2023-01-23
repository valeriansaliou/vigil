// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

use super::config::ConfigNotifyReminderBackoffFunction;

pub fn server_log_level() -> String {
    "error".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn server_workers() -> usize {
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
    10
}

pub fn metrics_poll_delay_sick() -> u64 {
    5
}

pub fn metrics_poll_parallelism() -> u16 {
    4
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

pub fn metrics_script_interval() -> u64 {
    300
}

pub fn script_parallelism() -> u16 {
    2
}

pub fn metrics_local_delay_dead() -> u64 {
    40
}

pub fn notify_startup_notification() -> bool {
    true
}

pub fn notify_reminder_backoff_function() -> ConfigNotifyReminderBackoffFunction {
    ConfigNotifyReminderBackoffFunction::None
}

pub fn notify_reminder_backoff_limit() -> u16 {
    3
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

pub fn notify_slack_mention_channel() -> bool {
    false
}

pub fn notify_generic_reminders_only() -> bool {
    false
}

pub fn probe_service_node_reveal_replica_name() -> bool {
    false
}
