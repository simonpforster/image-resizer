use std::collections::HashMap;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageFormat};
use log::{debug, info};

#[derive(Debug, Clone)]
pub struct ImageCacheItem {
    pub time: Instant,
    pub format: ImageFormat,
    pub image: DynamicImage,
}

pub struct Cache {
    map: HashMap<String, ImageCacheItem>,
}

impl Cache {
    pub fn default() -> Cache {
        Cache { map: HashMap::new() }
    }

    pub fn read_image(&self, path: &str) -> Option<&ImageCacheItem> {
        let cache_item = self.map.get(path);
        match cache_item {
            Some(_) => debug!("Cache hit: {}", path),
            None => debug!("Cache miss: {}", path),
        }
        cache_item
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageCacheItem) -> () {
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