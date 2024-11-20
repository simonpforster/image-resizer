use std::io::{Cursor};
use std::time::Instant;
use crate::domain::dimension::Dimension;
use crate::domain::dimension::Dimension::{Height, Width};
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageDecodeError;
use crate::repository::{ImageItem, ImageRepository};
use crate::{BUCKET_REPOSITORY, CACHE_REPOSITORY};
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use image::{DynamicImage, EncodableLayout, ImageFormat, ImageReader};
use log::{debug, error, info};

const RESIZE_OPTS: ResizeOptions = ResizeOptions {
    algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
    cropping: SrcCropping::FitIntoDestination((0.5, 0.5)),
    mul_div_alpha: true,
};

///
/// Get image from provided path, it attempts:
///     1. Memory Cache
///     2. Bucket (HTTP/2)
pub async fn read_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    let maybe_image_cached_item = CACHE_REPOSITORY.read_image(path).await.ok();

    let image_cache_item: ImageItem = match maybe_image_cached_item {
        Some(item) => item,
        None => {
            let new_image_cache_item = BUCKET_REPOSITORY.read_image(path).await?;

            let new_path: String = path.to_string();
            let cache_image = new_image_cache_item.clone();
            tokio::task::spawn(
                async move { CACHE_REPOSITORY.write_image(new_path, cache_image).await },
            );

            new_image_cache_item
        }
    };

    let timer = Instant::now();
    let cursor = Cursor::new(image_cache_item.image.as_bytes());
    let image: DynamicImage = ImageReader::with_format(cursor, image_cache_item.format).decode().map_err(|_| {
        error!("Could not decode image at {path}");
        ImageDecodeError {
            path: path.to_string(),
        }
    })?;
    info!("loading image took: {} ms for {}", timer.elapsed().as_millis(), path);
    debug!("Image decoded at {path}");
    Ok((image, image_cache_item.format))
}

/// Resize an image based on a provided `Dimension`.
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
