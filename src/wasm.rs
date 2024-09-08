use crate::fogmaps::FogMap as FogMapNative;
use crate::renderer::FogRenderer as FogRendererNative;
use crate::gpu::GpuFogRenderer as GpuFogRendererNative;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use wasm_bindgen_futures::future_to_promise;
use tiny_skia::{Pixmap, IntSize};
use js_sys::{Uint8Array, Promise};

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

    #[wasm_bindgen]
    pub fn render_and_blur_image(&self, fogmap: &FogMap, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();
        let mut pixmap = self
        .render
        .render_pixmap(fogmap_native_ref, view_x, view_y, zoom);

        // Apply blur filter
        self.apply_blur_filter(&mut pixmap);

        // Encode the blurred image to PNG
        pixmap.encode_png().unwrap()
    }

    fn apply_blur_filter(&self, pixmap: &mut Pixmap) {
        let kernel: [f32; 9] = [
            1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0,
            2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0,
            1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0
        ];

        let width = pixmap.width();
        let height = pixmap.height();
        let mut new_data = pixmap.data().to_vec();

        for y in 1..height-1 {
            for x in 1..width-1 {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;

                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = x as i32 + kx - 1;
                        let py = y as i32 + ky - 1;
                        let weight = kernel[(ky * 3 + kx) as usize];

                        let idx = (py * width as i32 + px) as usize * 4;
                        r += pixmap.data()[idx] as f32 * weight;
                        g += pixmap.data()[idx + 1] as f32 * weight;
                        b += pixmap.data()[idx + 2] as f32 * weight;
                        a += pixmap.data()[idx + 3] as f32 * weight;
                    }
                }

                let idx = (y * width + x) as usize * 4;
                new_data[idx] = r as u8;
                new_data[idx + 1] = g as u8;
                new_data[idx + 2] = b as u8;
                new_data[idx + 3] = a as u8;
            }
        }

        *pixmap = Pixmap::from_vec(new_data, IntSize::from_wh(width, height).unwrap()).unwrap();
    }
}

#[wasm_bindgen]
pub struct GpuFogRenderer {
    renderer: FogRendererNative,
    gpu_renderer: GpuFogRendererNative,
}

#[wasm_bindgen]
impl GpuFogRenderer {
    // #[wasm_bindgen(constructor)]
    pub fn create(width: u32, height: u32) -> Promise {
        future_to_promise(async move {
            let gpu_renderer = GpuFogRendererNative::new(width, height).await;
            Ok(Self { 
                renderer: FogRendererNative::new(),
                gpu_renderer }.into())
        })
    }

    #[wasm_bindgen]
    pub fn render_image(&self, fogmap: &FogMap, view_x: i64, view_y: i64, zoom: i16) -> Vec<u8> {
        let fogmap_native_ref = fogmap.get_native_fogmap_ref();
        let image_pix = self
            .renderer
            .render_pixmap(fogmap_native_ref, view_x, view_y, zoom);
        log("passing into gpu renderer");
        log(&format!("passing gpu image length: {}", image_pix.data().len()));
        let gpu_image = self.gpu_renderer.process_frame(image_pix.data());
        log(&format!("got gpu image length: {}", gpu_image.len()));
        let image = Pixmap::from_vec(
            gpu_image, 
            IntSize::from_wh(image_pix.width(), image_pix.height()).unwrap()
        ).unwrap().encode_png().unwrap();
        image
    }
}