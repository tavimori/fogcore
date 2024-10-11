use crate::fogmaps::FogMap as FogMapNative;
use crate::renderer::FogRendererGpu as FogRendererGpuNative;
use crate::renderer::TileShader;

use crate::utils::{DEFAULT_BG_COLOR, DEFAULT_FG_COLOR};
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use crate::log_print;
use js_sys::{Array, Promise, Uint8Array};
use tiny_skia::{IntSize, Pixmap};
use wasm_bindgen_futures::future_to_promise;

const WEB_DEFAULT_TILE_SIZE_POWER: i16 = 10;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
pub struct FogRenderer {}

#[wasm_bindgen]
impl FogRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[wasm_bindgen]
    pub fn render_image(&self, fogmap: &FogMap, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();
        let image = TileShader::render_pixmap(
            fogmap_native_ref,
            view_x,
            view_y,
            zoom,
            WEB_DEFAULT_TILE_SIZE_POWER,
            DEFAULT_BG_COLOR,
            DEFAULT_FG_COLOR,
        )
        .encode_png()
        .unwrap();
        image
    }

    #[wasm_bindgen]
    pub fn render_image_raw(
        &self,
        fogmap: &FogMap,
        view_x: i64,
        view_y: i64,
        zoom: i16,
    ) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();

        let image = TileShader::render_pixmap(
            fogmap_native_ref,
            view_x,
            view_y,
            zoom,
            WEB_DEFAULT_TILE_SIZE_POWER,
            DEFAULT_BG_COLOR,
            DEFAULT_FG_COLOR,
        );
        let data = image.data().to_vec();
        data
    }

    #[wasm_bindgen]
    pub fn render_and_blur_image(
        &self,
        fogmap: &FogMap,
        view_x: i64,
        view_y: i64,
        zoom: i16,
    ) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();
        let mut pixmap = TileShader::render_pixmap(
            fogmap_native_ref,
            view_x,
            view_y,
            zoom,
            WEB_DEFAULT_TILE_SIZE_POWER,
            DEFAULT_BG_COLOR,
            DEFAULT_FG_COLOR,
        );

        // Apply blur filter
        self.apply_alpha_dilation(&mut pixmap);

        // Encode the blurred image to PNG
        pixmap.encode_png().unwrap()
    }

    fn apply_alpha_dilation(&self, pixmap: &mut Pixmap) {
        let width = pixmap.width();
        let height = pixmap.height();
        let mut new_data = pixmap.data().to_vec();

        // 5x5 kernel for distance-based weight
        let kernel: [[f32; 5]; 5] = [
            [0.3, 0.5, 0.7, 0.5, 0.3],
            [0.5, 1.0, 1.0, 1.0, 0.5],
            [0.7, 1.0, 1.0, 1.0, 0.7],
            [0.5, 1.0, 1.0, 1.0, 0.5],
            [0.3, 0.5, 0.7, 0.5, 0.3],
        ];

        for y in 2..height - 2 {
            for x in 2..width - 2 {
                let mut min_alpha = 256.0;
                let mut max_r = 0;
                let mut max_g = 0;
                let mut max_b = 0;
                let self_idx = (y * width + x) as usize * 4;
                let self_alpha = pixmap.data()[self_idx + 3] as f32;

                for ky in 0..5 {
                    for kx in 0..5 {
                        let px = x as i32 + kx - 2;
                        let py = y as i32 + ky - 2;
                        let weight = kernel[ky as usize][kx as usize];

                        let idx = (py * width as i32 + px) as usize * 4;
                        let alpha =
                            pixmap.data()[idx + 3] as f32 * weight + self_alpha * (1.0 - weight);

                        if alpha < min_alpha {
                            min_alpha = alpha;
                            max_r = pixmap.data()[idx];
                            max_g = pixmap.data()[idx + 1];
                            max_b = pixmap.data()[idx + 2];
                        }
                    }
                }

                let idx = (y * width + x) as usize * 4;
                new_data[idx] = max_r;
                new_data[idx + 1] = max_g;
                new_data[idx + 2] = max_b;
                new_data[idx + 3] = min_alpha as u8;
            }
        }

        *pixmap = Pixmap::from_vec(new_data, IntSize::from_wh(width, height).unwrap()).unwrap();
    }
}

#[wasm_bindgen]
pub struct GpuFogRenderer {
    gpu_renderer: FogRendererGpuNative,
}

#[wasm_bindgen]
impl GpuFogRenderer {
    // #[wasm_bindgen(constructor)]
    pub fn create(width: u32, height: u32) -> Promise {
        future_to_promise(async move {
            let gpu_renderer = FogRendererGpuNative::new(width, height).await;
            Ok(Self {
                gpu_renderer,
            }
            .into())
        })
    }

    #[wasm_bindgen]
    pub fn render_image(
        &self,
        fogmap: &FogMap,
        view_x: i64,
        view_y: i64,
        zoom: i16,
        callback: js_sys::Function,
    ) {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();

        let image_pix = TileShader::render_pixmap(
            fogmap_native_ref,
            view_x,
            view_y,
            zoom,
            WEB_DEFAULT_TILE_SIZE_POWER,
            DEFAULT_BG_COLOR,
            DEFAULT_FG_COLOR,
        );

        log("passing into gpu renderer");
        log(&format!(
            "passing gpu image length: {}",
            image_pix.data().len()
        ));

        let closure = move |v: Vec<u8>| {
            log_print!("From closure of length {}", v.len());

            // FIXME: this is a hack to make the image the correct size
            let img = Pixmap::from_vec(v, IntSize::from_wh(1024, 1024).unwrap())
                .unwrap()
                .encode_png()
                .unwrap();

            let js_array = Uint8Array::new_with_length((img.len()) as u32);
            js_array.copy_from(&img);

            // Create a JS Array to pass as arguments
            let args = Array::new();
            args.push(&js_array.into());

            // Call the JavaScript function with the arguments
            let _ = callback.apply(&JsValue::NULL, &args);
        };

        self.gpu_renderer
            .process_frame(image_pix.data(), Box::new(closure));
    }
}
