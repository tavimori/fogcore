//! A library for rendering a person's living tracks.
//!
//! The library current supports importing data from the `Fog of World` App, and render it into pixmaps using `tiny-skia` library.
//!
//! # Usage
//! Please refer to the `examples` folder.

pub mod fogmaps;
pub mod renderer;
mod utils;

#[cfg(feature = "wasm")]
pub mod wasm;

mod logging;

pub use renderer::TileShader;
pub use renderer::TileRendererBasic;
pub use renderer::TileRendererTrait;
pub use renderer::TileRendererPremium;

pub use fogmaps::FogMap;
pub use utils::*;

