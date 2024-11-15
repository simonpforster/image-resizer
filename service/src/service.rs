use std::io::{Cursor};
use std::time::Instant;
use futures_util::{stream, StreamExt};
use http_body_util::StreamBody;
use http_body_util::combinators::BoxBody;
use hyper::body::{Frame, Bytes};
use image::{DynamicImage};
use log::{debug, error};
use crate::domain::dimension::{decode, Dimension};
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::*;
use crate::domain::{ExtensionProvider, ImageData};
use crate::image_service::{read_image, resize_image};
use crate::domain::server_timing::ServerTiming;
use crate::domain::server_timing::timing::Timing;

pub type InternalResponse = Result<ImageData, ErrorResponse>;

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

    let format_extension: String = format.get_format_extension();
    let content_length: u64 = bytes.len() as u64;
    let chunked = stream::iter(bytes).chunks(8192).map(|x| {
        Ok::<Frame<Bytes>, hyper::Error>(Frame::data(Bytes::from(x)))
    });
    let body: BoxBody<Bytes, hyper::Error> = BoxBody::new(StreamBody::new(chunked));
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

