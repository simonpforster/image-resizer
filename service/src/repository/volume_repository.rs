use std::fs;
use std::time::Instant;
use image::ImageFormat;
use log::{error, warn};
use crate::repository::{ImageItem, ImageRepository};
use crate::service::{ErrorResponse, ImageNotFoundError, ImageWriteError};

pub struct VolumeRepository {}

const ROOT_PATH: &str = "/mnt/shared-cache/";

impl VolumeRepository {
    pub async fn write_image(&self, path: &str, cache_item: &ImageItem) -> Result<(), ErrorResponse> {
        fs::write(ROOT_PATH.to_string() + path, &cache_item.image).map_err(|_| {
            error!("Could not write image at {path}");
            ImageWriteError {
                path: path.to_string(),
            }
        })?;
        Ok(())
    }
}

impl ImageRepository for VolumeRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
            warn!("Defaulting to Jpeg format for {path}");
            ImageFormat::Jpeg
        });

        let bytes = fs::read(ROOT_PATH.to_string() + path).map_err(|_| {
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