//! A library for rendering a person's living tracks.
//!
//! The library current supports importing data from the `Fog of World` App, and render it into pixmaps using `tiny-skia` library.
//!
//! # Usage
//! Please refer to the `examples` folder.

pub mod fogmaps;
pub mod renderer;
mod utils;
pub mod wasm;

pub use fogmaps::FogMap;
pub use renderer::FogRenderer;