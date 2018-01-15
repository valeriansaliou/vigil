// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#![feature(plugin)]
#![plugin(interpolate_idents, rocket_codegen)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate rocket;
extern crate rocket_contrib;

mod config;
mod prober;
mod responder;

use std::thread;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use clap::{App, Arg};
use log::LogLevelFilter;

use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use prober::manager::run as run_prober;
use responder::manager::run as run_responder;

struct AppArgs {
    config: String,
}

pub static THREAD_NAME_PROBER: &'static str = "vigil-prober";
pub static THREAD_NAME_RESPONDER: &'static str = "vigil-responder";

macro_rules! gen_spawn_managed {
    ($name:expr, $method:ident, $thread_name:ident, $managed_fn:ident) => (
        fn $method() {
            debug!("spawn managed thread: {}", $name);

            let worker = thread::Builder::new()
                .name($thread_name.to_string())
                .spawn($managed_fn);

            // Block on worker thread (join it)
            let has_error = if let Ok(worker_thread) = worker {
                worker_thread.join().is_err()
            } else {
                true
            };

            // Worker thread crashed?
            if has_error == true {
                error!("managed thread crashed ({}), setting it up again", $name);

                // Prevents thread start loop floods
                thread::sleep(Duration::from_secs(1));

                $method();
            }
        }
    )
}

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
}

gen_spawn_managed!("prober", spawn_prober, THREAD_NAME_PROBER, run_prober);
gen_spawn_managed!("responder", spawn_responder, THREAD_NAME_RESPONDER, run_responder);

fn make_app_args() -> AppArgs {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Path to configuration file")
                .default_value("./config.cfg")
                .takes_value(true),
        )
        .get_matches();

    // Generate owned app arguments
    AppArgs { config: String::from(matches.value_of("config").expect("invalid config value")) }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    APP_CONF.deref();
}

fn main() {
    let _logger = ConfigLogger::init(
        LogLevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Spawn probes (background thread)
    thread::spawn(spawn_prober);

    // Spawn Web responder (foreground thread)
    spawn_responder();

    error!("could not start");
}
