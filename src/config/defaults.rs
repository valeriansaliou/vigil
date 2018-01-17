// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

pub fn server_log_level() -> String {
    "warn".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn assets_path() -> PathBuf {
    PathBuf::from("./res/assets/")
}

pub fn branding_page_title() -> String {
    "Status Page".to_string()
}

pub fn metrics_poll_interval() -> u16 {
    120
}

pub fn metrics_poll_retry() -> u16 {
    2
}

pub fn metrics_poll_http_status_healthy_above() -> u16 {
    200
}

pub fn metrics_poll_http_status_healthy_below() -> u16 {
    400
}

pub fn metrics_poll_delay_dead() -> u16 {
    30
}

pub fn metrics_poll_delay_sick() -> u16 {
    10
}

pub fn metrics_push_delay_dead() -> u16 {
    20
}
