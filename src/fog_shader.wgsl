@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

const KERNEL: array<array<f32, 5>, 5> = array<array<f32, 5>, 5>(
    array<f32, 5>(0.3, 0.5, 0.7, 0.5, 0.3),
    array<f32, 5>(0.5, 1.0, 1.0, 1.0, 0.5),
    array<f32, 5>(0.7, 1.0, 1.0, 1.0, 0.7),
    array<f32, 5>(0.5, 1.0, 1.0, 1.0, 0.5),
    array<f32, 5>(0.3, 0.5, 0.7, 0.5, 0.3)
);

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = 1024u;  // Assuming width is 1024, adjust as needed
    let height = 1024u; // Assuming height is 1024, adjust as needed
    let x = global_id.x;
    let y = global_id.y;

    // TODO: a better way to handle border pixels
    // Skip border pixels
    if (x < 2u || x >= width - 2u || y < 2u || y >= height - 2u) {
        return;
    }

    let self_idx = y * width + x;
    let self_alpha = f32(input[self_idx] >> 24u) / 255.0;
    var min_alpha = 1.0;
    var max_r = 0u;
    var max_g = 0u;
    var max_b = 0u;

    for (var ky = 0u; ky < 5u; ky++) {
        for (var kx = 0u; kx < 5u; kx++) {
            let px = i32(x) + i32(kx) - 2;
            let py = i32(y) + i32(ky) - 2;
            let weight = KERNEL[ky][kx];

            let idx = u32(py) * width + u32(px);
            let pixel = input[idx];
            let alpha = f32(pixel >> 24u) / 255.0 * weight + self_alpha * (1.0 - weight);

            if (alpha < min_alpha) {
                min_alpha = alpha;
                max_r = pixel & 0xFFu;
                max_g = (pixel >> 8u) & 0xFFu;
                max_b = (pixel >> 16u) & 0xFFu;
            }
        }
    }

    let result = (u32(min_alpha * 255.0) << 24u) | (max_b << 16u) | (max_g << 8u) | max_r;
    output[self_idx] = result;
}