use std::net::SocketAddr;
use std::process::Command;
use std::str::FromStr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::{info, LevelFilter, SetLoggerError};
use log4rs::{Config, Handle};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::net::TcpListener;

use crate::router::router;

mod service;
mod router;
mod error;
mod dimension;
mod server_timing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = logger_setup();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("Attempting to start server at {addr}");
    let listener = TcpListener::bind(addr).await?;
    info!("Server started at {addr}");

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {

            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(router))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn logger_setup() -> Result<Handle, SetLoggerError> {
    let level: LevelFilter = LevelFilter::from_str("info").unwrap();

    let stdout: ConsoleAppender = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build();

    let log_conf: log4rs::Config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::backend::db", level))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(log_conf)
}
