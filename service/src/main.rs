use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use log::{info, LevelFilter, SetLoggerError};
use log4rs::{Config, Handle};
use tokio::net::TcpListener;
use tokio::sync::{RwLock};
use tokio::time;
use crate::cache::Cache;
use crate::logging::logger_setup;
use crate::router::router;

mod service;
mod router;
mod error;
mod dimension;
mod server_timing;
mod cache;
mod bucket_client;
mod logging;

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
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            CACHE.write().await.cull();
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

