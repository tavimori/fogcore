struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) point_size: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) color_bg: vec4<f32>,
    @location(2) point_coord: vec2<f32>,
};

struct Uniforms {
    matrix: mat4x4<f32>,
    color: vec4<f32>,
    color_bg: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Set the position by multiplying with the transformation matrix
    output.position = uniforms.matrix * input.position;
    
    // Pass the colors to fragment shader
    output.color = uniforms.color;
    output.color_bg = uniforms.color_bg;
    
    // Pass point size as point coordinates
    output.point_coord = vec2<f32>(input.point_size);
    
    return output;
}