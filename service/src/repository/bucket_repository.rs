use crate::client::bucket_client::bucket_request;
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageNotFoundError;
use crate::repository::ImageRepository;
use tracing::{error, instrument};

#[derive(Debug)]
pub struct BucketRepository {}

impl ImageRepository for BucketRepository {
    /// Request the image from the bucket and bundle into an `ImageItem`.
    #[instrument]
    async fn read_image(&self, path: &str) -> Result<Vec<u8>, ErrorResponse> {
        bucket_request(path).await.map_err(|_| {
            error!("Could not decode image at {path}");
            ImageNotFoundError {}
        })
    }
}
