use crate::renderer::renderer_cpu::FogRenderer;
use crate::renderer::renderer_gpu::FogRendererGpu;
use crate::utils;
use crate::FogMap;
use std::convert::TryInto;
use tiny_skia;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

use tiny_skia::IntSize;
use tokio::runtime::Runtime;

pub enum TileSize {
    TileSize256,
    TileSize512,
    TileSize1024,
}

impl TileSize {
    pub fn size(&self) -> u32 {
        match self {
            TileSize::TileSize256 => 256,
            TileSize::TileSize512 => 512,
            TileSize::TileSize1024 => 1024,
        }
    }

    pub fn power(&self) -> i16 {
        match self {
            TileSize::TileSize256 => 8,
            TileSize::TileSize512 => 9,
            TileSize::TileSize1024 => 10,
        }
    }
}

pub struct RenderResult {
    // coordinates are in lat or lng
    pub width: u32,
    pub height: u32,
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub data: Vec<u8>,
}

#[derive(PartialEq, Eq)]
struct RenderArea {
    zoom: i32,
    left_idx: i32,
    top_idx: i32,
    right_idx: i32,
    bottom_idx: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub lng: f64,
    pub lat: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct BBox {
    pub south_west: Point,
    pub north_east: Point,
}

#[allow(dead_code)]
pub struct RenderedTrackMap {
    track_map: FogMap,
    bg_color_prgba: tiny_skia::PremultipliedColorU8,
    fg_color_prgba: tiny_skia::PremultipliedColorU8,
    current_render_area: Option<RenderArea>,
    tile_size: TileSize,
    gpu_worker: Option<FogRendererGpu>,
}

#[allow(dead_code)]
impl RenderedTrackMap {
    pub fn new() -> Self {
        let track_map = FogMap::new();
        Self::new_with_track_map(track_map)
    }

    pub fn new_with_track_map(track_map: FogMap) -> Self {
        let opacity = 0.5;
        let alpha = (opacity * 255.0) as u8;
        let bg_color_prgba = tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, alpha).unwrap();
        let fg_color_prgba = tiny_skia::PremultipliedColorU8::TRANSPARENT;

        Self {
            track_map,
            bg_color_prgba,
            fg_color_prgba,
            current_render_area: None,
            tile_size: TileSize::TileSize512,
            gpu_worker: None,
        }
    }

