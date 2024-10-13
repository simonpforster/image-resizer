use std::io::Cursor;
use std::error;
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Request, Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::imageops::FilterType;
use image::ImageReader;
use log::{error, info};
use rand::Rng;

pub async fn process(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>> {
    let path = String::from(req.uri().path());
    //let parameters = req.uri().query();
    info!("Open image for path: {path}");

    let mut rng = rand::thread_rng();

    let reader = match ImageReader::open(String::from(".") + &path) {
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


    let new_image = image.resize(rng.gen_range(3000..image.width()), rng.gen_range(3000..image.height()), FilterType::Nearest);
    info!("Image resized, writing image to buffer");
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    match new_image.write_to(&mut cursor, image::ImageFormat::Jpeg) {
        Ok(val) => {
            info!("Image was written for {path}");
            val
        }
        Err(_) => {
            error!("Could write the image for {path}");
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
    let response = Response::builder().status(StatusCode::OK).body(body).unwrap();
    Ok(response)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}