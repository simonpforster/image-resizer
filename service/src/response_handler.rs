use crate::domain::ImageData;
use crate::service::InternalResponse;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::error;
use std::fmt::Display;
use hyper::http::HeaderValue;
use opentelemetry::global::ObjectSafeSpan;
use opentelemetry::trace::FutureExt;
use reqwest::header::HeaderMap;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::observability::propagators::HyperHeaderInjector;

const IMAGE_HEADER_NAME: &str = "content-type";
const CACHE_CONTROL_HEADER_NAME: &str = "cache-control";
const CACHE_CONTROL_HEADER_VALUE: &str = "max-age=31536000";
const IMAGE_HEADER_ROOT: &str = "image";
const SERVER_TIMING_HEADER_NAME: &str = "server-timing";
const TRACERESPONSE_HEADER: &str = "traceresponse";
const CONTENT_LENGTH_HEADER_NAME: &str = "content-length";


pub type ResultResponse =
Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;

#[instrument(skip(response))]
pub fn transform(response: InternalResponse) -> ResultResponse {
    match response {
        Ok(ImageData {
               body,
               server_timing,
               format_extension,
               content_length,
           }) => {
            let mut response = Response::new(body);
            let header_map = response.headers_mut();
            {
                header_map.insert(IMAGE_HEADER_NAME, HeaderValue::from_str(&(IMAGE_HEADER_ROOT.to_owned() + &*format_extension))?);
                header_map.insert(SERVER_TIMING_HEADER_NAME, HeaderValue::from_str(&format!("{}", server_timing))?);
                header_map.insert(CACHE_CONTROL_HEADER_NAME, HeaderValue::from_str(&CACHE_CONTROL_HEADER_VALUE)?);
                header_map.insert(CONTENT_LENGTH_HEADER_NAME, HeaderValue::from_str(&content_length.to_string())?);
                let _ = opentelemetry::global::get_text_map_propagator(|propagator| {
                    propagator.inject(&mut HyperHeaderInjector(header_map))
                });
                let current_span = tracing::Span::current();
                if let Some(span_context) = current_span.context().span_context() {
                    let trace_id = span_context.trace_id().to_string();
                    let span_id = span_context.span_id().to_string();
                    let trace_flags = span_context.trace_flags().to_string();

                    let traceresponse_value = format!(
                        "00-{}-{}-{}",
                        trace_id,
                        span_id,
                        trace_flags
                    );

                    header_map.insert(
                        TRACERESPONSE_HEADER,
                        HeaderValue::from_str(&traceresponse_value)?
                    );
                }
            }

            Ok(response)
        }
        Err(e) => Ok(e.handle()?),
    }
}
