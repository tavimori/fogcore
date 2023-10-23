use fogcore::{FogMap, FogRenderer};
use std::f64::consts::PI;
use std::fs::File;
use std::io::Read;
use tiny_skia;

// Web Mercator projection
fn lng_lat_to_tile_x_y(lng: f64, lat: f64, zoom: i16) -> (i64, i64) {
    let mul = (1 << zoom) as f64;
    let x = (lng + 180.0) / 360.0 * mul;
    let y = (PI - (lat * PI / 180.0).tan().asinh()) * mul / (2.0 * PI);
    (x as i64, y as i64)
}

fn main() {
    let mut fogmap = FogMap::new();

    let mut tile_file = File::open("tests/0921iihwtxn").unwrap();
    let mut content = Vec::new();
    tile_file.read_to_end(&mut content).unwrap();
    println!("loading a file with len{}.", content.len());
    fogmap.add_fow_file("0921iihwtxn", content);

    let mut renderer = FogRenderer::new();
    renderer.set_bg_color(tiny_skia::Color::BLACK);
    renderer.set_fg_color(tiny_skia::Color::WHITE);

    // The Palace Museum
    let lng = 116.39290;
    let lat = 39.91535;


    for zoom in 0..22 {
        // https://developers.google.com/maps/documentation/javascript/coordinates
        let (x, y) = lng_lat_to_tile_x_y(lng, lat, zoom);

        println!("draw x: {}, y: {}, zoom: {}", x, y, zoom);
        let pixmap = renderer.render_pixmap(&fogmap, x, y, zoom);
        pixmap.save_png(format!("image{}.png", zoom)).unwrap();
    }
}
