use crate::domain::ImageData;
use crate::service::InternalResponse;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::error;
use std::fmt::Display;
use hyper::http::HeaderValue;
use opentelemetry::trace::FutureExt;
use reqwest::header::HeaderMap;
use tracing::instrument;
use crate::observability::propagators::HyperHeaderInjector;

const IMAGE_HEADER_NAME: &str = "content-type";
const CACHE_CONTROL_HEADER_NAME: &str = "cache-control";
const CACHE_CONTROL_HEADER_VALUE: &str = "max-age=31536000";
const IMAGE_HEADER_ROOT: &str = "image";
const SERVER_TIMING_HEADER_NAME: &str = "Server-Timing";

pub type ResultResponse =
Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;

#[instrument(skip(response))]
pub fn transform(response: InternalResponse) -> ResultResponse {
    let mut header_map = HeaderMap::new();

    let _ = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject(&mut HyperHeaderInjector(&mut header_map))
    });
    match response {
        Ok(ImageData {
               body,
               server_timing,
               format_extension,
               content_length,
           }) => {
            header_map.insert(IMAGE_HEADER_NAME, HeaderValue::from_str(&(IMAGE_HEADER_ROOT.to_owned() + &*format_extension))?);
            header_map.insert(SERVER_TIMING_HEADER_NAME, HeaderValue::from_str(&format!("{}", server_timing))?);
            header_map.insert(CACHE_CONTROL_HEADER_NAME, HeaderValue::from_str(&CACHE_CONTROL_HEADER_VALUE)?);
            header_map.insert("content-length", HeaderValue::from_str(&content_length.to_string())?);
            let mut response = Response::new(body);
            let mut response_headers = response.headers_mut();
            response_headers = &mut header_map;
            Ok(response)
        }
        Err(e) => Ok(e.handle()?),
    }
}
