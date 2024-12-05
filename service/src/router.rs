use std::error;

use crate::response_handler::transform;
use crate::service::process_resize;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request, Response, StatusCode};
use opentelemetry::Context;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::observability::propagators::HyperHeaderExtractor;

#[instrument(skip(req))]
pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>> {
    let context: Context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HyperHeaderExtractor(&req.headers().clone()))
    });
    tracing::Span::current().set_parent(context);
    match (req.method(), req.uri().path(), req.uri().query()) {
        (&Method::GET, "/private/status", None) =>
            Ok(Response::new(full("OK"))),
        (&Method::GET, "/", None) => {
            let no_content = Response::builder().status(StatusCode::NO_CONTENT).body(full(Bytes::new()))?;
            Ok(no_content)
        }
        (&Method::GET, path, query_params) => {
            let resp = transform(process_resize(path, query_params).await);
            resp
        }
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
