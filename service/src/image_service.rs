use std::time::Instant;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use image::{DynamicImage, EncodableLayout, ImageFormat};
use log::{debug, error, warn};
use crate::bucket_client::bucket_request;
use crate::CACHE;
use crate::cache::ImageCacheItem;
use crate::dimension::Dimension;
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::{ImageDecodeError, ImageNotFoundError};

const RESIZE_OPTS: ResizeOptions = ResizeOptions {
    algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
    cropping: SrcCropping::FitIntoDestination((0.5, 0.5)),
    mul_div_alpha: true,
};

/// Attempts:
///     1. Memory Cache
///     2. Bucket (HTTP/2)
pub async fn read_image(path: &str) -> Result<(DynamicImage, ImageFormat), ErrorResponse> {
    let read_lock = CACHE.read().await;
    let maybe_image_cached_item = read_lock.read_image(path).map(|item| { item.clone() });
    drop(read_lock);

    let image_cache_item = match maybe_image_cached_item {
        Some(item) => item,
        None => {
            let new_image_cache_item = get_image_from_bucket(path).await?;

            let new_path: String = path.to_string();
            let cache_image = new_image_cache_item.clone();
            tokio::task::spawn(async move { CACHE.write().await.write_image(&new_path, cache_image); });

            new_image_cache_item
        }
    };

    debug!("Image decoded at {path}");
    Ok((image_cache_item.image, image_cache_item.format))
}

/// Resize an image based on a provided `Dimension`.
pub fn resize_image(dimension: Dimension, src_image: DynamicImage) -> DynamicImage {
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
        &RESIZE_OPTS,
    );
    dst_image
}

/// Request the image from the bucket and bundle into an `ImageCacheItem`
async fn get_image_from_bucket(path: &str) -> Result<ImageCacheItem, ErrorResponse> {
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
    Ok(ImageCacheItem { time: Instant::now(), format, image })
}