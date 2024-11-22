use std::net::SocketAddr;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use log::info;
use tokio::net::TcpListener;
use crate::logging::logger_setup;
use crate::repository::bucket_repository::BucketRepository;
use crate::repository::cache_repository::CacheRepository;
use crate::repository::volume_repository::VolumeRepository;
use crate::router::router;

mod service;
mod router;
mod logging;
mod response_handler;
mod image_service;
mod client;
mod repository;
mod domain;

lazy_static! {
    static ref CACHE_REPOSITORY: CacheRepository = CacheRepository {};
    static ref VOLUME_REPOSITORY: VolumeRepository = VolumeRepository {};
    static ref BUCKET_REPOSITORY: BucketRepository = BucketRepository {};
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

