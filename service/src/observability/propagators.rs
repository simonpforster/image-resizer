use hyper::header::HeaderName;
use hyper::HeaderMap;
use opentelemetry::propagation::{Extractor, Injector};

pub struct HyperHeaderInjector<'a>(pub &'a mut HeaderMap);

impl<'a> Injector for HyperHeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(header_name) = key.parse::<HeaderName>() {
            self.0.insert(header_name, value.parse().unwrap());
        }
    }
}

pub struct HyperHeaderExtractor<'a>(pub &'a HeaderMap);

impl<'a> Extractor for HyperHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)
            .and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys()
            .map(|k| k.as_str())
            .collect()
    }
}

