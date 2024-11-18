use fogcore::load_tracks_map_folder;
use fogcore::renderer::RenderedTrackMap;
use fogcore::renderer::TileRendererBasic;
use fogcore::renderer::TileRendererPremium;
use fogcore::renderer::TileRendererPremium2;
use fogcore::renderer::TileRendererTrait;
use fogcore::renderer::{BBox, Point};
use fogcore::TileSize;
use fogcore::{image_to_png_data, lat_to_tile_y, lng_to_tile_x};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

struct City {
    name: &'static str,
    lng: f64,
    lat: f64,
    zoom: i16,
}

fn generate_composed_image_with_white_background(png_data: &Vec<u8>) -> image::RgbaImage {
    let img = image::load_from_memory(png_data).unwrap();
    let rgba_img = img.to_rgba8();

    // Create new white background image
    let mut white_background = image::RgbaImage::new(rgba_img.width(), rgba_img.height());
    white_background
        .pixels_mut()
        .for_each(|p| *p = image::Rgba([255, 255, 255, 255]));

    // Composite the original image over the white background
    image::imageops::overlay(&mut white_background, &rgba_img, 0, 0);

    white_background
}

fn verify_image(name: &str, img: &image::RgbaImage) {
    // Save the image file
    let output_path = format!("tests/outputs/{}.png", name);
    img.save(&output_path).expect("Failed to save image");
    println!("Saved image file: {}", output_path);

    let hash_table_path = "tests/image_hashes.json";
    let mut hash_table: HashMap<String, String> = if Path::new(hash_table_path).exists() {
        let hash_table_content =
            fs::read_to_string(hash_table_path).expect("Failed to read hash table file");
        serde_json::from_str(&hash_table_content).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    // Calculate hash of the raw image data
    let mut hasher = Sha256::new();
    hasher.update(img.as_raw());
    let current_hash = format!("{:x}", hasher.finalize());

    if let Some(stored_hash) = hash_table.get(name) {
        // Entry exists, compare hashes
        assert_eq!(
            &current_hash, stored_hash,
            "Image hash mismatch for {}. Expected: {}, Got: {}",
            name, stored_hash, current_hash
        );
        println!("Verified image hash for: {}", name);
    } else {
        // No entry exists, add new entry
        hash_table.insert(name.to_string(), current_hash.clone());
        let hash_table_content =
            serde_json::to_string_pretty(&hash_table).expect("Failed to serialize hash table");
        fs::write(hash_table_path, hash_table_content).expect("Failed to write hash table file");
        println!("Added new hash entry for: {}", name);
    }
}

#[test]
fn main() {
    let fogmap = load_tracks_map_folder("static/tiles");

    let bg_color = image::Rgba([100, 0, 100, 255]);
    let fg_color = image::Rgba([0, 0, 0, 0]);

    // Define cities
    let cities = vec![
        City {
            name: "gba",
            lng: 113.6,
            lat: 22.7,
            zoom: 7,
        },
        City {
            name: "shenzhen",
            lng: 114.1,
            lat: 22.7,
            zoom: 9,
        },
        City {
            name: "athens",
            lng: 23.7,
            lat: 37.9,
            zoom: 9,
        },
        // City { name: "new_york", lng: -74.0, lat: 40.7, zoom: 9 },
        // City { name: "tokyo", lng: 139.7, lat: 35.7, zoom: 9 },
        // City { name: "sydney", lng: 151.2, lat: -33.9, zoom: 9 },
    ];

    // Process each city
    for city in cities {
        let tile_x = lng_to_tile_x(city.lng, city.zoom);
        let tile_y = lat_to_tile_y(city.lat, city.zoom);

        println!(
            "Processing {}: Tile X: {}, Tile Y: {}",
            city.name, tile_x, tile_y
        );

        let renderer = TileRendererBasic::new(TileSize::TileSize256);
        let image = renderer.render_image(&fogmap, tile_x, tile_y, city.zoom, bg_color, fg_color);
        let png = image_to_png_data(&image);
        let composed_image = generate_composed_image_with_white_background(&png);
        verify_image(city.name, &composed_image);
    }

    // You can add assertions here to verify the output if needed
    // For example:
    // assert!(Path::new("tests/outputs/shenzhen.png").exists());
}

#[test]
fn test_different_scale_rendering() {}

#[test]
fn test_different_size_rendering() {
    let tracks_map = load_tracks_map_folder("static/tiles");
    let mut rendered_map = RenderedTrackMap::new_with_track_map(tracks_map);

    let bbox = BBox {
        south_west: Point {
            lng: 113.841,
            lat: 22.445,
        },
        north_east: Point {
            lng: 114.343,
            lat: 22.769,
        },
    };

    rendered_map.set_rendering_backend(Box::new(TileRendererBasic::new(TileSize::TileSize256)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_256", &composed_image);

    rendered_map.set_rendering_backend(Box::new(TileRendererBasic::new(TileSize::TileSize512)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_512", &composed_image);

    rendered_map.set_rendering_backend(Box::new(TileRendererBasic::new(TileSize::TileSize1024)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_1024", &composed_image);

    // GPU rendering with high-DPI
    rendered_map.set_rendering_backend(Box::new(TileRendererPremium::new(TileSize::TileSize256)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_256_hidpi",
        &composed_image,
    );

    rendered_map.set_rendering_backend(Box::new(TileRendererPremium::new(TileSize::TileSize512)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_512_hidpi",
        &composed_image,
    );

    rendered_map.set_rendering_backend(Box::new(TileRendererPremium::new(TileSize::TileSize1024)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_1024_hidpi",
        &composed_image,
    );

    rendered_map.set_rendering_backend(Box::new(TileRendererPremium2::new(TileSize::TileSize256)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_256_wgpu",
        &composed_image,
    );

    rendered_map.set_rendering_backend(Box::new(TileRendererPremium2::new(TileSize::TileSize512)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_512_wgpu",
        &composed_image,
    );

    rendered_map.set_rendering_backend(Box::new(TileRendererPremium2::new(TileSize::TileSize1024)));
    let result = rendered_map
        .try_render_region_containing_bbox(bbox, 9)
        .unwrap();
    let composed_image = generate_composed_image_with_white_background(&result.data);
    verify_image(
        "different_size_rendering_shenzhen_1024_wgpu",
        &composed_image,
    );
}
