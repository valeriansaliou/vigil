// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::PathBuf;
use time;

use rocket::response::NamedFile;
use rocket_contrib::Template;

use super::context::{self, INDEX_CONFIG, IndexContext};
use APP_CONF;

lazy_static! {
    static ref FILE_PUBLIC_ROBOTS: PathBuf = APP_CONF.assets.path.join("./public/robots.txt");
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &IndexContext {
        // TODO: non static data (source from shared context)
        status: context::Status::Healthy,
        refreshed_at: time::strftime("%H:%M:%S UTC%z", &time::now()).unwrap_or("".to_string()),
        groups: Vec::new(),
        config: &*INDEX_CONFIG,
    })
}

#[get("/robots.txt")]
fn robots() -> Option<NamedFile> {
    NamedFile::open(&*FILE_PUBLIC_ROBOTS).ok()
}

#[get("/badge/<size>")]
fn badge(size: u16) -> String {
    format!("TODO: size={}", size)
}

#[get("/assets/fonts/<file..>")]
fn assets_fonts(file: PathBuf) -> Option<NamedFile> {
    // TODO: expire + cache header

    NamedFile::open(APP_CONF.assets.path.join("./fonts").join(file)).ok()
}

#[get("/assets/images/<file..>")]
fn assets_images(file: PathBuf) -> Option<NamedFile> {
    // TODO: expire + cache header

    NamedFile::open(APP_CONF.assets.path.join("./images").join(file)).ok()
}

#[get("/assets/stylesheets/<file..>")]
fn assets_stylesheets(file: PathBuf) -> Option<NamedFile> {
    // TODO: expire + cache header

    NamedFile::open(APP_CONF.assets.path.join("./stylesheets").join(file)).ok()
}
