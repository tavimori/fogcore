use wasm_bindgen::prelude::*;
use crate::fogmaps::FogMap as FogMapNative;
use crate::renderer::FogRenderer as FogRendererNative;
extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct FogMap {
    fogmap: FogMapNative
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
}


#[wasm_bindgen]
pub struct FogRenderer {
    render: FogRendererNative
}

#[wasm_bindgen]
impl FogRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            render: FogRendererNative::new(),
        }
    }
}