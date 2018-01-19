// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::PathBuf;

use rocket::http::Status;
use rocket::response::{NamedFile, Failure};
use rocket_contrib::{Template, Json};

use super::context::{INDEX_CONFIG, IndexContext};
use super::asset_file::AssetFile;
use prober::manager::{STORE as PROBER_STORE};
use prober::report::{handle as handle_report, HandleError};
use APP_CONF;

#[derive(Deserialize)]
pub struct ReporterData {
    replica: String,
    interval: u64,
    load: ReporterDataLoad,
}

#[derive(Deserialize)]
pub struct ReporterDataLoad {
    cpu: f32,
    ram: f32,
}

#[get("/")]
fn index() -> Template {
    // Notice acquire lock in a block to release it ASAP (ie. before template renders)
    let context = {
        IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            config: &*INDEX_CONFIG,
        }
    };

    Template::render("index", &context)
}

#[post("/reporter/<probe_id>/<node_id>", data = "<data>", format = "application/json")]
fn reporter(probe_id: String, node_id: String, data: Json<ReporterData>) -> Result<(), Failure> {
    match handle_report(
        &probe_id, &node_id, &data.replica, data.interval, data.load.cpu, data.load.ram
    ) {
        Ok(_) => Ok(()),
        Err(HandleError::InvalidLoad) => Err(Failure(Status::BadRequest)),
        Err(HandleError::NotFound) => Err(Failure(Status::NotFound)),
    }
}

#[get("/robots.txt")]
fn robots() -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./public/robots.txt")).ok()
}

#[get("/badge/<kind>")]
fn badge(kind: String) -> Option<NamedFile> {
    // Notice acquire lock in a block to release it ASAP (ie. before OS access to file)
    let status = {
        &PROBER_STORE.read().unwrap().states.status.as_str()
    };

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
