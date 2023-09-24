use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::collections::HashMap;
use std::convert::TryInto;

const FILENAME_MASK1: &str = "olhwjsktri";
const FILENAME_MASK2: &str = "eizxdwknmo";

const MAP_WIDTH: u32 = 512;
const TILE_WIDTH_OFFSET: u32 = 7;
const TILE_WIDTH: usize = 1 << TILE_WIDTH_OFFSET;
const TILE_HEADER_LEN: usize = TILE_WIDTH * TILE_WIDTH;
const TILE_HEADER_SIZE: usize = TILE_HEADER_LEN * 2;
const BLOCK_BITMAP_SIZE: usize = 512;
const BLOCK_EXTRA_DATA: usize = 3;
const BLOCK_SIZE: usize = BLOCK_BITMAP_SIZE + BLOCK_EXTRA_DATA;
const BITMAP_WIDTH_OFFSET: u32 = 6;
const BITMAP_WIDTH: u32 = 1 << BITMAP_WIDTH_OFFSET;
const ALL_OFFSET: u32 = TILE_WIDTH_OFFSET + BITMAP_WIDTH_OFFSET;

pub struct FogMap {}

pub struct Tile {
    file_name: String,
    id: u32,
    x: u32,
    y: u32,
    blocks: HashMap<(u32, u32), Block>,
}

impl Tile {
    pub fn create(file_name: &str, data: Vec<u8>) -> Self {
        let mut filename_encoding = std::collections::HashMap::new();
        for (i, char) in FILENAME_MASK1.chars().enumerate() {
            filename_encoding.insert(char, i);
        }
        // TODO: apply some checks here
        let id = file_name[4..file_name.len() - 2]
            .chars()
            .map(|id_masked| filename_encoding[&id_masked].to_string())
            .collect::<String>()
            .parse::<u32>()
            .unwrap();

        let x = id % MAP_WIDTH;
        let y = id / MAP_WIDTH;

        println!("parsing tile x:{} y:{}", x, y);

        let data_inflate = decompress_to_vec_zlib(&data).unwrap();

        println!("inflated data len: {}", data_inflate.len());

        let header = &data_inflate[0..TILE_HEADER_SIZE];

        let mut blocks = HashMap::new();

        for i in 0..TILE_HEADER_LEN {
            // parse two u8 as a single u16 according to little endian
            let block_idx: u16 = (header[i * 2] as u16) | ((header[i * 2 + 1] as u16) << 8);
            if block_idx > 0 {
                let block_x: u8 = (i % TILE_WIDTH).try_into().unwrap();
                let block_y: u8 = (i / TILE_WIDTH).try_into().unwrap();
                let start_offset = TILE_HEADER_SIZE + ((block_idx - 1) as usize) * BLOCK_SIZE;
                let end_offset = start_offset + BLOCK_SIZE;
                let data = data_inflate[start_offset..end_offset].to_vec();
                let block = Block::new_with_data(block_x, block_y, data);
                println!("inserting block {}-{}", block_x, block_y);
                blocks.insert((x, y), block);
            }
        }

        println!("inflated data len: {:?}", data_inflate.len());

        Self {
            file_name: file_name.to_string(),
            id,
            x,
            y,
            blocks,
        }
    }
}

struct Block {
    x: u8,
    y: u8,
    data: Vec<u8>,
}

impl Block {
    pub fn new_with_data(x: u8, y: u8, data: Vec<u8>) -> Self {
        Self { x, y, data }
    }
}
