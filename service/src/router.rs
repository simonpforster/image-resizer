use std::error;

use crate::response_handler::transform;
use crate::service::process_resize;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::Bytes;
use hyper::{HeaderMap, Method, Request, Response, StatusCode};
use opentelemetry::Context;
use opentelemetry::propagation::{Extractor, Injector};
use tracing::{info, instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

struct HyperHeaderExtractor<'a>(&'a HeaderMap);

impl<'a> Extractor for HyperHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)
            .and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys()
            .map(|k| k.as_str())
            .collect()
    }
}

// struct HyperHeaderInjector<'a>(&'a mut HeaderMap);
//
// impl<'a> Injector for HyperHeaderInjector<'a> {
//     fn set(&mut self, key: &str, value: String) {
//         if let Ok(header_name) = key.parse() {
//             self.0.insert(header_name, value.parse().unwrap());
//         }
//     }
// }

#[instrument]
pub async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>> {

    let context: Context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HyperHeaderExtractor(&req.headers().clone()))
    });

    info!("{:?}", context);
    tracing::Span::current().set_parent(context);
    match (req.method(), req.uri().path(), req.uri().query()) {
        (&Method::GET, "/private/status", None) => {
            let mut ok = Response::new(full("OK"));
            *ok.status_mut() = StatusCode::OK;
            Ok(ok)
        }
        (&Method::GET, path, query_params) => {
            let resp = transform(process_resize(path, query_params).await);
            resp
        },
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
