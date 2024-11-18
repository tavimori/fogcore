use std::f64::consts::PI;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::FogMap;
use image::{Rgba, RgbaImage};
use std::fs::{self, File};
use std::io::Cursor;
use std::io::Read;
use std::path::Path;

pub const DEFAULT_BG_COLOR2: Rgba<u8> = Rgba([0, 0, 0, 127]);
pub const DEFAULT_FG_COLOR2: Rgba<u8> = Rgba([0, 0, 0, 0]);

// TODO: make this consistant (currently these two are different)
// currently TileSize512 is used in rendered map, modify it to match with the default here.
// https://docs.mapbox.com/help/glossary/zoom-level/#tile-size
pub const DEFAULT_VIEW_SIZE_POWER: i16 = 8;
pub const DEFAULT_TILE_SIZE: TileSize = TileSize::TileSize512;

#[derive(Debug, Copy, Clone)]
pub enum TileSize {
    TileSize256,
    TileSize512,
    TileSize1024,
}

impl TileSize {
    pub fn size(&self) -> u32 {
        match self {
            TileSize::TileSize256 => 256,
            TileSize::TileSize512 => 512,
            TileSize::TileSize1024 => 1024,
        }
    }

    pub fn power(&self) -> i16 {
        match self {
            TileSize::TileSize256 => 8,
            TileSize::TileSize512 => 9,
            TileSize::TileSize1024 => 10,
        }
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lng_to_tile_x(lng: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let x = (lng + 180.0) / 360.0 * mul;
    x as i64
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lat_to_tile_y(lat: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let y = (PI - (lat * PI / 180.0).tan().asinh()) * mul / (2.0 * PI);
    y as i64
}

// TODO: split this into two functions?
pub fn tile_x_y_to_lng_lat(x: i32, y: i32, zoom: i32) -> (f64, f64) {
    let n = f64::powi(2.0, zoom);
    let lng = (x as f64 / n) * 360.0 - 180.0;
    let lat = (f64::atan(f64::sinh(PI * (1.0 - (2.0 * y as f64) / n))) * 180.0) / PI;
    (lng, lat)
}

pub fn image_to_png_data(image: &RgbaImage) -> Vec<u8> {
    let mut image_png: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut image_png), image::ImageFormat::Png)
        .unwrap();
    image_png
}

pub fn load_tracks_map_folder(tiles_dir: &str) -> FogMap {
    let mut fogmap = FogMap::new();
    let tiles_dir = Path::new(tiles_dir);

    // Load tiles
    if let Ok(entries) = fs::read_dir(tiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip hidden files (those starting with a dot)
                    if file_name.starts_with('.') {
                        continue;
                    }

                    let mut tile_file = File::open(&path).unwrap();
                    let mut content = Vec::new();
                    tile_file.read_to_end(&mut content).unwrap();
                    println!("Loading file: {} with length: {}", file_name, content.len());
                    fogmap.add_fow_file(file_name, content);
                }
            }
        }
    } else {
        panic!("Failed to read directory: {:?}", tiles_dir);
    }

    fogmap
}
