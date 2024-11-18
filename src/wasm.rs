use crate::fogmaps::FogMap as FogMapNative;
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
    renderer: TileRendererPremium2,
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
                renderer,
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
    pub async fn render_image(&self, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let image = self
            .renderer
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
