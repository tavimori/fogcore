use crate::renderer::TileRendererBasic;
use crate::renderer::TileRendererTrait;
use crate::utils;
use crate::utils::{TileSize, DEFAULT_BG_COLOR2, DEFAULT_FG_COLOR2};
use crate::FogMap;
use image::Rgba;
use image::RgbaImage;
use std::convert::TryInto;

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

pub struct RenderedTrackMap {
    track_map: FogMap,
    render_backend: Box<dyn TileRendererTrait>,
    bg_color: Rgba<u8>,
    fg_color: Rgba<u8>,
    current_render_area: Option<RenderArea>,
}

impl RenderedTrackMap {
    pub fn new() -> Self {
        let track_map = FogMap::new();
        Self::new_with_track_map(track_map)
    }

    pub fn new_with_track_map(track_map: FogMap) -> Self {
        let tile_size = TileSize::TileSize512;
        let render_backend = Box::new(TileRendererBasic::new(tile_size));
        Self {
            track_map,
            render_backend,
            bg_color: DEFAULT_BG_COLOR2,
            fg_color: DEFAULT_FG_COLOR2,
            current_render_area: None,
        }
    }

    pub fn set_fg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.fg_color = Rgba([r, g, b, a]);
        self.clear_buffer();
    }

    pub fn set_bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.bg_color = Rgba([r, g, b, a]);
        self.clear_buffer();
    }

    // for debug and testing purpose
    pub fn set_rendering_backend(&mut self, backend: Box<dyn TileRendererTrait>) {
        self.render_backend = backend;
        self.clear_buffer();
    }

    pub fn clear_buffer(&mut self) {
        self.current_render_area = None;
    }

    /// render a rectangle area of (multiple) map tiles. used for onetime rendering of the map on some divices.
    /// The area is usually larger than the display area to prevent flashing during
    /// zooming and panning.
    fn render_region_containing_bbox(&self, render_area: &RenderArea) -> RenderResult {
        // TODO: figure out if we need a skia-like backend for rendering, if so, we can use `rust-skia`
        // `rust-skia` looks good and has good performance (GPU friendly)

        let tile_size: u32 = self.render_backend.get_tile_size().size();

        let width_by_tile: u32 = (render_area.right_idx - render_area.left_idx + 1)
            .try_into()
            .unwrap();
        let height_by_tile: u32 = (render_area.bottom_idx - render_area.top_idx + 1)
            .try_into()
            .unwrap();

        let mut image = RgbaImage::new(tile_size * width_by_tile, tile_size * height_by_tile);

        for x in 0..width_by_tile {
            for y in 0..height_by_tile {
                // TODO: cache?

                self.render_backend.render_on_image(
                    &mut image,
                    x * tile_size,
                    y * tile_size,
                    &self.track_map,
                    render_area.left_idx as i64 + x as i64,
                    render_area.top_idx as i64 + y as i64,
                    render_area.zoom as i16,
                    self.bg_color,
                    self.fg_color,
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

        let image_png = utils::image_to_png_data(&image);

        RenderResult {
            width: tile_size * width_by_tile,
            height: tile_size * height_by_tile,
            top: overlay_top,
            left: overlay_left,
            right: overlay_right,
            bottom: overlay_bottom,
            data: image_png,
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
