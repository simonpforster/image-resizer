use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageNotFoundInCacheError;
use crate::repository::{ImageItem, ImageRepository};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, info};

lazy_static! {
    static ref CACHE: RwLock<Cache> = RwLock::new(Cache::default());
}

pub struct Cache {
    map: HashMap<String, ImageItem>,
}

impl Cache {
    pub fn default() -> Cache {
        Cache {
            map: HashMap::new(),
        }
    }

    fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let cache_item = self.map.get(path);
        match cache_item {
            Some(_) => debug!("Cache hit: {}", path),
            None => debug!("Cache miss: {}", path),
        }
        cache_item.cloned().ok_or(ImageNotFoundInCacheError {
            path: path.to_string(),
        })
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageItem) {
        self.map.insert(path.to_string(), cache_item);
        debug!("Cache write: {}", path);
    }

    pub fn cull(&mut self) {
        let cull_timer = Instant::now();
        let start_length = self.map.len();
        self.map
            .retain(|_, cache_item| cache_item.time.elapsed() < Duration::from_secs(300));
        self.map.shrink_to_fit();
        let diff = start_length - self.map.len();
        if diff > 1 {
            info!(
                "Cache culled ({} ms) {} items.",
                cull_timer.elapsed().as_millis(),
                diff
            )
        };
    }
}

pub struct CacheRepository {}

impl CacheRepository {
    pub async fn write_image(
        &self,
        new_path: String,
        cache_item: ImageItem,
    ) -> Result<(), ErrorResponse> {
        CACHE.write().await.write_image(&new_path, cache_item);
        Ok(())
    }

    pub async fn cull_images_loop(&self) {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            CACHE.write().await.cull();
        }
    }
}

impl ImageRepository for CacheRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        CACHE.read().await.read_image(path)
    }
}
