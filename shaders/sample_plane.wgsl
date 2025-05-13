// sample sdf kernel

@group(0) @binding(0)
var<storage, read_write> distances: array<f32>;

@compute @workgroup_size(16, 16)
fn kernel_plane_distance(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let image_size = num_workgroups.x * 16;
    let index = global_id.x + image_size * global_id.y;

    var position = vec3<f32>(global_id) / f32(image_size - 1) * 2.0 - vec3(1.0, 1.0, 0.0);
    position.y *= - 1.0;
    position.z = 0.0;
    let distance = signed_distance_function(position);

    distances[index] = distance;
}