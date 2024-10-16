use std::collections::HashMap;
use log::info;
use crate::dimension::Dimension::{Height, Width};
use crate::error::ErrorResponse;
use crate::error::ErrorResponse::ImageWriteError;

#[derive(Debug, Clone)]
pub enum Dimension {
    Height(u32),
    Width(u32),
}

pub fn decode(query: &str) -> Result<Dimension, ErrorResponse> {
    let params: HashMap<&str, &str> =
        query.split('&').collect::<Vec<&str>>().iter()
            .map(|pair| {
                let some: Vec<&str> = pair.split('=').collect();
                match some.iter().count() {
                    2 => {
                        let couple: (&str, &str) = (some.get(0)?, some.get(1)?);
                        Some(couple)
                    }
                    _ => None,
                }
            }).flatten().collect::<HashMap::<&str, &str>>();
    info!("Parsed query params into hashmap, finding relevant ones");

    let opt_width = params.get("width");
    let opt_height = params.get("height");

    match opt_width {
        Some(w) => {
            let a = str::parse::<u32>(w).map_err(|_| ImageWriteError { path: query.to_string() })?;
            Ok(Width(a))
        }
        None => match opt_height {
            Some(h) => {
                let a = str::parse::<u32>(h).map_err(|_| ImageWriteError { path: query.to_string() })?;
                Ok(Height(a))
            }
            None => {
                // need param error
                Err(ImageWriteError { path: query.to_string() })
            }
        },
    }
}