use std::str::FromStr;
use log4rs::{Config, Handle};
use log::{LevelFilter, SetLoggerError};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Logger, Root};
use crate::logging::encoder::MyEncoder;

mod log_message;
mod encoder;

pub fn logger_setup() -> Result<Handle, SetLoggerError> {
    let level: LevelFilter = LevelFilter::from_str("debug").unwrap();

    let stdout: ConsoleAppender = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(MyEncoder::new()))
        .build();

    let log_conf: log4rs::Config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::backend::db", level))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(log_conf)
}
