// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::PathBuf;

use rocket::response::NamedFile;
use rocket_contrib::Template;

use super::context::{INDEX_CONFIG, IndexContext};
use super::asset_file::AssetFile;
use prober::manager::{STORE as PROBER_STORE};
use APP_CONF;

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        &IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            config: &*INDEX_CONFIG,
        },
    )
}

#[get("/robots.txt")]
fn robots() -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./public/robots.txt")).ok()
}

#[get("/badge/<kind>")]
fn badge(kind: String) -> Option<NamedFile> {
    let status = &PROBER_STORE.read().unwrap().states.status.as_str();

    NamedFile::open(APP_CONF.assets.path.join(
        format!("./images/badges/{}-{}-default.svg", kind, status)
    )).ok()
}

#[get("/assets/fonts/<file..>")]
fn assets_fonts(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./fonts").join(file)).ok()
}

#[get("/assets/images/<file..>")]
fn assets_images(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./images").join(file)).ok()
}

#[get("/assets/stylesheets/<file..>")]
fn assets_stylesheets(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./stylesheets").join(file)).ok()
}
