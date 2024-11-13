use std::collections::HashMap;
use std::time::Instant;
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

    pub fn read_image(&self, url: &str) -> Result<&ImageCacheItem, CacheError> {
        info!("attempting to get: {}", url);
        self.map.get(url).ok_or(CacheError {})
    }

    pub fn write_image(&mut self, url: &str, cache_item: ImageCacheItem) -> Result<Option<ImageCacheItem>, CacheError> {
        info!("attempting to put: {}", url);
        Ok(self.map.insert(url.to_string(), cache_item))
    }
}

#[derive(Debug)]
pub struct CacheError {}