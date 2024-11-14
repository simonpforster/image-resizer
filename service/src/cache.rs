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
        let read_timer = Instant::now();
        let cache_item = self.map.get(path);
        let elapsed = read_timer.elapsed().as_millis();
        match cache_item {
            Some(_) => info!("Cache hit ({} ms): {}", elapsed, path),
            None => info!("Cache miss ({} ms): {}", elapsed, path),
        }
        cache_item
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageCacheItem) -> () {
        let read_timer = Instant::now();
        self.map.insert(path.to_string(), cache_item);
        info!("Cache write ({} ms): {}", read_timer.elapsed().as_millis(), path);
    }

    pub fn cull(&mut self) -> () {
        let find_expired_timer = Instant::now();
        let expired_paths: Vec<String> = self.map.iter()
            .filter(|(_, item)| {
                item.time.elapsed() >= Duration::from_secs(300)
            })
            .map(|(path, _)| {
                path.to_string()
            }).collect();
        let expired_elapsed = find_expired_timer.elapsed().as_millis();
        info!("Expired found ({} ms).", expired_elapsed);
        let cull_timer = Instant::now();
        expired_paths.iter().for_each(|path| { self.map.remove(path); });
        info!("Cache cleared ({} ms) {} items.",  cull_timer.elapsed().as_millis(), expired_paths.len());
    }
}