    pub fn set_fg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.fg_color_prgba = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
        self.clear_buffer();
    }

    pub fn set_bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.bg_color_prgba = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
        self.clear_buffer();
    }

    pub fn set_tile_size(&mut self, tile_size: TileSize) {
        self.tile_size = tile_size;
        self.clear_buffer();
        // if gpu is used, we need to re-create the gpu worker
        if let Some(_) = &self.gpu_worker {
            self.set_use_gpu(true);
        }
    }

    pub fn set_use_gpu(&mut self, use_gpu: bool) {
        let tile_size = self.tile_size.size();
        if use_gpu {
            let gpu_worker = Runtime::new()
                .unwrap()
                .block_on(async move { FogRendererGpu::new(tile_size, tile_size).await });
            self.gpu_worker = Some(gpu_worker);
        } else {
            self.gpu_worker = None;
        }
        self.clear_buffer();
    }

    pub fn clear_buffer(&mut self) {
        self.current_render_area = None;
    }

    /// render a rectangle area of (multiple) map tiles. used for onetime rendering of the map on some divices.
    /// The area is usually larger than the display area to prevent flashing during
    /// zooming and panning.
    fn render_region_containing_bbox(&self, render_area: &RenderArea) -> RenderResult {
        // TODO: Change render backend. Right now we are using `tiny-skia`,
        // it should work just fine and we don't really need fancy features.
        // However, it is mostly a research project and does not feel like production ready,
        // `rust-skia` looks a lot better and has better performance (unlike `tiny-skia` is
        // purely on CPU, `rust-skia` can be ran on GPU). The reason we use `tiny-skia` right
        // now is that it is pure rust, so we don't need to think about how to build depenceies
        // for various platform.

        // TODO: make tile renderer static
        let mut tile_renderer = FogRenderer::new();
        tile_renderer.set_fg_color(
            self.fg_color_prgba.demultiply().red(),
            self.fg_color_prgba.demultiply().green(),
            self.fg_color_prgba.demultiply().blue(),
            self.fg_color_prgba.demultiply().alpha(),
        );
        tile_renderer.set_bg_color(
            self.bg_color_prgba.demultiply().red(),
            self.bg_color_prgba.demultiply().green(),
            self.bg_color_prgba.demultiply().blue(),
            self.bg_color_prgba.demultiply().alpha(),
        );

        tile_renderer.set_tile_size_power(self.tile_size.power());
        let tile_size: u32 = self.tile_size.size();

        let width_by_tile: u32 = (render_area.right_idx - render_area.left_idx + 1)
            .try_into()
            .unwrap();
        let height_by_tile: u32 = (render_area.bottom_idx - render_area.top_idx + 1)
            .try_into()
            .unwrap();

        // TODO: reuse resurces?
        let mut pixmap =
            Pixmap::new(tile_size * width_by_tile, tile_size * height_by_tile).unwrap();
        // color must be set to the tile renderer directly upon its creation

        for x in 0..width_by_tile {
            for y in 0..height_by_tile {
                // TODO: cache?

                let tile_pixmap = tile_renderer.render_pixmap(
                    &self.track_map,
                    render_area.left_idx as i64 + x as i64,
                    render_area.top_idx as i64 + y as i64,
                    render_area.zoom as i16,
                );

                debug_assert_eq!(tile_pixmap.width(), tile_size);
                debug_assert_eq!(tile_pixmap.height(), tile_size);

                println!("processing tile pixmap`");

                let processed_tile_pixmap_data = {
                    if let Some(gpu_worker) = &self.gpu_worker {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(gpu_worker.process_frame_async(tile_pixmap.data()))
                    } else {
                        tile_pixmap.data().to_vec()
                    }
                };

                // println!("output_image_data length: {}", output_image_data.len());
                let processed_tile_pixmap = Pixmap::from_vec(
                    processed_tile_pixmap_data,
                    IntSize::from_wh(tile_size, tile_size).unwrap(),
                )
                .unwrap();

                pixmap.draw_pixmap(
                    (x * tile_size) as i32,
                    (y * tile_size) as i32,
                    processed_tile_pixmap.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }

        let (overlay_left, overlay_top) =
            utils::tile_x_y_to_lng_lat(render_area.left_idx, render_area.top_idx, render_area.zoom);

        let (overlay_right, overlay_bottom) = utils::tile_x_y_to_lng_lat(
            render_area.right_idx + 1,
            render_area.bottom_idx + 1,
            render_area.zoom,
        );

        RenderResult {
            width: pixmap.width(),
            height: pixmap.height(),
            top: overlay_top,
            left: overlay_left,
            right: overlay_right,
            bottom: overlay_bottom,
            data: pixmap.encode_png().unwrap(),
        }
    }

    /// Render a region containing a bounding box.
    pub fn try_render_region_containing_bbox(
        &mut self,
        bbox: BBox,
        zoom: i32,
    ) -> Option<RenderResult> {
        // TODO: This doesn't really work when antimeridian is involved, see
        // the upstream issue: https://github.com/maplibre/maplibre-native/issues/1681
        let mut left_idx = utils::lng_to_tile_x(bbox.south_west.lng, zoom as i16) as i32;
        let mut top_idx = utils::lat_to_tile_y(bbox.north_east.lat, zoom as i16) as i32;

        // TODO: idx use i32 or i64? zoom use i16 or i32?
        let mut right_idx = utils::lng_to_tile_x(bbox.south_west.lng, zoom as i16) as i32;
        let mut bottom_idx = utils::lat_to_tile_y(bbox.north_east.lat, zoom as i16) as i32;

        // TODO: There is a hack to make sure we always cover a bit bigger to
        // avoid the gap between user move to new area and drawing that area.
        let n = f64::powi(2.0, zoom.into()) as i32;
        top_idx = std::cmp::max(top_idx - 1, 0);
        bottom_idx = std::cmp::min(bottom_idx + 1, n - 1);
        left_idx -= 1;
        right_idx += 1;
        if (right_idx - left_idx).abs() >= n {
            left_idx = 0;
            right_idx = n - 1;
        } else {
            if left_idx < 0 {
                left_idx += n;
            }
            while right_idx < left_idx {
                right_idx += n;
            }
        }

        let render_area = RenderArea {
            zoom,
            left_idx,
            top_idx,
            right_idx,
            bottom_idx,
        };

        if let Some(previous_render_area) = &self.current_render_area {
            if previous_render_area == &render_area {
                return None;
            }
        }

        let render_result = self.render_region_containing_bbox(&render_area);
        self.current_render_area = Some(render_area);
        Some(render_result)
    }
}
