// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde_derive::Deserialize;
use std::path::PathBuf;

use rocket::http::Status;
use rocket::response::NamedFile;
use rocket::{get, post};
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

use super::asset_file::AssetFile;
use super::context::{IndexContext, INDEX_CONFIG, INDEX_ENVIRONMENT};
use super::reporter_guard::ReporterGuard;
use crate::prober::manager::{run_dispatch_plugins, STORE as PROBER_STORE};
use crate::prober::report::{
    handle_health as handle_health_report, handle_load as handle_load_report, HandleHealthError,
    HandleLoadError,
};
use crate::prober::status::Status as HealthStatus;
use crate::APP_CONF;

#[derive(Deserialize)]
pub struct ReporterData {
    replica: String,
    interval: u64,
    health: Option<HealthStatus>,
    load: Option<ReporterDataLoad>,
}

#[derive(Deserialize)]
pub struct ReporterDataLoad {
    cpu: f32,
    ram: f32,
}

#[get("/")]
pub fn index() -> Template {
    // Notice acquire lock in a block to release it ASAP (ie. before template renders)
    let context = {
        IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            environment: &*INDEX_ENVIRONMENT,
            config: &*INDEX_CONFIG,
        }
    };

    Template::render("index", &context)
}

#[post(
    "/reporter/<probe_id>/<node_id>",
    data = "<data>",
    format = "application/json"
)]
pub fn reporter(
    _auth: ReporterGuard,
    probe_id: String,
    node_id: String,
    data: Json<ReporterData>,
) -> Result<(), Status> {
    // Route report to handler (depending on its contents)
    if let Some(ref load) = data.load {
        // Load reports should come for 'push' nodes only
        match handle_load_report(
            &probe_id,
            &node_id,
            &data.replica,
            data.interval,
            load.cpu,
            load.ram,
        ) {
            Ok(forward) => {
                // Trigger a plugins check
                run_dispatch_plugins(&probe_id, &node_id, forward);

                Ok(())
            }
            Err(HandleLoadError::InvalidLoad) => Err(Status::BadRequest),
            Err(HandleLoadError::WrongMode) => Err(Status::PreconditionFailed),
            Err(HandleLoadError::NotFound) => Err(Status::NotFound),
        }
    } else if let Some(ref health) = data.health {
        // Health reports should come for 'local' nodes only
        match handle_health_report(&probe_id, &node_id, &data.replica, data.interval, health) {
            Ok(_) => Ok(()),
            Err(HandleHealthError::WrongMode) => Err(Status::PreconditionFailed),
            Err(HandleHealthError::NotFound) => Err(Status::NotFound),
        }
    } else {
        // Report contents is invalid
        Err(Status::BadRequest)
    }
}

#[get("/robots.txt")]
pub fn robots() -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./public/robots.txt")).ok()
}

#[get("/status/text")]
pub fn status_text() -> &'static str {
    &PROBER_STORE.read().unwrap().states.status.as_str()
}

#[get("/badge/<kind>")]
pub fn badge(kind: String) -> Option<NamedFile> {
    // Notice acquire lock in a block to release it ASAP (ie. before OS access to file)
    let status = { &PROBER_STORE.read().unwrap().states.status.as_str() };

    NamedFile::open(
        APP_CONF
            .assets
            .path
            .join(format!("./images/badges/{}-{}-default.svg", kind, status)),
    )
    .ok()
}

#[get("/assets/fonts/<file..>")]
pub fn assets_fonts(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./fonts").join(file)).ok()
}

#[get("/assets/images/<file..>")]
pub fn assets_images(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./images").join(file)).ok()
}

#[get("/assets/stylesheets/<file..>")]
pub fn assets_stylesheets(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./stylesheets").join(file)).ok()
}

#[get("/assets/javascripts/<file..>")]
pub fn assets_javascripts(file: PathBuf) -> Option<AssetFile> {
    AssetFile::open(APP_CONF.assets.path.join("./javascripts").join(file)).ok()
}
