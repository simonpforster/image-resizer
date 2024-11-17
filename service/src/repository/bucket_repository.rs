use crate::client::bucket_client::bucket_request;
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageNotFoundError;
use crate::repository::{ImageItem, ImageRepository};
use image::ImageFormat;
use log::{error, warn};
use std::time::Instant;

pub struct BucketRepository {}

impl ImageRepository for BucketRepository {
    /// Request the image from the bucket and bundle into an `ImageItem`.
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
            warn!("Defaulting to Jpeg format for {path}");
            ImageFormat::Jpeg
        });

        let bytes: Vec<u8> = bucket_request(path).await.map_err(|_| {
            error!("Could not decode image at {path}");
            ImageNotFoundError {
                path: path.to_string(),
            }
        })?;
        Ok(ImageItem {
            time: Instant::now(),
            format,
            image: bytes,
        })
    }
}
