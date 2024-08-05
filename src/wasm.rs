use crate::fogmaps::FogMap as FogMapNative;
use crate::renderer::FogRenderer as FogRendererNative;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn lng_to_tile_x(lng: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let x = (lng + 180.0) / 360.0 * mul;
    x as i64
}

#[wasm_bindgen]
pub fn lat_to_tile_y(lat: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let y = (PI - (lat * PI / 180.0).tan().asinh()) * mul / (2.0 * PI);
    y as i64
}

#[wasm_bindgen]
pub struct FogMap {
    fogmap: FogMapNative,
}

#[wasm_bindgen]
impl FogMap {
    // #[wasm_bindgen]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        println!("creating fog map");
        Self {
            fogmap: FogMapNative::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_fow_file(&mut self, file_name: &str, data: &[u8]) {
        // log(&format!("adding file {} with length {}", file_name, data.len()));
        let data_clone = data.to_vec();
        self.fogmap.add_fow_file(file_name, data_clone);
    }

    fn get_native_fogmap_ref(&self) -> &FogMapNative {
        return &self.fogmap;
    }
}

#[wasm_bindgen]
pub struct FogRenderer {
    render: FogRendererNative,
}

#[wasm_bindgen]
impl FogRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            render: FogRendererNative::new(),
        }
    }

    #[wasm_bindgen]
    pub fn render_image(&self, fogmap: &FogMap, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();
        let image = self
            .render
            .render_pixmap(fogmap_native_ref, view_x, view_y, zoom)
            .encode_png()
            .unwrap();
        image
    }
}
