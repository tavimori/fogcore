use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::collections::HashMap;
use std::convert::TryInto;

const FILENAME_MASK1: &str = "olhwjsktri";
const FILENAME_MASK2: &str = "eizxdwknmo";

const MAP_WIDTH: u64 = 512;
pub const TILE_WIDTH_OFFSET: i16 = 7;
const TILE_WIDTH: usize = 1 << TILE_WIDTH_OFFSET;
const TILE_HEADER_LEN: usize = TILE_WIDTH * TILE_WIDTH;
const TILE_HEADER_SIZE: usize = TILE_HEADER_LEN * 2;
const BLOCK_BITMAP_SIZE: usize = 512;
const BLOCK_EXTRA_DATA: usize = 3;
const BLOCK_SIZE: usize = BLOCK_BITMAP_SIZE + BLOCK_EXTRA_DATA;
pub const BITMAP_WIDTH_OFFSET: i16 = 6;
pub const BITMAP_WIDTH: u64 = 1 << BITMAP_WIDTH_OFFSET;
const ALL_OFFSET: i16 = TILE_WIDTH_OFFSET + BITMAP_WIDTH_OFFSET;

/// An in-memory efficient representation of a persons tracks on the Earth.
pub struct FogMap {
    pub tiles: HashMap<(u64, u64), Tile>,
}

impl FogMap {
    /// Creates an empty FogMap
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    /// Adds tracks by importing from a data file of the `Fog of World` App.
    ///
    /// Note that this operations will NOT REPLACE the existing tracks in FogMap, this operation is purely incremental.
    pub fn add_fow_file(&mut self, file_name: &str, data: Vec<u8>) {
        // TODO: current implementation will replace the tile if exist, change it to additive editing.

        let mut filename_encoding = std::collections::HashMap::new();
        for (i, char) in FILENAME_MASK1.chars().enumerate() {
            filename_encoding.insert(char, i);
        }
        // TODO: apply some checks here
        let id = file_name[4..file_name.len() - 2]
            .chars()
            .map(|id_masked| filename_encoding[&id_masked].to_string())
            .collect::<String>()
            .parse::<u64>()
            .unwrap();

        let x = id % MAP_WIDTH;
        let y = id / MAP_WIDTH;

        println!("parsing tile x:{} y:{}", x, y);

        let tile = self.tiles.entry((x, y)).or_insert(Tile::new());

        let data_inflate = decompress_to_vec_zlib(&data).unwrap();

        println!("inflated data len: {}", data_inflate.len());

        let header = &data_inflate[0..TILE_HEADER_SIZE];

        for i in 0..TILE_HEADER_LEN {
            // parse two u8 as a single u16 according to little endian
            let block_idx: u16 = (header[i * 2] as u16) | ((header[i * 2 + 1] as u16) << 8);
            if block_idx > 0 {
                let block_x: u64 = (i % TILE_WIDTH).try_into().unwrap();
                let block_y: u64 = (i / TILE_WIDTH).try_into().unwrap();
                let start_offset = TILE_HEADER_SIZE + ((block_idx - 1) as usize) * BLOCK_SIZE;
                let end_offset = start_offset + BLOCK_SIZE;
                let data = data_inflate[start_offset..end_offset].to_vec();
                let block = Block::new_with_data(data);
                println!("inserting block {}-{}", block_x, block_y);
                tile.add_by_blocks(block_x, block_y, block)
            }
        }

        println!("inflated data len: {:?}", data_inflate.len());
    }

    pub fn get_tile(&self, x: u64, y: u64) -> Option<&Tile> {
        // TODO: make everything u64?
        self.tiles.get(&(x, y))
    }
}

pub struct Tile {
    blocks: HashMap<(u64, u64), Block>,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    fn add_by_blocks(&mut self, x: u64, y: u64, block: Block) {
        // TODO: current implementation will replace the tile if exist, change it to additive editing.
        // TODO: rethink the data type and whether should use into()
        self.blocks.insert((x, y), block);
    }

    pub fn blocks(&self) -> &HashMap<(u64, u64), Block> {
        &self.blocks
    }
}

pub struct Block {
    data: Vec<u8>,
}

impl Block {
    pub fn new_with_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn is_visited(&self, x: u64, y: u64) -> bool {
        let bit_offset = 7 - (x % 8);
        let i = (x / 8) as usize;
        let j = (y) as usize;
        (self.data[i + j * 8] & (1 << bit_offset)) != 0
    }
}
