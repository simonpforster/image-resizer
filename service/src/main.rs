use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use log::{info, LevelFilter, SetLoggerError};
use log4rs::{Config, Handle};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::Instant;
use crate::cache::Cache;
use crate::router::router;

mod service;
mod router;
mod error;
mod dimension;
mod server_timing;
mod cache;

lazy_static! {
    static ref CACHE: RwLock<Cache> = RwLock::new(Cache::default());
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = logger_setup();

    // Cache culler task
    tokio::task::spawn(async {
        let mut interval = time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            let cull_timer = Instant::now();
            CACHE.write().await.cull();
            info!("In write lock queue + use for {} ms.", cull_timer.elapsed().as_millis());
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("Attempting to start server at {addr}");
    let listener = TcpListener::bind(addr).await?;
    info!("Server started at {addr}");

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http2::Builder::new(TokioExecutor)
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
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build();

    let log_conf: log4rs::Config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::backend::db", level))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(log_conf)
}
