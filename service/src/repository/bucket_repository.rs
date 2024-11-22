use crate::client::bucket_client::bucket_request;
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageNotFoundError;
use crate::repository::{ImageItem, ImageRepository};
use image::ImageFormat;
use log::error;
use std::time::Instant;
use crate::domain::format_from_path;
use crate::service::ImageDecodeError;

pub struct BucketRepository {}

impl ImageRepository for BucketRepository {
    /// Request the image from the bucket and bundle into an `ImageItem`.
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let format: ImageFormat = format_from_path(path);

        let bytes: Vec<u8> = bucket_request(path).await.map_err(|_| {
            error!("Bucket could not decode image at {path}");
            ImageNotFoundError {
                path: path.to_string(),
            }
        })?;
        let image = image::load_from_memory_with_format(bytes.as_slice(), format).map_err(|_| {
            error!("Bucket could not decode image at {path}");
            ImageDecodeError {
                path: path.to_string(),
            }
        })?;

        Ok(ImageItem {
            time: Instant::now(),
            image,
        })
    }
}
