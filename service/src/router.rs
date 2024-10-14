use std::error;

use http_body_util::{BodyExt, combinators::BoxBody, Full};
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::Bytes;
use log::info;
use crate::service::process;

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>,  Box<dyn error::Error + Send + Sync>> {
    info!("Incoming {} request at {}", req.method().as_str(), req.uri());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/private/status") => {
            let mut ok = Response::new(full("OK"));
            *ok.status_mut() = StatusCode::OK;
            Ok(ok)
        }
        (&Method::GET, _) => {
            // we'll be back
            process(req).await
        }
        _ => {
            let mut not_found = Response::new(full("Endpoint not found"));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
