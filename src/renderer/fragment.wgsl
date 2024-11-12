struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) color_bg: vec4<f32>,
    @location(2) point_coord: vec2<f32>,
};

@fragment
fn main(input: FragmentInput) -> @location(0) vec4<f32> {
    let border = 0.1;
    let radius = 0.5;
    
    // Calculate distance from center
    let coord = input.point_coord - vec2<f32>(0.5);
    let dist = radius - length(coord);

    // Calculate blend factor
    var t = 0.0;
    if (dist > border) {
        t = 1.0;
    } else if (dist > 0.0) {
        t = dist / border;
    }

    // Mix colors based on distance
    return mix(input.color_bg, input.color, t);
}