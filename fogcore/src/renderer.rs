use crate::fogmaps::{Block, Tile};
use crate::fogmaps::{BITMAP_WIDTH, BITMAP_WIDTH_OFFSET, TILE_WIDTH_OFFSET};
use crate::FogMap;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

const FOW_TILE_ZOOM: i16 = 9;
const FOW_BLOCK_ZOOM: i16 = FOW_TILE_ZOOM + TILE_WIDTH_OFFSET;
const CANVAS_SIZE_OFFSET: i16 = 9;
// const CANVAS_SIZE: u16 = 1 << CANVAS_SIZE_OFFSET;

pub struct FogRenderer {
    canvas_size_order: u16,
}

impl FogRenderer {
    /// Create a FogRenderer.
    pub fn new() -> Self {
        Self {
            canvas_size_order: 9,
        }
    }

    /// Render a given location of FogMap data onto a Pixmap.
    ///
    /// * `fogmap`: an instance of FogMap.
    /// * `tile_x`: x-index of a tile, provided the zoom level.
    /// * `tile_y`: y-index of a tile, provided the zoom level.
    /// * `zoom`: zoom levels. Please refer to [OSM zoom levels](https://wiki.openstreetmap.org/wiki/Zoom_levels) for more infomation.
    /// * `width`: width of an image in pixels.
    pub fn render_pixmap(
        &self,
        fogmap: &FogMap,
        tile_x: u64,
        tile_y: u64,
        zoom: i16,
        canvas_size_order: i16,
    ) -> Pixmap {
        let width = 1 << canvas_size_order;
        let mut pixmap = Pixmap::new(width, width).unwrap();

        // TODO: if zero?

        let mut paint = Paint::default();
        paint.set_color(Color::TRANSPARENT);

        if zoom <= FOW_TILE_ZOOM {
            // the drawing tile is larger than or equal to a fogmap tile, render multiple fogmap tiles.

            let canvas_num_fow_tile_offset = FOW_TILE_ZOOM - zoom;
            let fm_tile_x_min = tile_x << canvas_num_fow_tile_offset;
            let fm_tile_x_max = (tile_x + 1) << canvas_num_fow_tile_offset;
            let fm_tile_y_min = tile_y << canvas_num_fow_tile_offset;
            let fm_tile_y_max = (tile_y + 1) << canvas_num_fow_tile_offset;

            for fm_tile_x in fm_tile_x_min..fm_tile_x_max {
                for fm_tile_y in fm_tile_y_min..fm_tile_y_max {
                    if let Some(tile) = fogmap.get_tile(fm_tile_x, fm_tile_y) {
                        let canvas_fow_tile_size_offset =
                            canvas_size_order - canvas_num_fow_tile_offset;
                        Self::render_tile_on_pixmap(
                            tile,
                            &mut pixmap,
                            (fm_tile_x - fm_tile_x_min) << canvas_fow_tile_size_offset,
                            (fm_tile_y - fm_tile_y_min) << canvas_fow_tile_size_offset,
                            canvas_fow_tile_size_offset,
                        );
                    }
                }
            }
        } else {
            // the drawing tile is smaller than a fogmap tile.
            let tile_over_offset = zoom - FOW_TILE_ZOOM;
            let fow_tile_x = tile_x >> tile_over_offset;
            let fow_tile_y = tile_y >> tile_over_offset;
            let sub_tile_mask = (1 << tile_over_offset) - 1;

            let canvas_num_fow_block_offset = TILE_WIDTH_OFFSET - tile_over_offset;

            if zoom > FOW_BLOCK_ZOOM {
                // sub-block rendering
                let fow_block_x = (tile_x & sub_tile_mask) >> -canvas_num_fow_block_offset;
                let fow_block_y = (tile_y & sub_tile_mask) >> -canvas_num_fow_block_offset;
                let sub_block_mask = (1 << (tile_over_offset - TILE_WIDTH_OFFSET)) - 1;

                let canvas_num_fow_pixel_offset = canvas_num_fow_block_offset + BITMAP_WIDTH_OFFSET;

                let fm_block_pixel_x_min = (tile_x & sub_block_mask) << canvas_num_fow_pixel_offset;
                let fm_block_pixel_x_max =
                    ((tile_x & sub_block_mask) + 1) << canvas_num_fow_pixel_offset;
                let fm_block_pixel_y_min = (tile_y & sub_block_mask) << canvas_num_fow_pixel_offset;
                let fm_block_pixel_y_max =
                    ((tile_y & sub_block_mask) + 1) << canvas_num_fow_pixel_offset;

                if let Some(tile) = fogmap.tiles.get(&(fow_tile_x, fow_tile_y)) {
                    if let Some(block) = tile.blocks().get(&(fow_block_x, fow_block_y)) {
                        for fm_pix_x in fm_block_pixel_x_min..fm_block_pixel_x_max {
                            for fm_pix_y in fm_block_pixel_y_min..fm_block_pixel_y_max {
                                let canvas_fow_pixel_size_offset =
                                    CANVAS_SIZE_OFFSET - canvas_num_fow_pixel_offset;
                                if block.is_visited(fm_pix_x, fm_pix_y) {
                                    let x = (fm_pix_x - fm_block_pixel_x_min)
                                        << canvas_fow_pixel_size_offset;
                                    let y = (fm_pix_y - fm_block_pixel_y_min)
                                        << canvas_fow_pixel_size_offset;

                                    pixmap.fill_rect(
                                        Rect::from_xywh(
                                            x as f32,
                                            y as f32,
                                            (1 << canvas_fow_pixel_size_offset) as f32,
                                            (1 << canvas_fow_pixel_size_offset) as f32,
                                        )
                                        .unwrap(),
                                        &paint,
                                        Transform::identity(),
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                // sub-tile rendering
                let fow_block_x_min = (tile_x & sub_tile_mask) << canvas_num_fow_block_offset;
                let fow_block_x_max = ((tile_x & sub_tile_mask) + 1) << canvas_num_fow_block_offset;
                let fow_block_y_min = (tile_y & sub_tile_mask) << canvas_num_fow_block_offset;
                let fow_block_y_max = ((tile_y & sub_tile_mask) + 1) << canvas_num_fow_block_offset;

                let canvas_fow_block_size_offset = CANVAS_SIZE_OFFSET - canvas_num_fow_block_offset;

                if let Some(tile) = fogmap.tiles.get(&(fow_tile_x, fow_tile_y)) {
                    for (key, block) in tile.blocks().iter() {
                        let (block_x, block_y) = *key;
                        if block_x >= fow_block_x_min
                            && block_x < fow_block_x_max
                            && block_y >= fow_block_y_min
                            && block_y < fow_block_y_max
                        {
                            let dx = (block_x - fow_block_x_min) << canvas_fow_block_size_offset;
                            let dy = (block_y - fow_block_y_min) << canvas_fow_block_size_offset;
                            Self::render_block_on_pixmap(
                                block,
                                &mut pixmap,
                                dx,
                                dy,
                                canvas_fow_block_size_offset,
                            );
                        }
                    }
                }
            }
        }

        pixmap
    }

    fn render_tile_on_pixmap(tile: &Tile, pixmap: &mut Pixmap, dx: u64, dy: u64, zoom: i16) {
        let overscan_offset = zoom - TILE_WIDTH_OFFSET;
        for (key, block) in tile.blocks().iter() {
            let (x, y) = key;
            let block_dx = dx + (*x as u64) << overscan_offset;
            let block_dy = dy + (*y as u64) << overscan_offset;
            Self::render_block_on_pixmap(block, pixmap, block_dx, block_dy, overscan_offset);
        }
    }

    fn render_block_on_pixmap(block: &Block, pixmap: &mut Pixmap, dx: u64, dy: u64, zoom: i16) {
        let mut paint = Paint::default();
        paint.set_color(Color::TRANSPARENT);

        if zoom <= 0 {
            pixmap.fill_rect(
                Rect::from_xywh(dx as f32, dy as f32, 1.0, 1.0).unwrap(),
                &paint,
                Transform::identity(),
                None,
            );
        } else {
            let overscan_offset = zoom - BITMAP_WIDTH_OFFSET;

            for x in 0..BITMAP_WIDTH {
                for y in 0..BITMAP_WIDTH {
                    if block.is_visited(x, y) {
                        // for each pixel of block, we may draw multiple pixel of image
                        pixmap.fill_rect(
                            Rect::from_xywh(
                                (dx + (x as u64) << overscan_offset) as f32,
                                (dy + (y as u64) << overscan_offset) as f32,
                                (1 << std::cmp::max(overscan_offset, 0)) as f32,
                                (1 << std::cmp::max(overscan_offset, 0)) as f32,
                            )
                            .unwrap(),
                            &paint,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
        }
    }
}
