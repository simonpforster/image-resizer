use crate::domain::error::ErrorResponse::{
    ImageDecodeError, ImageNotFoundError, ImageNotFoundInCacheError, ImageWriteError,
};
use crate::router::full;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ErrorResponse
where
    ErrorResponse: error::Error,
{
    ImageNotFoundError { path: String },
    ImageDecodeError { path: String },
    ImageWriteError { path: String },
    ImageNotFoundInCacheError { path: String },
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageNotFoundError { path } => write!(f, "Image not found for: {path}"),
            ImageNotFoundInCacheError { path } => write!(f, "Image not found in cache for: {path}"),
            ImageDecodeError { path } => write!(f, "Image could not be decoded for: {path}"),
            ImageWriteError { path } => write!(f, "Image could not be written for: {path}"),
        }
    }
}

impl ErrorResponse {
    pub fn handle(&self) -> hyper::http::Result<Response<BoxBody<Bytes, hyper::Error>>> {
        match self {
            ImageNotFoundError { path } => error_response(
                StatusCode::NOT_FOUND,
                format!("Image not found for: {path}"),
            ),
            ImageNotFoundInCacheError { path } => error_response(
                StatusCode::NOT_FOUND,
                format!("Image not found for: {path}"),
            ),
            ImageDecodeError { path } => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Image could not be decoded for: {path}"),
            ),
            ImageWriteError { path } => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Image could not be written for: {path}"),
            ),
        }
    }
}

impl error::Error for ErrorResponse {}

fn error_response(
    status_code: StatusCode,
    message: String,
) -> hyper::http::Result<Response<BoxBody<Bytes, hyper::Error>>> {
    Response::builder().status(status_code).body(full(message))
}
