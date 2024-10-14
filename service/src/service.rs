use std::io::Cursor;
use std::error;
use std::fmt::{Debug, Display, Formatter};
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{error, info, warn};
use rand::{Rng};
use crate::service::ErrorResponse::{ImageNotFoundError, ImageDecodeError, ImageWriteError};

const PATH: &str = "/mnt/gcsfuse";

const IMAGE_HEADER_NAME: &str = "content-type";
const IMAGE_HEADER_ROOT: &str = "image";

pub type ResultResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;
pub type InternalResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, ErrorResponse>;

#[derive(Debug)]
pub enum ErrorResponse
where
    ErrorResponse: error::Error,
{
    ImageNotFoundError { path: String },
    ImageDecodeError { path: String },
    ImageWriteError { path: String },
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageNotFoundError { path } => write!(f, "Image not found for: {path}"),
            ImageDecodeError { path } => write!(f, "Image could not be decoded for: {path}"),
            ImageWriteError { path } => write!(f, "Image could not be written for: {path}"),
        }
    }
}

impl ErrorResponse {
    fn handle(&self) -> hyper::http::Result<Response<BoxBody<Bytes, hyper::Error>>> {
        match self {
            ImageNotFoundError { path } => {
                error_response(
                    StatusCode::NOT_FOUND,
                    format!("Image not found for: {path}"),
                )
            }
            ImageDecodeError { path } => {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Image could not be decoded for: {path}"),
                )
            }
            ImageWriteError { path } => {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Image could not be written for: {path}"),
                )
            }
        }
    }
}

impl error::Error for ErrorResponse {}

pub fn process(path: &str) -> InternalResponse {
    info!("Open image for path: {path}");

    let mut rng = rand::thread_rng();

    let reader = ImageReader::open(String::from(PATH) + &path).map_err(|_| {
        error!("Could not find image at {path}");
        ImageNotFoundError { path: path.to_string() }
    })?;
    info!("Found image at {path}");

    let format: ImageFormat = reader.format().unwrap_or_else(|| {
        warn!("Defaulting to Jpeg format for {path}");
        ImageFormat::Jpeg
    });
    let format_extension: String = format.extensions_str().to_owned().iter().next().map(|ext| "/".to_owned() + ext).unwrap_or_else(|| "".to_owned());
    info!("Image format found {format_extension}");

    let image = reader.decode().map_err(|_| {
        error!("Could not decode image at {path}");
        ImageDecodeError { path: path.to_string() }
    })?;
    info!("Image decoded at {path}");

    let new_image: DynamicImage = image.resize(rng.gen_range(20..image.width()), rng.gen_range(20..image.height()), FilterType::Nearest);
    info!("Image resized, writing image to buffer");
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    new_image.write_to(&mut cursor, format).map_err(|_| {
        error!("Could not write the image for {path}");
        ImageWriteError { path: path.to_string() }
    })?;
    info!("Image was written for {path}");


    let chunked = stream::iter(bytes).chunks(8192).map(|x| {
        Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x)))
    });
    let body: BoxBody<Bytes, hyper::Error> = BoxBody::new(StreamBody::new(chunked));
    info!("Respond Success!");


    let response =
        Response::builder()
            .status(StatusCode::OK)
            .header(IMAGE_HEADER_NAME, IMAGE_HEADER_ROOT.to_owned() + &*format_extension)
            .body(body)
            .unwrap();
    Ok(response)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn transform(response: InternalResponse) -> ResultResponse {
    match response {
        Ok(resp) => Ok(resp),
        Err(e) => Ok(e.handle()?)
    }
}

pub fn error_response(status_code: StatusCode, message: String) -> hyper::http::Result<Response<BoxBody<Bytes, hyper::Error>>> {
    Response::builder()
        .status(status_code)
        .body(full(message))
}