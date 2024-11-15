use image::ImageFormat;

pub mod error;
pub mod dimension;
pub mod server_timing;

pub trait ExtensionProvider {
    fn get_format_extension(&self) -> String;
}


impl ExtensionProvider for ImageFormat {
    /// A little Pimp My Library pattern
    fn get_format_extension(&self) -> String {
        self.extensions_str().to_owned().iter().next().map(|ext| "/".to_owned() + ext).unwrap_or_else(|| "".to_owned())
    }
}