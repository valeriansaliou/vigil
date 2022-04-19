// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::time::SystemTime;

use chrono::offset::Utc;
use chrono::DateTime;

pub struct ConfigLogger;

impl log::Log for ConfigLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        if self.enabled(record.metadata()) {
            println!(
                "{} ({}) - {}",
                datetime.format("%Y-%m-%d %T"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

impl ConfigLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_max_level(level);
        log::set_boxed_logger(Box::new(ConfigLogger))
    }
}
