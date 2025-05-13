// sample sdf kernel

@group(0) @binding(0)
var<storage, read> positions: array<f32>;
@group(0) @binding(1)
var<storage, read_write> distances: array<f32>;

const DISPATCH_MAX: u32 = 1 << 15;

@compute @workgroup_size(64, 1)
fn kernel_sample_distance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x + global_id.y * DISPATCH_MAX;

    let array_length = arrayLength(&positions);
    if index * 3 + 2 >= array_length {
        return;
    }

    let position = vec3(positions[index * 3 + 0], positions[index * 3 + 1], positions[index * 3 + 2]);
    let distance = signed_distance_function(position);

    distances[index] = distance;
}