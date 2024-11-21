use std::path::Path;
use std::time::Instant;
use futures_util::TryFutureExt;
use image::ImageFormat;
use log::{error, info, warn};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use crate::repository::{ImageItem, ImageRepository};
use crate::service::{ErrorResponse, ImageDecodeError, ImageNotFoundError, ImageWriteError};

pub struct VolumeRepository {}

const ROOT_PATH: &str = "/mnt/shared-cache";

impl VolumeRepository {
    pub async fn write_image(&self, path: &str, cache_item: &ImageItem) -> Result<(), ErrorResponse> {
        let timer = Instant::now();
        let full_path = ROOT_PATH.to_string() + path;
        let parent = Path::new(&full_path).parent().unwrap();

        let _ = tokio::fs::create_dir_all(parent).await.map_err(|_| {
            error!("Could not create dirs to image at {full_path}");
            ImageWriteError {
                path: path.to_string(),
            }
        })?;
        tokio::fs::write(&full_path, &cache_item.image).await.map_err(|_| {
            error!("Could not write image at {full_path}");
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
        let full_path = ROOT_PATH.to_string() + path;

        let format: ImageFormat = ImageFormat::from_path(path).unwrap_or_else(|_| {
            warn!("Defaulting to Jpeg format for {full_path}");
            ImageFormat::Jpeg
        });

        let file: File = File::open(&full_path).map_err(|_| {
            error!("FS could not read image at {full_path}");
            ImageNotFoundError {
                path: path.to_string(),
            }
        }).await?;

        let mut reader = BufReader::new(file);
        let mut bytes: Vec<u8> = Vec::new();
        let _ = reader.read_to_end(&mut bytes).map_err(|_| {
            error!("FS could not read byte buffer at {full_path}");
            ImageDecodeError {
                path: path.to_string(),
            }
        }).await?;

        info!("FS read took {} ms for {}", timer.elapsed().as_millis(), path);
        Ok(ImageItem {
            time: Instant::now(),
            format,
            image: bytes,
        })
    }
}