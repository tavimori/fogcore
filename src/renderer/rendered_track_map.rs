use crate::renderer::renderer_cpu::FogRenderer;
use crate::utils;
use crate::FogMap;
use std::convert::TryInto;
use tiny_skia;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

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

pub struct Point {
    pub lng: f64,
    pub lat: f64,
}

pub struct BBox {
    pub south_west: Point,
    pub north_east: Point,
}

pub struct RenderedTrackMap {
    use_gpu: bool,
    track_map: FogMap,
    bg_color_prgba: tiny_skia::PremultipliedColorU8,
    fg_color_prgba: tiny_skia::PremultipliedColorU8,
    current_render_area: Option<RenderArea>,
}

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
            use_gpu: false,
            track_map,
            bg_color_prgba,
            fg_color_prgba,
            current_render_area: None,
        }
    }

    pub fn set_fg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.fg_color_prgba = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
        // TODO: clear buffer
    }

    pub fn set_bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.bg_color_prgba = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
        // TODO: clear buffer
    }

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

        // TODO: design this interface?
        const TILE_SIZE_POWER: i16 = 10;

        let tile_size: u32 = 1 << TILE_SIZE_POWER;
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

                pixmap.draw_pixmap(
                    (x * tile_size) as i32,
                    (y * tile_size) as i32,
                    tile_pixmap.as_ref(),
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

    fn try_render_region_containing_bbox(&mut self, bbox: BBox, zoom: i32) -> Option<RenderResult> {
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
