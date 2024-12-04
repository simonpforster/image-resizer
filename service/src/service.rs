pub(crate) use crate::domain::dimension::{decode, Dimension};
pub(crate) use crate::domain::error::ErrorResponse;
pub(crate) use crate::domain::error::ErrorResponse::*;
use crate::domain::server_timing::{timing::Timing, ServerTiming};
use crate::domain::{ExtensionProvider, ImageData};
use crate::image_service::{get_image, encode_image, resize_image};
use futures_util::{stream, StreamExt};
use http_body_util::combinators::BoxBody;
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame};
use image::DynamicImage;
use std::io::Cursor;
use std::time::Instant;
use tracing::instrument;

use tracing::{debug, error};

pub type InternalResponse = Result<ImageData, ErrorResponse>;

#[instrument]
pub async fn process_resize(path: &str, opt_query: Option<&str>) -> InternalResponse {
    let process_timer: Instant = Instant::now();

    let decoding_timer = Instant::now();
    debug!("Processing query parameters");
    let opt_dimension: Option<Dimension> = match opt_query {
        Some(query) => decode(query).ok(),
        None => None,
    };

    debug!("Dimensions parsed");
    let (image, format) = get_image(path).await?;
    let decoding_timing: Timing = Timing::new("dec", decoding_timer.elapsed(), None);

    let resizing_timer = Instant::now();

    let new_image: DynamicImage = match opt_dimension {
        Some(dimension) => resize_image(dimension, image),
        None => image,
    };

    let resizing_timing: Timing = Timing::new("res", resizing_timer.elapsed(), None);

    debug!("Image resized, writing image to buffer");

    let encoding_timer = Instant::now();
    let (body, content_length) = encode_image(new_image, format)?;
    let encoding_timing: Timing = Timing::new("enc", encoding_timer.elapsed(), None);


    let format_extension: String = format.get_format_extension();
    let server_timing: ServerTiming =
        ServerTiming::new([decoding_timing, resizing_timing, encoding_timing].to_vec());

    debug!("Success {} ms: {path}", process_timer.elapsed().as_millis());
    Ok(ImageData {
        body,
        server_timing,
        format_extension,
        content_length,
    })
}
