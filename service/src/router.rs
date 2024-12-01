use std::error;

use crate::response_handler::transform;
use crate::service::process_resize;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request, Response, StatusCode};
use tracing::debug;

pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>> {
    let request_id = req
        .headers()
        .get("traceparent")
        .map(|d| d.to_str().unwrap_or("none"))
        .unwrap_or("none");
    debug!("request_id: {}", request_id);
    match (req.method(), req.uri().path(), req.uri().query()) {
        (&Method::GET, "/private/status", None) => {
            let mut ok = Response::new(full("OK"));
            *ok.status_mut() = StatusCode::OK;
            Ok(ok)
        }
        (&Method::GET, path, query_params) => transform(process_resize(path, query_params).await),
        _ => {
            let mut not_found = Response::new(full("Endpoint not found"));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// TODO remove this
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
