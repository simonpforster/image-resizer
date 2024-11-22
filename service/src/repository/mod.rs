use crate::domain::error::ErrorResponse;
use std::time::Instant;

pub(crate) mod bucket_repository;
pub(crate) mod cache_repository;
pub(crate) mod volume_repository;

#[derive(Debug, Clone)]
pub struct ImageItem {
    pub time: Instant,
    pub image: Vec<u8>,
}

pub trait ImageRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse>;
}
