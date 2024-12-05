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
    ImageNotFoundError {},
    ImageDecodeError {},
    ImageWriteError {},
    ImageNotFoundInCacheError {},
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageNotFoundError {} => write!(f, "Image not found."),
            ImageNotFoundInCacheError {} => write!(f, "Image not found in cache."),
            ImageDecodeError {} => write!(f, "Image could not be decoded."),
            ImageWriteError {} => write!(f, "Image could not be written."),
        }
    }
}

impl ErrorResponse {
    pub fn handle(&self) -> hyper::http::Result<Response<BoxBody<Bytes, hyper::Error>>> {
        match self {
            ImageNotFoundError {} => error_response(
                StatusCode::NOT_FOUND,
                format!("Image not found."),
            ),
            ImageNotFoundInCacheError {} => error_response(
                StatusCode::NOT_FOUND,
                format!("Image not found."),
            ),
            ImageDecodeError {} => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Image could not be decoded."),
            ),
            ImageWriteError {} => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Image could not be written."),
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
