use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{debug, info};
use crate::CACHE;
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::ImageNotFoundInCacheError;
use crate::repository::{ImageItem, ImageRepository};

pub struct Cache {
    map: HashMap<String, ImageItem>,
}

impl Cache {
    pub fn default() -> Cache {
        Cache { map: HashMap::new() }
    }

    fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        let cache_item = self.map.get(path);
        match cache_item {
            Some(_) => debug!("Cache hit: {}", path),
            None => debug!("Cache miss: {}", path),
        }
        cache_item.map(|item| { item.clone() }).ok_or(ImageNotFoundInCacheError { path: path.to_string() })
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageItem) -> () {
        self.map.insert(path.to_string(), cache_item);
        debug!("Cache write: {}", path);
    }

    pub fn cull(&mut self) -> () {
        let cull_timer = Instant::now();
        let start_length = self.map.len();
        self.map.retain(|_, cache_item| cache_item.time.elapsed() < Duration::from_secs(300));
        self.map.shrink_to_fit();
        let diff = start_length - self.map.len();
        if diff > 1 { info!("Cache culled ({} ms) {} items.",  cull_timer.elapsed().as_millis(), diff) };
    }
}

pub struct CacheRepository {}

impl ImageRepository for CacheRepository {
    async fn read_image(&self, path: &str) -> Result<ImageItem, ErrorResponse> {
        CACHE.read().await.read_image(path)
    }
}