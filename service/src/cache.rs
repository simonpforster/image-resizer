use std::collections::HashMap;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageFormat};
use log::info;

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
        info!("Cache hit: {}", path);
        self.map.get(path)
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageCacheItem) -> () {
        info!("Cache miss: {}", path);
        self.map.insert(path.to_string(), cache_item);
    }

    pub fn cull(&mut self) -> () {
        let cull_timer = Instant::now();
        let expired_paths: Vec<String> = self.map.iter()
            .filter(|(_, item)| {
                item.time.elapsed() >= Duration::from_secs(300)
            })
            .map(|(path, _)| {
                path.to_string()
            }).collect();
        expired_paths.iter().for_each(|path| { self.map.remove(path); });
        info!("Culled {} items, took {}ms.", expired_paths.len(), cull_timer.elapsed().as_millis());
    }
}