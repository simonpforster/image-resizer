use crate::domain::dimension::Dimension::{Height, Width};
use crate::domain::error::ErrorResponse;
use crate::domain::error::ErrorResponse::ImageWriteError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Dimension {
    Height(u32),
    Width(u32),
}

pub fn decode(query: &str) -> Result<Dimension, ErrorResponse> {
    let params: HashMap<&str, &str> = query
        .split('&')
        .collect::<Vec<&str>>()
        .iter()
        .filter_map(|pair| {
            let some: Vec<&str> = pair.split('=').collect();
            match some.len() {
                2 => {
                    let couple: (&str, &str) = (some.first()?, some.get(1)?);
                    Some(couple)
                }
                _ => None,
            }
        })
        .collect::<HashMap<&str, &str>>();

    let opt_width = params.get("width");
    let opt_height = params.get("height");

    match opt_width {
        Some(w) => {
            let a = str::parse::<u32>(w).map_err(|_| ImageWriteError {
                path: query.to_string(),
            })?;
            Ok(Width(a))
        }
        None => match opt_height {
            Some(h) => {
                let a = str::parse::<u32>(h).map_err(|_| ImageWriteError {
                    path: query.to_string(),
                })?;
                Ok(Height(a))
            }
            None => {
                // need param error
                Err(ImageWriteError {
                    path: query.to_string(),
                })
            }
        },
    }
}
