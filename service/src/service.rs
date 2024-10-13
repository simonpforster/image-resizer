use std::io::Cursor;
use std::error;
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Request, Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use log::{error, info, warn};
use rand::Rng;

const IMAGE_HEADER_NAME: &str = "content-type";
const IMAGE_HEADER_ROOT: &str = "image";

pub async fn process(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>> {
    let path = String::from(req.uri().path());
    //let parameters = req.uri().query();
    info!("Open image for path: {path}");

    let mut rng = rand::thread_rng();

    let reader = match ImageReader::open(String::from("/mnt/gcsfuse") + &path) {
        Ok(reader) => {
            info!("Found image at {path}");
            reader
        }
        Err(_) => {
            error!("Could not find image at {path}");
            return Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full("Image not found for: path"))
                    .unwrap());
        }
    };

    let format = reader.format().unwrap_or_else(|| {
        warn!("Defaulting to Jpeg format for {path}");
        ImageFormat::Jpeg
    });

    let image = match reader.decode() {
        Ok(image) => {
            info!("Image decoded at {path}");
            image
        }
        Err(_) => {
            error!("Could not decode image at {path}");
            return Ok(
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full("Image could not be decoded for: path"))
                    .unwrap());
        }
    };


    let new_image = image.resize(rng.gen_range(20..image.width()), rng.gen_range(20..image.height()), FilterType::Nearest);
    info!("Image resized, writing image to buffer");
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    match new_image.write_to(&mut cursor, format) {
        Ok(val) => {
            info!("Image was written for {path}");
            val
        }
        Err(_) => {
            error!("Could not write the image for {path}");
            return Ok(
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full("Image could not be written for: path"))
                    .unwrap());
        }
    };


    let chunked = stream::iter(bytes).chunks(8192).map(|x| {
        Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x)))
    });
    let body: BoxBody<Bytes, hyper::Error> = BoxBody::new(StreamBody::new(chunked));
    info!("Respond Success!");

    let image_extension = format.extensions_str().to_owned().iter().next().map(|ext| "/".to_owned() + ext).unwrap_or_else(|| "".to_owned());

    let response =
        Response::builder()
            .status(StatusCode::OK)
            .header(IMAGE_HEADER_NAME, IMAGE_HEADER_ROOT.to_owned() + &*image_extension)
            .body(body)
            .unwrap();
    Ok(response)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}