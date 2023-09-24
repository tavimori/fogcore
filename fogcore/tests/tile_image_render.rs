use fogcore::fogmaps::{FogMap, Tile};
use std::fs::File;
use std::io::Read;

#[test]
fn main() {
    let mut tile_file = File::open("tests/0921iihwtxn").unwrap();
    let mut content = Vec::new();
    tile_file.read_to_end(&mut content).unwrap();
    println!("loading a file with len{}.", content.len());
    let tile = Tile::create("0921iihwtxn", content);
}
