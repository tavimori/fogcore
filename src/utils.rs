use std::f64::consts::PI;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}


#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lng_to_tile_x(lng: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let x = (lng + 180.0) / 360.0 * mul;
    x as i64
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lat_to_tile_y(lat: f64, zoom: i16) -> i64 {
    let mul = (1 << zoom) as f64;
    let y = (PI - (lat * PI / 180.0).tan().asinh()) * mul / (2.0 * PI);
    y as i64
}