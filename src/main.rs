// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket;
extern crate base64;
extern crate indexmap;
extern crate native_tls;
extern crate openssl_probe;
extern crate regex;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
extern crate time;
extern crate toml;
extern crate url;
extern crate url_serde;

#[cfg(feature = "notifier-email")]
extern crate lettre;
#[cfg(feature = "notifier-email")]
extern crate lettre_email;

#[cfg(feature = "notifier-xmpp")]
extern crate libstrophe;

mod aggregator;
mod config;
mod notifier;
mod prober;
mod responder;

use std::ops::Deref;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use log::LevelFilter;

use crate::aggregator::manager::run as run_aggregator;
use crate::config::config::Config;
use crate::config::logger::ConfigLogger;
use crate::config::reader::ConfigReader;
use crate::prober::manager::{initialize_store as initialize_store_prober, run as run_prober};
use crate::responder::manager::run as run_responder;

struct AppArgs {
    config: String,
}

pub static THREAD_NAME_PROBER: &'static str = "vigil-prober";
pub static THREAD_NAME_AGGREGATOR: &'static str = "vigil-aggregator";
pub static THREAD_NAME_RESPONDER: &'static str = "vigil-responder";

macro_rules! gen_spawn_managed {
    ($name:expr, $method:ident, $thread_name:ident, $managed_fn:ident) => {
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
    };
}

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
}

gen_spawn_managed!("prober", spawn_prober, THREAD_NAME_PROBER, run_prober);
gen_spawn_managed!(
    "aggregator",
    spawn_aggregator,
    THREAD_NAME_AGGREGATOR,
    run_aggregator
);
gen_spawn_managed!(
    "responder",
    spawn_responder,
    THREAD_NAME_RESPONDER,
    run_responder
);

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
    AppArgs {
        config: String::from(matches.value_of("config").expect("invalid config value")),
    }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    let (_, _) = (APP_ARGS.deref(), APP_CONF.deref());

    // Ensure assets path exists
    assert_eq!(
        APP_CONF.assets.path.exists(),
        true,
        "assets directory not found: {:?}",
        APP_CONF.assets.path
    );
}

fn main() {
    // Ensure OpenSSL root chain is found on current environment
    openssl_probe::init_ssl_cert_env_vars();

    // Initialize shared logger
    let _logger = ConfigLogger::init(
        LevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Initialize prober store
    initialize_store_prober();

    // Spawn probes (background thread)
    thread::spawn(spawn_prober);

    // Spawn aggregator (background thread)
    thread::spawn(spawn_aggregator);

    // Spawn Web responder (foreground thread)
    spawn_responder();

    error!("could not start");
}
