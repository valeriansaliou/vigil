// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_files::NamedFile;
use actix_web::{get, web, web::Data, web::Json, HttpResponse};
use tera::Tera;

use super::context::{IndexContext, INDEX_CONFIG, INDEX_ENVIRONMENT};
use super::payload::ReporterPayload;
use crate::prober::manager::{run_dispatch_plugins, STORE as PROBER_STORE};
use crate::prober::report::{
    handle_flush as handle_flush_report, handle_health as handle_health_report,
    handle_load as handle_load_report, HandleFlushError, HandleHealthError, HandleLoadError,
};
use crate::prober::status::Status;
use crate::APP_CONF;

#[get("/")]
async fn index(tera: Data<Tera>) -> HttpResponse {
    // Notice acquire lock in a block to release it ASAP (ie. before template renders)
    let context = {
        IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            environment: &*INDEX_ENVIRONMENT,
            config: &*INDEX_CONFIG,
        }
    };
    let render = tera.render(
        "index.tera",
        &tera::Context::from_serialize(context).unwrap(),
    );
    if let Ok(s) = render {
        HttpResponse::Ok().content_type("text/html").body(s)
    } else {
        HttpResponse::InternalServerError().body(format!("Template Error {:?}", render))
    }
}

#[get("/robots.txt")]
async fn robots() -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("public").join("robots.txt")).ok()
}

#[get("/status/text")]
async fn status_text() -> &'static str {
    &PROBER_STORE.read().unwrap().states.status.as_str()
}

#[get("/badge/{kind}")]
async fn badge(web::Path(kind): web::Path<String>) -> Option<NamedFile> {
    // Notice acquire lock in a block to release it ASAP (ie. before OS access to file)
    let status = { &PROBER_STORE.read().unwrap().states.status.as_str() };

    if let Ok(badge_file) = NamedFile::open(
        APP_CONF
            .assets
            .path
            .join("images")
            .join("badges")
            .join(format!("{}-{}-default.svg", kind, status)),
    ) {
        // Return badge file without 'Last-Modified' HTTP header, which would otherwise hold the \
        //   date the actual badge image file was last modified, which is not what we want there, \
        //   as it would make browsers believe they can use a previous cache they hold, on a \
        //   badge image that can be for a different status.
        Some(
            badge_file
                .disable_content_disposition()
                .use_last_modified(false),
        )
    } else {
        None
    }
}

#[get("/assets/fonts/{folder}/{file}")]
async fn assets_fonts(web::Path((folder, file)): web::Path<(String, String)>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("fonts").join(folder).join(file)).ok()
}

#[get("/assets/images/{folder}/{file}")]
async fn assets_images(
    web::Path((folder, file)): web::Path<(String, String)>,
) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("images").join(folder).join(file)).ok()
}

#[get("/assets/stylesheets/{file}")]
async fn assets_stylesheets(web::Path(file): web::Path<String>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("stylesheets").join(file)).ok()
}

#[get("/assets/javascripts/{file}")]
async fn assets_javascripts(web::Path(file): web::Path<String>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("javascripts").join(file)).ok()
}

pub async fn start_planned_maintenance(web::Path(probe_id): web::Path<String>) -> HttpResponse {
    let store = &mut PROBER_STORE.write().unwrap();
    if let Some(ref mut probe) = store.states.probes.get_mut(&probe_id) {
        probe.status = Status::Maintenance;
        info!("Starting planned maintenance for probe: {:?}. Notifications will be suppressed for this probe.", probe_id);
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::BadRequest().body(format!("Could not find service named '{}'", probe_id))
    }
}

pub async fn stop_planned_maintenance(web::Path(probe_id): web::Path<String>) -> HttpResponse {
    let store = &mut PROBER_STORE.write().unwrap();
    if let Some(ref mut probe) = store.states.probes.get_mut(&probe_id) {
        if probe.status == Status::Maintenance {
            probe.status = Status::Healthy;
            info!("Stopping planned maintenance for probe: {:?}", probe_id);
            HttpResponse::Ok().finish()
        } else {
            HttpResponse::BadRequest().body(format!(
                "ERROR: Service is not currently set to status maintenance: {:?}",
                probe_id
            ))
        }
    } else {
        HttpResponse::BadRequest().body(format!("Could not find service named '{}'", probe_id))
    }
}

// Notice: reporter report route is managed in manager due to authentication needs
pub async fn reporter_report(
    web::Path((probe_id, node_id)): web::Path<(String, String)>,
    data: Json<ReporterPayload>,
) -> HttpResponse {
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

                HttpResponse::Ok().finish()
            }
            Err(HandleLoadError::InvalidLoad) => HttpResponse::BadRequest().finish(),
            Err(HandleLoadError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
            Err(HandleLoadError::NotFound) => HttpResponse::NotFound().finish(),
        }
    } else if let Some(ref health) = data.health {
        // Health reports should come for 'local' nodes only
        match handle_health_report(&probe_id, &node_id, &data.replica, data.interval, health) {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(HandleHealthError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
            Err(HandleHealthError::NotFound) => HttpResponse::NotFound().finish(),
        }
    } else {
        // Report contents is invalid
        HttpResponse::BadRequest().finish()
    }
}

// Notice: reporter flush route is managed in manager due to authentication needs
pub async fn reporter_flush(
    web::Path((probe_id, node_id, replica_id)): web::Path<(String, String, String)>,
) -> HttpResponse {
    // Flush reports should come for 'push' and 'local' nodes only
    match handle_flush_report(&probe_id, &node_id, &replica_id) {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(HandleFlushError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
        Err(HandleFlushError::NotFound) => HttpResponse::NotFound().finish(),
    }
}
