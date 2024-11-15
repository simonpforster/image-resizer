use std::time::Instant;
use image::{DynamicImage, ImageFormat};
use crate::error::ErrorResponse;

pub(crate) mod cache_repository;
pub(crate) mod bucket_repository;

#[derive(Debug, Clone)]
pub struct ImageItem {
    pub time: Instant,
    pub format: ImageFormat,
    pub image: DynamicImage,
}

pub trait ImageRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse>;
}
