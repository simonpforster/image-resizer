use crate::domain::dimension::Dimension;
use crate::domain::dimension::Dimension::{Height, Width};
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageDecodeError;
use crate::domain::{format_from_path, ExtensionProvider};
use crate::repository::ImageRepository;
use crate::{BUCKET_REPOSITORY, VOLUME_REPOSITORY};
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::{BufReader, Cursor};
use std::time::Instant;
use tokio_util::bytes;
use tracing::{debug, error, info, instrument, Instrument};
use futures_util::{stream, StreamExt};
use hyper::body::{Bytes, Frame};
use http_body_util::combinators::{BoxBody};
use http_body_util::StreamBody;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::service::ImageWriteError;

const RESIZE_OPTS: ResizeOptions = ResizeOptions {
    algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
    cropping: SrcCropping::FitIntoDestination((0.5, 0.5)),
    mul_div_alpha: true,
};

/// Get image from provided path, it attempts:
///     1. Volume cache
///     2. Bucket (HTTP/2)
#[instrument]
pub async fn get_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    let image_bytes: Vec<u8> = match VOLUME_REPOSITORY.read_image(path).await.ok() {
        Some(item) => item,
        None => {
            let bucket_item = BUCKET_REPOSITORY.read_image(path).await?;
            VOLUME_REPOSITORY.write_image(&path, &bucket_item).await?;
            bucket_item
        }
    };

    let image = decode_image(image_bytes, format_from_path(path))?;
    debug!("Image decoded at {path}");
    Ok((image, format_from_path(&path)))
}

/// Resize an image based on a provided `Dimension`.
/// TODO make output Vec<u8>
#[instrument(skip(src_image))]
pub fn resize_image(dimension: Dimension, src_image: DynamicImage) -> DynamicImage {
    let mut dst_image: DynamicImage;
    let mut resizer: Resizer = Resizer::new();
    match dimension {
        Width(new_width) => {
            let new_height =
                ((new_width * src_image.height()) as f64 / src_image.width() as f64) as u32;
            dst_image = DynamicImage::new(new_width, new_height, src_image.color());
        }
        Height(new_height) => {
            let new_width =
                ((new_height * src_image.width()) as f64 / src_image.height() as f64) as u32;
            dst_image = DynamicImage::new(new_width, new_height, src_image.color());
        }
    };
    let _ = resizer.resize(&src_image, &mut dst_image, &RESIZE_OPTS);
    dst_image
}

/// Decode bytes to `DynamicImage`.
#[instrument(skip(image_bytes))]
pub fn decode_image(image_bytes: Vec<u8>, format: ImageFormat) -> Result<DynamicImage, ErrorResponse> {
    let cursor = Cursor::new(image_bytes);
    let mut reader = BufReader::new(cursor);
    ImageReader::with_format(&mut reader, format)
        .decode()
        .map_err(|_| {
            ImageDecodeError {}
        })
}

/// Take a dynamic image and write it as `Bytes`.
#[instrument(skip(image))]
pub fn encode_image(image: DynamicImage, format: ImageFormat) -> Result<(BoxBody<Bytes, hyper::Error>, u64), ErrorResponse> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    image.write_to(&mut cursor, format).map_err(|_| {
        ImageWriteError {}
    })?;
    let content_length: u64 = bytes.len() as u64;
    let chunked = stream::iter(bytes)
        .chunks(8192)
        .map(|x| Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x))));
    let body: BoxBody<Bytes, hyper::Error> = BoxBody::new(StreamBody::new(chunked));
    Ok((body, content_length))
}
