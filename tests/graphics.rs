use fogcore::fogmaps::FogMap;
use fogcore::renderer::FogRenderer;
use fogcore::{lng_to_tile_x, lat_to_tile_y};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tiny_skia::Pixmap;

struct City {
    name: &'static str,
    lng: f64,
    lat: f64,
    zoom: i16,
}

#[test]
fn main() {
    let mut fogmap = FogMap::new();
    let tiles_dir = Path::new("static/tiles");

    // Load tiles
    if let Ok(entries) = fs::read_dir(tiles_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
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
        }
    } else {
        panic!("Failed to read directory: {:?}", tiles_dir);
    }

    let renderer = FogRenderer::new();

    // Define cities
    let cities = vec![
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
        let output_path = format!("tests/outputs/{}.png", city.name);
        let mut file = File::create(&output_path).unwrap();
        let png = pixmap.encode_png().unwrap();
        
        file.write_all(&png).unwrap();
        println!("Saved to {}", output_path);
    }

    // You can add assertions here to verify the output if needed
    // For example:
    // assert!(Path::new("tests/outputs/shenzhen.png").exists());
}
