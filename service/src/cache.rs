use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use http_body_util::BodyExt;
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
        let cache_item = self.map.get(path);
        match cache_item {
            Some(_) => info!("Cache hit: {}", path),
            None => info!("Cache miss: {}", path),
        }
        cache_item
    }

    pub fn write_image(&mut self, path: &str, cache_item: ImageCacheItem) -> () {
        self.map.insert(path.to_string(), cache_item);
        info!("Cache write: {}", path);
    }

    pub fn cull(&mut self) -> () {
        let cull_timer = Instant::now();
        let start_length = self.map.len();
        let removables: Vec<&String> = self.map.iter().filter(|(_, cache_item)| {
            cache_item.time.elapsed() >= Duration::from_secs(30)
        }).map(|(k, _)| { k }).collect();

        let _ = removables.iter().map(|path| {
            let cull_timer_spec = Instant::now();
            self.map.remove(path);
            info!("Dropping {} took {} ms. ", path, cull_timer_spec.elapsed().as_millis());
        }).flatten().collect();

        info!("Cache culled ({} ms) {} items.",  cull_timer.elapsed().as_millis(), start_length - self.map.len());
    }
}