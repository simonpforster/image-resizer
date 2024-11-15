use std::io::{Cursor};
use std::error;
use std::time::Instant;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use futures_util::{stream, StreamExt};
use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Response, StatusCode};
use hyper::body::{Frame, Bytes};
use image::{DynamicImage, EncodableLayout, ImageFormat};
use log::{debug, error, warn};
use crate::{CACHE};
use crate::bucket_client::bucket_request;
use crate::cache::ImageCacheItem;
use crate::dimension::{decode, Dimension};
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::*;
use crate::server_timing::ServerTiming;
use crate::server_timing::timing::Timing;

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

const OPTS: ResizeOptions = ResizeOptions {
    algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
    cropping: SrcCropping::FitIntoDestination((0.5, 0.5)),
    mul_div_alpha: true,
};

pub async fn process_resize(path: &str, opt_query: Option<&str>) -> InternalResponse {
    let process_timer: Instant = Instant::now();

    let decoding_timer = Instant::now();
    debug!("Processing query parameters");
    let opt_dimension: Option<Dimension> = match opt_query {
        Some(query) => decode(query).ok(),
        None => None,
    };

    debug!("Dimensions parsed");
    let (image, format) = read_image(path).await?;
    let decoding_timing: Timing = Timing::new("dec", decoding_timer.elapsed(), None);

    let resizing_timer = Instant::now();

    let new_image: DynamicImage = match opt_dimension {
        Some(dimension) => resize_image(dimension, image),
        None => image,
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

    debug!("Success {} ms: {path}", process_timer.elapsed().as_millis());
    Ok(ImageData {
        body,
        server_timing,
        format_extension,
        content_length,
    })
}

async fn read_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    let read_lock = CACHE.read().await;
    let maybe_image_cached_item = read_lock.read_image(path).map(|d| d.clone());
    drop(read_lock);

    let image_cache_item = match maybe_image_cached_item {
        Some(item) => item,
        None => {
            let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
                warn!("Defaulting to Jpeg format for {path}");
                ImageFormat::Jpeg
            });

            let bytes = bucket_request(path).await.map_err(|_| {
                error!("Could not decode image at {path}");
                ImageNotFoundError { path: path.to_string() }
            })?;

            let image = image::load_from_memory_with_format(bytes.as_bytes(), format).map_err(|_| {
                error!("Could not decode image at {path}");
                ImageDecodeError { path: path.to_string() }
            })?;
            let new_image_cache_item = ImageCacheItem { time: Instant::now(), format, image };
            let new_path: String = path.to_string();
            let borr = new_image_cache_item.clone();

            // write to cache in seperate task so that it doesn't block the response to the client
            tokio::task::spawn(async move { CACHE.write().await.write_image(&new_path, borr); });

            new_image_cache_item
        }
    };

    debug!("Image decoded at {path}");
    Ok((image_cache_item.image, image_cache_item.format))
}

fn resize_image(dimension: Dimension, src_image: DynamicImage) -> DynamicImage {
    let mut dst_image: DynamicImage;
    let mut resizer: Resizer = Resizer::new();
    match dimension {
        Width(new_width) => {
            let new_height = ((new_width * src_image.height()) as f64 / src_image.width() as f64) as u32;
            dst_image = DynamicImage::new(
                new_width,
                new_height,
                src_image.color(),
            );
        }
        Height(new_height) => {
            let new_width = ((new_height * src_image.width()) as f64 / src_image.height() as f64) as u32;
            dst_image = DynamicImage::new(
                new_width,
                new_height,
                src_image.color(),
            );
        }
    };
    let _ = resizer.resize(
        &src_image,
        &mut dst_image,
        &OPTS,
    );
    dst_image
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