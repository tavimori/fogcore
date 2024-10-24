use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fogcore::load_tracks_map_folder;
use fogcore::TileRendererBasic;
use fogcore::TileRendererPremium;
use fogcore::TileRendererTrait;
use fogcore::TileSize;
use fogcore::{lat_to_tile_y, lng_to_tile_x};
use std::hint::black_box;

fn benchmark_tile_rendering_basic(c: &mut Criterion) {
    let fogmap = load_tracks_map_folder("static/tiles");
    let bg_color = image::Rgba([100, 0, 100, 255]);
    let fg_color = image::Rgba([0, 0, 0, 0]);
    let lng = 114.1;
    let lat = 22.7;

    let mut group = c.benchmark_group("tile_rendering_basic");

    for zoom in [1, 5, 10, 15, 20] {
        for &resolution in &[
            TileSize::TileSize256,
            TileSize::TileSize512,
            TileSize::TileSize1024,
        ] {
            let renderer = TileRendererBasic::new(resolution);

            group.bench_function(
                BenchmarkId::new(format!("zoom_{:02}", zoom), format!("{:?}", resolution)),
                |b| {
                    b.iter(|| {
                        let tile_x = black_box(lng_to_tile_x(lng, zoom));
                        let tile_y = black_box(lat_to_tile_y(lat, zoom));

                        renderer.render_image(
                            &fogmap,
                            tile_x,
                            tile_y,
                            black_box(zoom),
                            bg_color,
                            fg_color,
                        )
                    })
                },
            );
        }
    }

    group.finish();
}

fn benchmark_tile_rendering_premium(c: &mut Criterion) {
    let fogmap = load_tracks_map_folder("static/tiles");
    let bg_color = image::Rgba([100, 0, 100, 255]);
    let fg_color = image::Rgba([0, 0, 0, 0]);
    let lng = 114.1;
    let lat = 22.7;

    let mut group = c.benchmark_group("tile_rendering_premium");

    for zoom in [1, 5, 10, 15, 20] {
        for &resolution in &[
            TileSize::TileSize256,
            TileSize::TileSize512,
            TileSize::TileSize1024,
        ] {
            let renderer = TileRendererPremium::new(resolution);

            group.bench_function(
                BenchmarkId::new(format!("zoom_{:02}", zoom), format!("{:?}", resolution)),
                |b| {
                    b.iter(|| {
                        let tile_x = black_box(lng_to_tile_x(lng, zoom));
                        let tile_y = black_box(lat_to_tile_y(lat, zoom));

                        renderer.render_image(
                            &fogmap,
                            tile_x,
                            tile_y,
                            black_box(zoom),
                            bg_color,
                            fg_color,
                        )
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tile_rendering_basic,
    benchmark_tile_rendering_premium
);
criterion_main!(benches);
