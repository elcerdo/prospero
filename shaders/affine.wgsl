// affine compute shader

struct Settings {
    scale: u64,
    offset: u64,
}

@group(0) @binding(0)
var<storage, read> input: array<u64>;
@group(0) @binding(1)
var<storage, read_write> output: array<u64>;
@group(0) @binding(2)
var<uniform> settings: Settings;

const DISPATCH_MAX: u32 = 1 << 15;

@compute @workgroup_size(1)
fn kernel_double(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x + global_id.y * DISPATCH_MAX;

    let array_length = arrayLength(&input);
    if index >= array_length {
        return;
    }

    output[index] = input[index] * settings.scale + settings.offset;
}