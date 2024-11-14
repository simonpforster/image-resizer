use std::collections::HashMap;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageFormat};
use log::info;

#[derive(Debug, Clone)]
pub struct ImageCacheItem {
    pub time: Instant,
    pub format: ImageFormat,
    pub image: Box<DynamicImage>,
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
        let removables: Vec<String> = self.map.iter().filter(|(_, cache_item)| {
            cache_item.time.elapsed() >= Duration::from_secs(30)
        }).map(|(k, _)| { k.to_owned() }).collect();

        info!("Should cull {} items.", removables.len());
        let _ = removables.iter().for_each(|path| {
            let cull_timer_spec = Instant::now();
            let _ = self.map.remove(path).unwrap().format;
            info!("Culling {} took {} ms. ", path, cull_timer_spec.elapsed().as_millis());
        });

        info!("Cache culled ({} ms) {} items.",  cull_timer.elapsed().as_millis(), start_length - self.map.len());
    }
}