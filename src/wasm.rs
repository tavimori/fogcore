use crate::fogmaps::FogMap as FogMapNative;
use crate::renderer::tile_shader2::TileShader2;
use crate::renderer::TileRendererPremium2;
use crate::utils::DEFAULT_TILE_SIZE;

use crate::utils::{DEFAULT_BG_COLOR2, DEFAULT_FG_COLOR2};
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use js_sys::Promise;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct FogMap {
    fogmap: FogMapNative,
    renderer: Option<TileRendererPremium2>,
}

#[wasm_bindgen]
impl FogMap {
    #[wasm_bindgen]
    pub fn new() -> Promise {
        console_error_panic_hook::set_once();
        future_to_promise(async move {
            let renderer = TileRendererPremium2::new_async(DEFAULT_TILE_SIZE).await;
            Ok(Self {
                fogmap: FogMapNative::new(),
                renderer: Some(renderer),
            }
            .into())
        })
    }

    #[wasm_bindgen]
    pub fn new_no_renderer() -> Promise {
        console_error_panic_hook::set_once();
        future_to_promise(async move {
            Ok(Self {
                fogmap: FogMapNative::new(),
                renderer: None,
            }
            .into())
        })
    }

    #[wasm_bindgen]
    pub fn add_fow_file(&mut self, file_name: &str, data: &[u8]) {
        // log(&format!("adding file {} with length {}", file_name, data.len()));
        let data_clone = data.to_vec();
        self.fogmap.add_fow_file(file_name, data_clone);
    }

    #[wasm_bindgen]
    pub fn add_fow_zip(&mut self, data: &[u8]) {
        self.fogmap.add_fow_zip(data).unwrap();
    }

    // TODO: use the correct zoom level
    #[wasm_bindgen]
    pub fn get_bounding_mercator_pixels(
        &self,
        sw_x: f64,
        sw_y: f64,
        ne_x: f64,
        ne_y: f64,
    ) -> Vec<f32> {
        // TODO: check cross-antimeridian
        // log_print!("sw_x: {}, sw_y: {}, ne_x: {}, ne_y: {}", sw_x, sw_y, ne_x, ne_y);

        let x_min = if sw_x < ne_x { sw_x } else { ne_x };
        let x_max = if sw_x > ne_x { sw_x } else { ne_x };
        let y_min = if sw_y < ne_y { sw_y } else { ne_y };
        let y_max = if sw_y > ne_y { sw_y } else { ne_y };

        let x_diff = x_max - x_min;
        let y_diff = y_max - y_min;

        let diff_min = if x_diff < y_diff { x_diff } else { y_diff };

        // log_print!("diff_min: {}", diff_min);

        const MAX_ZOOM: i16 = 24; // Common max zoom level for web maps
        let mut zoom_level: i16 = 0;
        while zoom_level < MAX_ZOOM && diff_min * (1 << zoom_level) as f64 <= 1.0 {
            zoom_level += 1;
        }

        let zoom_coefficient = (1 << zoom_level) as f64;

        let x_min = (x_min * zoom_coefficient) as i64;
        let y_min = (y_min * zoom_coefficient) as i64;
        let x_max = (x_max * zoom_coefficient) as i64;
        let y_max = (y_max * zoom_coefficient) as i64;

        let buffer_size_power = 10;

        let mut pixels = Vec::new();
        for x in x_min..(x_max + 1) {
            for y in y_min..(y_max + 1) {
                let x_mercator = x as f64 / zoom_coefficient as f64;
                let y_mercator = y as f64 / zoom_coefficient as f64;
                let temp = TileShader2::get_pixels_coordinates(
                    0,
                    0,
                    &self.fogmap,
                    x,
                    y,
                    zoom_level,
                    buffer_size_power,
                );
                for (i, &value) in temp.iter().step_by(2).enumerate() {
                    pixels.push(
                        value / (1 << buffer_size_power) as f32 / zoom_coefficient as f32
                            + x_mercator as f32,
                    );
                    pixels.push(
                        temp[i * 2 + 1] / (1 << buffer_size_power) as f32 / zoom_coefficient as f32
                            + y_mercator as f32,
                    );
                }
            }
        }
        pixels
    }

    #[wasm_bindgen]
    pub async fn render_image(&self, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let image = self
            .renderer
            .as_ref()
            .unwrap()
            .render_image_async(
                &self.fogmap,
                view_x,
                view_y,
                zoom,
                DEFAULT_BG_COLOR2,
                DEFAULT_FG_COLOR2,
            )
            .await;
        image.into_vec()
    }
}
