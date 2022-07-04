// Vigil
//
// Microservices Status Page
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::sync::RwLock;

lazy_static! {
    pub static ref STORE: Arc<RwLock<Store>> = Arc::new(RwLock::new(Store {
        announcements: Vec::new(),
    }));
}

pub struct Store {
    pub announcements: Vec<Announcement>,
}

#[derive(Serialize)]
pub struct Announcement {
    title: String,
    text: String,
    date: Option<String>,
}
