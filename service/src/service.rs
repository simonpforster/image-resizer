use std::cmp::min;
use std::collections::HashMap;
use std::io::Cursor;
use std::error;
use std::fs::exists;
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{error, info, warn};
use crate::dimension::{decode, Dimension};
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::*;

const PATH: &str = "./";
// const PATH: &str = "/mnt/gcsfuse";

const IMAGE_HEADER_NAME: &str = "content-type";
const IMAGE_HEADER_ROOT: &str = "image";

pub type ResultResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;
pub type InternalResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, ErrorResponse>;

pub fn process(path: &str) -> InternalResponse {

    let (image, format) = read_image(path)?;

    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    image.write_to(&mut cursor, format).map_err(|_| {
        error!("Could not write the image for {path}");
        ImageWriteError { path: path.to_string() }
    })?;
    info!("Image was written for {path}");

    let format_extension: String = get_format_extension(format);
    let body: BoxBody<Bytes, hyper::Error> = bytes_to_stream(bytes);

    let response =
        Response::builder()
            .status(StatusCode::OK)
            .header(IMAGE_HEADER_NAME, IMAGE_HEADER_ROOT.to_owned() + &*format_extension)
            .body(body)
            .unwrap();
    Ok(response)
}


pub fn process_resize(path: &str, query: &str) -> InternalResponse {

    info!("Processing query parameters");
    let dimension: Dimension = decode(query)?;
    info!("Dimensions parsed");

    let (image, format) = read_image(path)?;

    let new_image: DynamicImage = match dimension {
        Width(new_width) => {
            if new_width < image.width() {
                let new_height = (new_width * image.height()) / image.width() ;
                info!("Resizing WRT to width to {new_width}x{new_height}");
                image.resize(new_width, new_height, FilterType::Nearest)
            } else {
                image
            }
        }
        Height(new_height) => {
            if new_height < image.height() {
                let new_width = (new_height * image.width()) / image.height();
                info!("Resizing WRT to height to {new_width}x{new_height}");
                image.resize(new_width, new_height, FilterType::Nearest)
            } else {
                image
            }
        }
    };

    info!("Image resized, writing image to buffer");

    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    new_image.write_to(&mut cursor, format).map_err(|_| {
        error!("Could not write the image for {path}");
        ImageWriteError { path: path.to_string() }
    })?;
    info!("Image was written for {path}");

    let format_extension: String = get_format_extension(format);
    let body: BoxBody<Bytes, hyper::Error> = bytes_to_stream(bytes);

    let response =
        Response::builder()
            .status(StatusCode::OK)
            .header(IMAGE_HEADER_NAME, IMAGE_HEADER_ROOT.to_owned() + &*format_extension)
            .body(body)
            .unwrap();
    Ok(response)
}

fn read_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    info!("Open image for path: {path}");
    let reader = ImageReader::open(String::from(PATH) + &path).map_err(|_| {
        error!("Could not find image at {path}");
        ImageNotFoundError { path: path.to_string() }
    })?;
    info!("Found image at {path}");

    let format: ImageFormat = reader.format().unwrap_or_else(|| {
        warn!("Defaulting to Jpeg format for {path}");
        ImageFormat::Jpeg
    });

    let image = reader.decode().map_err(|_| {
        error!("Could not decode image at {path}");
        ImageDecodeError { path: path.to_string() }
    })?;
    info!("Image decoded at {path}");
    Ok((image, format))
}

fn get_format_extension(image_format: ImageFormat) -> String {
    let format_extension: String = image_format.extensions_str().to_owned().iter().next().map(|ext| "/".to_owned() + ext).unwrap_or_else(|| "".to_owned());
    info!("Image format found {format_extension}");
    format_extension
}

fn bytes_to_stream(bytes: Vec<u8>) -> BoxBody<Bytes, hyper::Error> {
    let chunked = stream::iter(bytes).chunks(8192).map(|x| {
        Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x)))
    });
    let body: BoxBody<Bytes, hyper::Error> = BoxBody::new(StreamBody::new(chunked));
    info!("Respond Success!");
    body
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn transform(response: InternalResponse) -> ResultResponse {
    match response {
        Ok(resp) => Ok(resp),
        Err(e) => Ok(e.handle()?),
    }
}