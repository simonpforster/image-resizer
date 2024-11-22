use crate::client::bucket_client::bucket_request;
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageNotFoundError;
use crate::repository::{ImageItem, ImageRepository};
use log::{error};
use std::time::Instant;

pub struct BucketRepository {}

impl ImageRepository for BucketRepository {
    /// Request the image from the bucket and bundle into an `ImageItem`.
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let bytes: Vec<u8> = bucket_request(path).await.map_err(|_| {
            error!("Could not decode image at {path}");
            ImageNotFoundError {
                path: path.to_string(),
            }
        })?;
        Ok(ImageItem {
            time: Instant::now(),
            image: bytes,
        })
    }
}
