use fogcore::renderer::FogRenderer;
use fogcore::renderer::RenderedTrackMap;
use fogcore::renderer::{BBox, Point, TileSize};
use fogcore::{lat_to_tile_y, lng_to_tile_x};
use fogcore::load_tracks_map_folder;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use serde_json;
use sha2::{Sha256, Digest};

struct City {
    name: &'static str,
    lng: f64,
    lat: f64,
    zoom: i16,
}

fn generate_composed_image_with_white_background(image: &Vec<u8>) -> Vec<u8> {
    let pixmap = tiny_skia::Pixmap::decode_png(image).unwrap();
    let mut white_background = tiny_skia::Pixmap::new(pixmap.width(), pixmap.height()).unwrap();
    white_background.fill(tiny_skia::Color::WHITE);
    white_background.draw_pixmap(0, 0, pixmap.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    white_background.encode_png().unwrap()
}

fn verify_image(name: &str, image: &Vec<u8>) {
    let hash_table_path = "tests/image_hashes.json";
    let mut hash_table: HashMap<String, String> = if Path::new(hash_table_path).exists() {
        let hash_table_content = fs::read_to_string(hash_table_path)
            .expect("Failed to read hash table file");
        serde_json::from_str(&hash_table_content).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    // Calculate hash of the current image
    let mut hasher = Sha256::new();
    hasher.update(image);
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
        let hash_table_content = serde_json::to_string_pretty(&hash_table)
            .expect("Failed to serialize hash table");
        fs::write(hash_table_path, hash_table_content)
            .expect("Failed to write hash table file");
        println!("Added new hash entry for: {}", name);
    }

    // Always save the image file
    let output_path = format!("tests/outputs/{}.png", name);
    let mut file = File::create(&output_path).expect("Failed to create file");
    file.write_all(image).expect("Failed to write to file");
    println!("Saved image file: {}", output_path);
}

#[test]
fn main() {
    let fogmap = load_tracks_map_folder("static/tiles");

    let mut renderer = FogRenderer::new();
    renderer.set_bg_color(100, 0, 100, 255);
    renderer.set_fg_color(0, 0, 0, 0);

    // Define cities
    let cities = vec![
        City { name: "gba", lng: 113.6, lat: 22.7, zoom: 7 },
        City { name: "shenzhen", lng: 114.1, lat: 22.7, zoom: 9 },
        City { name: "athens", lng: 23.7, lat: 37.9, zoom: 9 },
        // City { name: "new_york", lng: -74.0, lat: 40.7, zoom: 9 },
        // City { name: "tokyo", lng: 139.7, lat: 35.7, zoom: 9 },
        // City { name: "sydney", lng: 151.2, lat: -33.9, zoom: 9 },
    ];

    // Process each city
    for city in cities {
        let tile_x = lng_to_tile_x(city.lng, city.zoom);
        let tile_y = lat_to_tile_y(city.lat, city.zoom);

        println!("Processing {}: Tile X: {}, Tile Y: {}", city.name, tile_x, tile_y);

        let pixmap = renderer.render_pixmap(&fogmap, tile_x, tile_y, city.zoom);
        let png = pixmap.encode_png().unwrap();
        verify_image(city.name, &png);
    }

    // You can add assertions here to verify the output if needed
    // For example:
    // assert!(Path::new("tests/outputs/shenzhen.png").exists());
}

#[test]
fn test_different_size_rendering() {
    let tracks_map = load_tracks_map_folder("static/tiles");
    let mut rendered_map = RenderedTrackMap::new_with_track_map(tracks_map);

    let bbox = BBox {
        south_west: Point { lng: 113.841, lat: 22.445 },
        north_east: Point { lng: 114.343, lat: 22.769 },
    };

    rendered_map.set_tile_size(TileSize::TileSize256);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_256", &composed_png);

    rendered_map.set_tile_size(TileSize::TileSize512);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_512", &composed_png);

    rendered_map.set_tile_size(TileSize::TileSize1024);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_1024", &composed_png);

    // GPU rendering with high-DPI
    rendered_map.set_use_gpu(true);

    rendered_map.set_tile_size(TileSize::TileSize256);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_256_hidpi", &composed_png);

    rendered_map.set_tile_size(TileSize::TileSize512);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_512_hidpi", &composed_png);

    rendered_map.set_tile_size(TileSize::TileSize1024);
    let result = rendered_map.try_render_region_containing_bbox(bbox, 9).unwrap();
    let composed_png = generate_composed_image_with_white_background(&result.data);
    verify_image("different_size_rendering_shenzhen_1024_hidpi", &composed_png);
}
