use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::collections::HashMap;
use std::convert::TryInto;
use std::f64::consts::PI;

const FILENAME_MASK1: &str = "olhwjsktri";
const FILENAME_MASK2: &str = "eizxdwknmo";

const MAP_WIDTH_OFFSET: i16 = 9;
const MAP_WIDTH: i64 = 1 << MAP_WIDTH_OFFSET;
pub const TILE_WIDTH_OFFSET: i16 = 7;
const TILE_WIDTH: i64 = 1 << TILE_WIDTH_OFFSET;
const TILE_HEADER_LEN: i64 = TILE_WIDTH * TILE_WIDTH;
const TILE_HEADER_SIZE: usize = (TILE_HEADER_LEN * 2) as usize;
const BLOCK_BITMAP_SIZE: usize = 512;
const BLOCK_EXTRA_DATA: usize = 3;
const BLOCK_SIZE: usize = BLOCK_BITMAP_SIZE + BLOCK_EXTRA_DATA;
pub const BITMAP_WIDTH_OFFSET: i16 = 6;
pub const BITMAP_WIDTH: i64 = 1 << BITMAP_WIDTH_OFFSET;
const ALL_OFFSET: i16 = TILE_WIDTH_OFFSET + BITMAP_WIDTH_OFFSET;

/// An in-memory efficient representation of a persons tracks on the Earth.
pub struct FogMap {
    pub tiles: HashMap<(i64, i64), Tile>,
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
            .parse::<i64>()
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
            let index = (i as usize) * 2;
            let block_idx: u16 = (header[index] as u16) | ((header[index + 1] as u16) << 8);
            if block_idx > 0 {
                let block_x: i64 = (i % TILE_WIDTH);
                let block_y: i64 = (i / TILE_WIDTH);
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

    pub fn get_tile(&self, x: i64, y: i64) -> Option<&Tile> {
        self.tiles.get(&(x, y))
    }

    // Web Mercator projection
    pub fn lng_lat_to_tile_x_y(lng: f64, lat: f64, zoom: i16) -> (i64, i64) {
        let mul = (1 << zoom) as f64;
        let x = (lng + 180.0) / 360.0 * mul;
        let y = (PI - (lat * PI / 180.0).tan().asinh()) * mul / (2.0 * PI);
        (x as i64, y as i64)
    }

    pub fn add_line(&mut self, start_lng: f64, start_lat: f64, end_lng: f64, end_lat: f64) {
        println!("[{},{}] to [{},{}]", start_lng, start_lat, end_lng, end_lat);

        let (x0, y0) =
            Self::lng_lat_to_tile_x_y(start_lng, start_lat, ALL_OFFSET + MAP_WIDTH_OFFSET);
        let (x1, y1) = Self::lng_lat_to_tile_x_y(end_lng, end_lat, ALL_OFFSET + MAP_WIDTH_OFFSET);

        // Iterators, counters required by algorithm
        // Calculate line deltas
        let dx = x1 as i64 - x0 as i64;
        let dy = y1 as i64 - y0 as i64;
        // Create a positive copy of deltas (makes iterating easier)
        let dx0 = dx.abs();
        let dy0 = dy.abs();
        // Calculate error intervals for both axis
        let mut px = 2 * dy0 - dx0;
        let mut py = 2 * dx0 - dy0;
        // The line is X-axis dominant
        if dy0 <= dx0 {
            let (mut x, mut y, xe) = if dx >= 0 {
                // Line is drawn left to right
                (x0, y0, x1)
            } else {
                // Line is drawn right to left (swap ends)
                (x1, y1, x0)
            };
            while x < xe {
                let (tile_x, tile_y) = (x >> ALL_OFFSET, y >> ALL_OFFSET);
                println!("accessing tile {}-{}", tile_x, tile_y);
                let tile = self.tiles.entry((tile_x, tile_y)).or_insert(Tile::new());
                (x, y, px) = tile.add_line(
                    x - (tile_x << ALL_OFFSET),
                    y - (tile_y << ALL_OFFSET),
                    xe - (tile_x << ALL_OFFSET),
                    px,
                    dx0,
                    dy0,
                    true,
                    (dx < 0 && dy < 0) || (dx > 0 && dy > 0),
                );
                x += tile_x << ALL_OFFSET;
                y += tile_y << ALL_OFFSET;
                println!("tile draw: tile_x: {}, tile_y: {}", tile_x, tile_y);
            }
        } else {
            // The line is Y-axis dominant
            let (mut x, mut y, ye) = if dy >= 0 {
                // Line is drawn bottom to top
                (x0, y0, y1)
            } else {
                // Line is drawn top to bottom
                (x1, y1, y0)
            };
            println!("y {} ye {}", y, ye);
            while y < ye {
                let (tile_x, tile_y) = (x >> ALL_OFFSET, y >> ALL_OFFSET);
                println!("accessing tile {}-{}", tile_x, tile_y);
                let tile = self.tiles.entry((tile_x, tile_y)).or_insert(Tile::new());
                (x, y, py) = tile.add_line(
                    x - (tile_x << ALL_OFFSET),
                    y - (tile_y << ALL_OFFSET),
                    ye - (tile_y << ALL_OFFSET),
                    py,
                    dx0,
                    dy0,
                    false,
                    (dx < 0 && dy < 0) || (dx > 0 && dy > 0),
                );
                x += tile_x << ALL_OFFSET;
                y += tile_y << ALL_OFFSET;
                println!("tile draw: tile_x: {}, tile_y: {}", tile_x, tile_y);
            }
        }
    }
}

pub struct Tile {
    blocks: HashMap<(i64, i64), Block>,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    fn add_by_blocks(&mut self, x: i64, y: i64, block: Block) {
        // TODO: current implementation will replace the tile if exist, change it to additive editing.
        // TODO: rethink the data type and whether should use into()
        self.blocks.insert((x, y), block);
    }

