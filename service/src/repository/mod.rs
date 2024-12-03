use crate::domain::error::ErrorResponse;

pub(crate) mod bucket_repository;
pub(crate) mod volume_repository;

pub trait ImageRepository {
    async fn read_image(&self, path: &str) -> Result<Vec<u8>, ErrorResponse>;
}
