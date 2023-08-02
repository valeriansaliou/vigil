// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use actix_files::NamedFile;
use actix_web::{get, web, web::Data, web::Json, HttpResponse};
use std::time::{Duration, SystemTime};
use tera::Tera;
use time;
use uuid::Uuid;

use super::announcements::{
    Announcement, DATE_NOW_FORMATTER as ANNOUNCEMENTS_DATE_NOW_FORMATTER,
    STORE as ANNOUNCEMENTS_STORE,
};
use super::context::{IndexContext, INDEX_CONFIG, INDEX_ENVIRONMENT};
use super::payload::{
    ManagerAnnouncementInsertRequestPayload, ManagerAnnouncementInsertResponsePayload,
    ManagerAnnouncementsResponsePayload, ManagerProberAlertsIgnoredResolveRequestPayload,
    ManagerProberAlertsIgnoredResolveResponsePayload, ManagerProberAlertsResponsePayload,
    ManagerProberAlertsResponsePayloadEntry, ReporterRequestPayload,
};
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
            announcements: &ANNOUNCEMENTS_STORE.read().unwrap().announcements,
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
async fn badge(kind: web::Path<String>) -> Option<NamedFile> {
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
async fn assets_fonts(path: web::Path<(String, String)>) -> Option<NamedFile> {
    let info = path.into_inner();
    NamedFile::open(APP_CONF.assets.path.join("fonts").join(info.0).join(info.1)).ok()
}

#[get("/assets/images/{folder}/{file}")]
async fn assets_images(
    path: web::Path<(String, String)>,
) -> Option<NamedFile> {
    let info = path.into_inner();
    NamedFile::open(APP_CONF.assets.path.join("images").join(info.0).join(info.1)).ok()
}

#[get("/assets/stylesheets/{file}")]
async fn assets_stylesheets(file: web::Path<String>) -> Option<NamedFile> {
    NamedFile::open(APP_CONF.assets.path.join("stylesheets").join(file.into_inner())).ok()
}

#[get("/assets/javascripts/{file}")]
async fn assets_javascripts(file: web::Path<String>) -> Option<NamedFile> {
    let file = file.into_inner();
    NamedFile::open(APP_CONF.assets.path.join("javascripts").join(file)).ok()
}

// Notice: reporter report route is managed in manager due to authentication needs
pub async fn reporter_report(
    path: web::Path<(String, String)>,
    data: Json<ReporterRequestPayload>,
) -> HttpResponse {
    let info = path.into_inner();
    let probe_id = info.0;
    let node_id: String = info.1;
    debug!("reporter report: {}:{}", probe_id, node_id);
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
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    let info = path.into_inner();
    debug!("reporter flush: {}:{}:{}", info.0, info.1, info.2);
    // Flush reports should come for 'push' and 'local' nodes only
    match handle_flush_report(&info.0, &info.1, &info.2) {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(HandleFlushError::WrongMode) => HttpResponse::PreconditionFailed().finish(),
        Err(HandleFlushError::NotFound) => HttpResponse::NotFound().finish(),
    }
}

// Notice: manager announcements route is managed in manager due to authentication needs
pub async fn manager_announcements() -> HttpResponse {
    // List all announcements in store
    HttpResponse::Ok().json(
        ANNOUNCEMENTS_STORE
            .read()
            .unwrap()
            .announcements
            .iter()
            .map(|announcement| ManagerAnnouncementsResponsePayload {
                id: announcement.id.to_owned(),
                title: announcement.title.to_owned(),
            })
            .collect::<Vec<ManagerAnnouncementsResponsePayload>>(),
    )
}

// Notice: manager announcement insert route is managed in manager due to authentication needs
pub async fn manager_announcement_insert(
    data: Json<ManagerAnnouncementInsertRequestPayload>,
) -> HttpResponse {
    // Validate data
    if data.title.len() > 0 && data.text.len() > 0 {
        // Generate unique identifier and insert in announcements
        let id = Uuid::new_v4().hyphenated().to_string();

        let mut store = ANNOUNCEMENTS_STORE.write().unwrap();

        store.announcements.push(Announcement {
            id: id.to_owned(),
            title: data.title.to_owned(),
            text: data.text.to_owned(),

            date: Some(
                time::OffsetDateTime::now_utc()
                    .format(&ANNOUNCEMENTS_DATE_NOW_FORMATTER)
                    .unwrap_or("?".to_string()),
            ),
        });

        HttpResponse::Ok().json(ManagerAnnouncementInsertResponsePayload { id: id })
    } else {
        // Announcement data is invalid
        HttpResponse::BadRequest().finish()
    }
}

// Notice: manager announcement retract route is managed in manager due to authentication needs
pub async fn manager_announcement_retract(
    announcement_id: web::Path<String>,
) -> HttpResponse {
    let announcement_id = announcement_id.into_inner();
    let mut store = ANNOUNCEMENTS_STORE.write().unwrap();

    // Find announcement index (if it exists)
    let announcement_index = store
        .announcements
        .iter()
        .position(|announcement| announcement.id == announcement_id);

    if let Some(announcement_index) = announcement_index {
        // Remove target announcement
        store.announcements.remove(announcement_index);

        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

// Notice: manager prober alerts route is managed in manager due to authentication needs
pub async fn manager_prober_alerts() -> HttpResponse {
    let mut alerts = ManagerProberAlertsResponsePayload::default();

    // Classify probes with a non-healthy status
    let probes = &PROBER_STORE.read().unwrap().states.probes;

    for (probe_id, probe) in probes.iter() {
        for (node_id, node) in probe.nodes.iter() {
            for (replica_id, replica) in node.replicas.iter() {
                // Replica is either sick or dead, append to alerts
                if replica.status == Status::Sick || replica.status == Status::Dead {
                    let alert_entry = ManagerProberAlertsResponsePayloadEntry {
                        probe: probe_id.to_owned(),
                        node: node_id.to_owned(),
                        replica: replica_id.to_owned(),
                    };

                    match replica.status {
                        Status::Sick => alerts.sick.push(alert_entry),
                        Status::Dead => alerts.dead.push(alert_entry),
                        _ => {}
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(alerts)
}

// Notice: manager prober alerts ignored resolve route is managed in manager due to authentication \
//   needs
pub async fn manager_prober_alerts_ignored_resolve() -> HttpResponse {
    let states = &PROBER_STORE.read().unwrap().states;

    // Calculate remaining ignore reminders seconds (if any set or if time is still left)
    let reminders_seconds = states
        .notifier
        .reminder_ignore_until
        .and_then(|reminder_ignore_until| {
            reminder_ignore_until.duration_since(SystemTime::now()).ok()
        })
        .map(|reminder_ignore_duration_since| reminder_ignore_duration_since.as_secs() as u16);

    HttpResponse::Ok().json(ManagerProberAlertsIgnoredResolveResponsePayload {
        reminders_seconds: reminders_seconds,
    })
}

// Notice: manager prober alerts ignored update route is managed in manager due to authentication \
//   needs
pub async fn manager_prober_alerts_ignored_update(
    data: Json<ManagerProberAlertsIgnoredResolveRequestPayload>,
) -> HttpResponse {
    let mut store = PROBER_STORE.write().unwrap();

    // Assign reminder ignore intil date (re-map from seconds to date time if set)
    store.states.notifier.reminder_ignore_until = data
        .reminders_seconds
        .map(|reminders_seconds| SystemTime::now() + Duration::from_secs(reminders_seconds as _));

    HttpResponse::Ok().finish()
}