    pub fn blocks(&self) -> &HashMap<(i64, i64), Block> {
        &self.blocks
    }

    fn add_line(
        &mut self,
        x: i64,
        y: i64,
        e: i64,
        p: i64,
        dx0: i64,
        dy0: i64,
        xaxis: bool,
        quadrants13: bool,
    ) -> (i64, i64, i64) {
        let mut p = p;
        let mut x = x;
        let mut y = y;
        if xaxis {
            // Rasterize the line
            while x < e {
                if (x >> BITMAP_WIDTH_OFFSET >= TILE_WIDTH
                    || y >> BITMAP_WIDTH_OFFSET < 0
                    || y >> BITMAP_WIDTH_OFFSET >= TILE_WIDTH)
                {
                    break;
                }
                let block_x = x >> BITMAP_WIDTH_OFFSET;
                let block_y = y >> BITMAP_WIDTH_OFFSET;

                let block = self
                    .blocks
                    .entry((block_x, block_y))
                    .or_insert(Block::new());

                (x, y, p) = block.add_line(
                    x - (block_x << BITMAP_WIDTH_OFFSET),
                    y - (block_y << BITMAP_WIDTH_OFFSET),
                    e - (block_x << BITMAP_WIDTH_OFFSET),
                    p,
                    dx0,
                    dy0,
                    xaxis,
                    quadrants13,
                );

                x += block_x << BITMAP_WIDTH_OFFSET;
                y += block_y << BITMAP_WIDTH_OFFSET;
            }
        } else {
            // Rasterize the line
            while y < e {
                if (y >> BITMAP_WIDTH_OFFSET >= TILE_WIDTH
                    || x >> BITMAP_WIDTH_OFFSET < 0
                    || x >> BITMAP_WIDTH_OFFSET >= TILE_WIDTH)
                {
                    break;
                }
                let block_x = x >> BITMAP_WIDTH_OFFSET;
                let block_y = y >> BITMAP_WIDTH_OFFSET;

                let block = self
                    .blocks
                    .entry((block_x, block_y))
                    .or_insert(Block::new());
                (x, y, p) = block.add_line(
                    x - (block_x << BITMAP_WIDTH_OFFSET),
                    y - (block_y << BITMAP_WIDTH_OFFSET),
                    e - (block_y << BITMAP_WIDTH_OFFSET),
                    p,
                    dx0,
                    dy0,
                    xaxis,
                    quadrants13,
                );

                x += block_x << BITMAP_WIDTH_OFFSET;
                y += block_y << BITMAP_WIDTH_OFFSET;
            }
        }
        (x, y, p)
    }
}

pub struct Block {
    data: Vec<u8>,
}

impl Block {
    pub fn new() -> Self {
        let data = vec![0u8; BLOCK_SIZE];
        Self { data }
    }

    // TODO: if a block is from fog of world, there may be some extra data in the end.
    pub fn new_with_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn is_visited(&self, x: i64, y: i64) -> bool {
        let bit_offset = 7 - (x % 8);
        let i = (x / 8) as usize;
        let j = (y) as usize;
        (self.data[i + j * 8] & (1 << bit_offset)) != 0
    }

    fn set_point(&mut self, x: i64, y: i64, val: bool) {
        let bit_offset = 7 - (x % 8);
        let i = (x / 8) as usize;
        let j = (y) as usize;
        let val_number = if val { 1 } else { 0 };
        self.data[i + j * 8] =
            (self.data[i + j * 8] & !(1 << bit_offset)) | (val_number << bit_offset);
    }

    // a modified Bresenham algorithm with initialized error from upper layer
    fn add_line(
        &mut self,
        x: i64,
        y: i64,
        e: i64,
        p: i64,
        dx0: i64,
        dy0: i64,
        xaxis: bool,
        quadrants13: bool,
    ) -> (i64, i64, i64) {
        // Draw the first pixel
        let mut p = p;
        let mut x = x;
        let mut y = y;
        self.set_point(x, y, true);
        if xaxis {
            // Rasterize the line
            while x < e {
                x = x + 1;
                // Deal with octants...
                if p < 0 {
                    p = p + 2 * dy0;
                } else {
                    if quadrants13 {
                        y = y + 1;
                    } else {
                        y = y - 1;
                    }
                    p = p + 2 * (dy0 - dx0);
                }

                if (x >= BITMAP_WIDTH || y < 0 || y >= BITMAP_WIDTH) {
                    break;
                }
                // Draw pixel from line span at
                // currently rasterized position
                self.set_point(x, y, true);
            }
        } else {
            // The line is Y-axis dominant
            // Rasterize the line
            while y < e {
                y = y + 1;
                // Deal with octants...
                if p <= 0 {
                    p = p + 2 * dx0;
                } else {
                    if quadrants13 {
                        x = x + 1;
                    } else {
                        x = x - 1;
                    }
                    p = p + 2 * (dx0 - dy0);
                }

                if (y >= BITMAP_WIDTH || x < 0 || x >= BITMAP_WIDTH) {
                    break;
                }
                // Draw pixel from line span at
                // currently rasterized position
                self.set_point(x, y, true);
            }
        }
        (x, y, p)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add_line() {
        let mut fogmap = FogMap::new();
        fogmap.add_line(121.5157559, 31.29735617, 121.515725, 31.29731979);
    }
}
