use std::error;

use http_body_util::{BodyExt, combinators::BoxBody, Full};
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::Bytes;
use log::{debug, info};
use crate::service::{process, process_resize, transform};

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>,  Box<dyn error::Error + Send + Sync>> {
    //debug!("Incoming {} request at {}", req.method().as_str(), req.uri());
    match (req.method(), req.uri().path(), req.uri().query()) {
        (&Method::GET, "/private/status", None) => {
            let mut ok = Response::new(full("OK"));
            *ok.status_mut() = StatusCode::OK;
            Ok(ok)
        }
        (&Method::GET, path, Some(query)) => {
            transform(process_resize(path, query))
        }
        (&Method::GET, path, None) => {
            transform(process(path))
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
