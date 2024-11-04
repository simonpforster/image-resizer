use std::io::{BufReader, Cursor, Read};
use std::error;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::time::Instant;
use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use fast_image_resize::FilterType;
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{debug, error, info, warn};
use crate::dimension::{decode, Dimension};
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::*;
use crate::server_timing::ServerTiming;
use crate::server_timing::timing::Timing;

const PATH: &str = "/mnt";

const IMAGE_HEADER_NAME: &str = "content-type";
const CACHE_CONTROL_HEADER_NAME: &str = "cache-control";
const CACHE_CONTROL_HEADER_VALUE: &str = "max-age=31536000";
const IMAGE_HEADER_ROOT: &str = "image";
const SERVER_TIMING_HEADER_NAME: &str = "Server-Timing";

pub type ResultResponse = Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn error::Error + Send + Sync>>;
pub type InternalResponse = Result<ImageData, ErrorResponse>;

pub struct ImageData {
    body: BoxBody<Bytes, hyper::Error>,
    server_timing: ServerTiming,
    format_extension: String,
    content_length: u64,
}


pub fn process(path: &str) -> InternalResponse {
    let process_timer: Instant = Instant::now();

    let decoding_timer = Instant::now();
    let format: ImageFormat = ImageFormat::from_path(path).unwrap();
    debug!("Image format found.");


    let file: File = File::open(String::from(PATH) + &path).map_err(|_| {
        error!("Could not find image at {path}");
        ImageNotFoundError { path: path.to_string() }
    })?;
    let content_length = file.metadata().map_err(|_| {
        error!("Could not retrieve file metadata {path}");
        ImageNotFoundError { path: path.to_string() }
    })?.size();

    let mut reader = BufReader::new(file);
    let mut buf: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buf).expect("TODO: panic message");
    debug!("Found image at {path}");
    let decoding_timing: Timing = Timing::new("dec", decoding_timer.elapsed(), None);


    let encoding_timer = Instant::now();
    let format_extension: String = get_format_extension(format);
    let body: BoxBody<Bytes, hyper::Error> = bytes_to_stream(buf);
    let encoding_timing: Timing = Timing::new("enc", encoding_timer.elapsed(), None);


    let server_timing: ServerTiming = ServerTiming::new([decoding_timing, encoding_timing].to_vec());
    info!("Success simple {}: {path}", process_timer.elapsed().as_millis());
    Ok(ImageData {
        body,
        server_timing,
        format_extension,
        content_length,
    })
}

const OPTS: ResizeOptions = ResizeOptions {
    algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
    cropping: SrcCropping::FitIntoDestination((0f64, 0f64)),
    mul_div_alpha: true,
};

pub fn process_resize(path: &str, query: &str) -> InternalResponse {
    let process_timer: Instant = Instant::now();

    let decoding_timer = Instant::now();
    debug!("Processing query parameters");
    let dimension: Dimension = decode(query)?;
    debug!("Dimensions parsed");

    let (image, format) = read_image(path)?;
    let decoding_timing: Timing = Timing::new("dec", decoding_timer.elapsed(), None);

    let mut new_image: DynamicImage;
    let mut resizer: Resizer = Resizer::new();

    let resizing_timer = Instant::now();
    match dimension {
        Width(new_width) => {
            if new_width < image.width() {
                let new_height_f = (new_width * image.height()) as f64 / image.width() as f64;
                let new_height = new_height_f.round() as u32;
                let new_aspect_ratio = new_width as f64 / new_height as f64;
                info!("calculated new_dimensions are {new_width} / {new_height} = {new_aspect_ratio}");
                new_image = DynamicImage::new(
                    new_width,
                    new_height,
                    image.color(),
                );
                let _ = resizer.resize(
                    &image,
                    &mut new_image,
                    &OPTS,
                );
            } else {
                new_image = image;
            }
        }
        Height(new_height) => {
            if new_height < image.height() {
                let new_width = (new_height * image.width()) / image.height();
                new_image = DynamicImage::new(
                    new_width,
                    new_height,
                    image.color(),
                );
                let _ = resizer.resize(
                    &image,
                    &mut new_image,
                    &OPTS,
                );
            } else {
                new_image = image;
            }
        }
    };
    let resizing_timing: Timing = Timing::new("res", resizing_timer.elapsed(), None);

    debug!("Image resized, writing image to buffer");

    let encoding_timer = Instant::now();
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    new_image.write_to(&mut cursor, format).map_err(|_| {
        error!("Could not write the image for {path}");
        ImageWriteError { path: path.to_string() }
    })?;
    debug!("Image was written for {path}");

    let format_extension: String = get_format_extension(format);
    let content_length: u64 = bytes.len() as u64;
    let body: BoxBody<Bytes, hyper::Error> = bytes_to_stream(bytes);
    let encoding_timing: Timing = Timing::new("enc", encoding_timer.elapsed(), None);

    let server_timing: ServerTiming = ServerTiming::new([decoding_timing, resizing_timing, encoding_timing].to_vec());

    info!("Success resize {}: {path}?{query}", process_timer.elapsed().as_millis());
    Ok(ImageData {
        body,
        server_timing,
        format_extension,
        content_length,
    })
}

fn read_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    debug!("Open image for path: {path}");
    let reader = ImageReader::open(String::from(PATH) + &path).map_err(|_| {
        error!("Could not find image at {path}");
        ImageNotFoundError { path: path.to_string() }
    })?;
    debug!("Found image at {path}");

    let format: ImageFormat = reader.format().unwrap_or_else(|| {
        warn!("Defaulting to Jpeg format for {path}");
        ImageFormat::Jpeg
    });

    let image = reader.decode().map_err(|_| {
        error!("Could not decode image at {path}");
        ImageDecodeError { path: path.to_string() }
    })?;
    debug!("Image decoded at {path}");
    Ok((image, format))
}

fn get_format_extension(image_format: ImageFormat) -> String {
    let format_extension: String = image_format.extensions_str().to_owned().iter().next().map(|ext| "/".to_owned() + ext).unwrap_or_else(|| "".to_owned());
    debug!("Image format found {format_extension}");
    format_extension
}

fn bytes_to_stream(bytes: Vec<u8>) -> BoxBody<Bytes, hyper::Error> {
    let chunked = stream::iter(bytes).chunks(8192).map(|x| {
        Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x)))
    });
    BoxBody::new(StreamBody::new(chunked))
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn transform(response: InternalResponse) -> ResultResponse {
    match response {
        Ok(ImageData {
               body,
               server_timing,
               format_extension,
               content_length
           }) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(IMAGE_HEADER_NAME, IMAGE_HEADER_ROOT.to_owned() + &*format_extension)
                .header(SERVER_TIMING_HEADER_NAME, server_timing.to_string())
                .header(CACHE_CONTROL_HEADER_NAME, CACHE_CONTROL_HEADER_VALUE)
                .header("content-length", content_length)
                .body(body)?)
        }
        Err(e) => Ok(e.handle()?),
    }
}