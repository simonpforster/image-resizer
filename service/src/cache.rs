use std::collections::HashMap;
use std::time::Instant;
use image::{DynamicImage, ImageFormat};
use log::info;

#[derive(Debug, Clone)]
pub struct ImageCacheItem {
    time: Instant,
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

    pub fn get_image(&self, url: &str) -> Result<&ImageCacheItem, CacheError> {
        info!("attempting to get: {}", url);
        self.map.get(url).ok_or(CacheError {})
    }

    pub fn put_image(&mut self, url: &str, format: ImageFormat, image: DynamicImage) -> Result<ImageCacheItem, CacheError> {
        info!("attempting to put: {}", url);
        self.map.insert(url.to_string(), ImageCacheItem { time: Instant::now(), format, image }).ok_or_else(|| CacheError {})
    }
}

#[derive(Debug)]
pub struct CacheError {}