use fogcore::{FogMap, FogRenderer};
use std::fs::File;
use std::io::Read;

fn main() {
    let mut fogmap = FogMap::new();
    let renderer = FogRenderer::new();

    let mut tile_file = File::open("tests/0921iihwtxn").unwrap();
    let mut content = Vec::new();
    tile_file.read_to_end(&mut content).unwrap();
    println!("loading a file with len{}.", content.len());
    // let tile = Tile::create("0921iihwtxn", content);
    fogmap.add_fow_file("0921iihwtxn", content);
}
