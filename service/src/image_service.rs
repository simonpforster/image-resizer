use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use image::{DynamicImage, ImageFormat};
use log::debug;
use crate::dimension::Dimension;
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::repository::bucket_repository::BucketRepository;
use crate::repository::cache_repository::CacheRepository;
use crate::repository::ImageRepository;

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
    let maybe_image_cached_item = (CacheRepository {}).read_image(path).await.ok();

    let image_cache_item = match maybe_image_cached_item {
        Some(item) => item,
        None => {
            let new_image_cache_item = (BucketRepository {}).read_image(path).await?;

            let new_path: String = path.to_string();
            let cache_image = new_image_cache_item.clone();
            tokio::task::spawn(async move { (CacheRepository {}).write_image(new_path, cache_image) });

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

