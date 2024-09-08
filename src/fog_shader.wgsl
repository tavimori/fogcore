@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.y * 1024u + global_id.x;
    
    // Simply copy the input to the output
    output[index] = input[index];
}