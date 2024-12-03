use crate::domain::ImageData;
use crate::service::InternalResponse;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::error;
use tracing::instrument;

const IMAGE_HEADER_NAME: &str = "content-type";
const CACHE_CONTROL_HEADER_NAME: &str = "cache-control";
const CACHE_CONTROL_HEADER_VALUE: &str = "max-age=31536000";
const IMAGE_HEADER_ROOT: &str = "image";
const SERVER_TIMING_HEADER_NAME: &str = "Server-Timing";

pub type ResultResponse =
    Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;

#[instrument]
pub fn transform(response: InternalResponse) -> ResultResponse {
    match response {
        Ok(ImageData {
            body,
            server_timing,
            format_extension,
            content_length,
        }) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(
                IMAGE_HEADER_NAME,
                IMAGE_HEADER_ROOT.to_owned() + &*format_extension,
            )
            .header(SERVER_TIMING_HEADER_NAME, server_timing.to_string())
            .header(CACHE_CONTROL_HEADER_NAME, CACHE_CONTROL_HEADER_VALUE)
            .header("content-length", content_length)
            .body(body)?),
        Err(e) => Ok(e.handle()?),
    }
}
