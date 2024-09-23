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
    // let mut tile_file = File::open("tests/33c1lljorhmz").unwrap();
    let mut content = Vec::new();
    tile_file.read_to_end(&mut content).unwrap();
    // println!("loading a file with len{}.", content.len());
    fogmap.add_fow_file("0921iihwtxn", content);
    // fogmap.add_fow_file("33c1lljorhmz", content);

    let mut renderer = FogRenderer::new();
    renderer.set_bg_color(100, 0, 100, 255);
    renderer.set_fg_color(0, 0, 0, 0);

    // pkx
    let pkx_lng = 116.4233640802707;
    let pkx_lat = 39.51154847952295;

    // sha
    let sha_lng = 121.34651031397549;
    let sha_lat = 31.202812552714104;

    fogmap.add_line(pkx_lng, pkx_lat, sha_lng, sha_lat);

    // Melbourne to Hawaii
    let (start_lng, start_lat, end_lng, end_lat) =
        (144.847737, 37.6721702, -160.3644029, 21.3186185);
    fogmap.add_line(start_lng, start_lat, end_lng, end_lat);

    // Hawaii to Guan
    let (start_lng, start_lat, end_lng, end_lat) =
        (-160.3644029, 21.3186185, 121.4708788, 9.4963078);
    fogmap.add_line(start_lng, start_lat, end_lng, end_lat);

    for zoom in 0..20 {
        // https://developers.google.com/maps/documentation/javascript/coordinates
        let (x, y) = lng_lat_to_tile_x_y(sha_lng, sha_lat, zoom);

        println!("draw x: {}, y: {}, zoom: {}", x, y, zoom);
        let pixmap = renderer.render_pixmap(&fogmap, x, y, zoom);
        pixmap.save_png(format!("image{}.png", zoom)).unwrap();
    }
}
