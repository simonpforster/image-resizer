use crate::domain::server_timing::ServerTiming;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use image::ImageFormat;

pub mod dimension;
pub mod error;
pub mod server_timing;

pub struct ImageData {
    pub body: BoxBody<Bytes, hyper::Error>,
    pub server_timing: ServerTiming,
    pub format_extension: String,
    pub content_length: u64,
}

pub trait ExtensionProvider {
    fn get_format_extension(&self) -> String;
}

impl ExtensionProvider for ImageFormat {
    /// A little Pimp My Library pattern
    fn get_format_extension(&self) -> String {
        self.extensions_str()
            .first()
            .map(|ext| "/".to_owned() + ext)
            .unwrap_or_else(|| "".to_owned())
    }
}