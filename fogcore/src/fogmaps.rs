struct FogMap {}

struct Tile {
    file_name: String,
    id: usize,
    x: u32,
    y: u32,
    blocks: u32,
}

impl Tile {
    pub fn create(filename: String, data: Vec<u8>) {}
}

struct Block {
    x: u32,
    y: u32,
    bitmap: Vec<u8>,
    extra_data: Vec<u8>,
}

impl Block {
    pub fn new(x: u32, y: u32, bitmap: Vec<u8>, extra_data: Vec<u8>) -> Self {
        Self {
            x,
            y,
            bitmap,
            extra_data,
        }
    }
}
