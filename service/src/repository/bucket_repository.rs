use std::time::Instant;
use image::{DynamicImage, EncodableLayout, ImageFormat};
use log::{error, warn};
use crate::client::bucket_client::bucket_request;
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::{ImageDecodeError, ImageNotFoundError};
use crate::repository::{ImageItem, ImageRepository};

pub struct BucketRepository {}

impl ImageRepository for BucketRepository {
    /// Request the image from the bucket and bundle into an `ImageItem`.
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
            warn!("Defaulting to Jpeg format for {path}");
            ImageFormat::Jpeg
        });

        let bytes = bucket_request(path).await.map_err(|_| {
            error!("Could not decode image at {path}");
            ImageNotFoundError { path: path.to_string() }
        })?;

        let image: DynamicImage = image::load_from_memory_with_format(bytes.as_bytes(), format).map_err(|_| {
            error!("Could not decode image at {path}");
            ImageDecodeError { path: path.to_string() }
        })?;
        Ok(ImageItem { time: Instant::now(), format, image })
    }
}

