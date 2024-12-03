use crate::repository::ImageRepository;
use crate::service::{ErrorResponse, ImageNotFoundInCacheError, ImageWriteError};
use futures_util::TryFutureExt;
use std::path::Path;
use std::time::Instant;
use tracing::{error, info, instrument};

#[derive(Debug)]
pub struct VolumeRepository {}

const ROOT_PATH: &str = "/mnt/shared-cache";

impl VolumeRepository {
    #[instrument(skip(cache_item))]
    pub async fn write_image(
        &self,
        path: &str,
        cache_item: &[u8],
    ) -> Result<(), ErrorResponse> {
        let timer = Instant::now();
        let full_path = ROOT_PATH.to_string() + path;
        let parent = Path::new(&full_path).parent().unwrap();

        let _ = tokio::fs::create_dir_all(parent).await.map_err(|_| {
            error!("Could not create dirs to image at {full_path}");
            ImageWriteError {
                path: path.to_string(),
            }
        })?;
        tokio::fs::write(&full_path, cache_item)
            .await
            .map_err(|_| {
                error!("Could not write image at {full_path}");
                ImageWriteError {
                    path: path.to_string(),
                }
            })?;
        info!(
            "FS write took {} ms for {}",
            timer.elapsed().as_millis(),
            path
        );
        Ok(())
    }
}

impl ImageRepository for VolumeRepository {
    #[instrument]
    async fn read_image(&self, path: &str) -> Result<Vec<u8>, ErrorResponse> {
        let full_path = ROOT_PATH.to_string() + path;

        let bytes: Vec<u8> = tokio::fs::read(&full_path)
            .map_err(|_| {
                info!("FS could not read image at {full_path}");
                ImageNotFoundInCacheError {
                    path: path.to_string(),
                }
            })
            .await?;
        Ok(bytes)
    }
}
