use std::path::Path;
use std::time::Instant;
use image::ImageFormat;
use log::{error, info, warn};
use crate::repository::{ImageItem, ImageRepository};
use crate::service::{ErrorResponse, ImageNotFoundError, ImageWriteError};

pub struct VolumeRepository {}

const ROOT_PATH: &str = "/mnt/shared-cache/";

impl VolumeRepository {
    pub async fn write_image(&self, path: &str, cache_item: &ImageItem) -> Result<(), ErrorResponse> {
        let timer = Instant::now();
        let d = Path::new(path);
        let _ = tokio::fs::create_dir_all(d.parent().unwrap()).await;
        tokio::fs::write(ROOT_PATH.to_string() + path, &cache_item.image).await.map_err(|_| {
            error!("Could not write image at {path}");
            ImageWriteError {
                path: path.to_string(),
            }
        })?;
        info!("FS write took {} ms for {}", timer.elapsed().as_millis(), path);
        Ok(())
    }
}

impl ImageRepository for VolumeRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let timer = Instant::now();
        let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
            warn!("Defaulting to Jpeg format for {path}");
            ImageFormat::Jpeg
        });

        let bytes = tokio::fs::read(ROOT_PATH.to_string() + path).await.map_err(|_| {
            error!("Could not decode image at {path}");
            ImageNotFoundError {
                path: path.to_string(),
            }
        })?;

        info!("FS read took {} ms for {}", timer.elapsed().as_millis(), path);
        Ok(ImageItem {
            time: Instant::now(),
            format,
            image: bytes,
        })
    }
}