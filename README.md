# WGPU propero challenge

See Matt Keeter's [propero challenge](https://www.mattkeeter.com/projects/prospero/) for details.

## Challenge entry

The `prospero` example converts `propero.vm` into a WGSL shader and then evaluate distance values in the XY plane using `shaders/sample_plane.wgsl`.
Workgroups size is set 16x16 and is not optimized yet.
To run this example, use the following command.

```sh
cargo run --example prospero --release
```

## Sample timings

Here a few sample timings for the few machines I could test it on.

```
AdapterInfo {
    name: "Intel(R) Iris(R) Xe Graphics",
    vendor: 32902,
    device: 18086,
    device_type: IntegratedGpu,
    driver: "Intel Corporation",
    driver_info: "101.6078",
    backend: Vulkan,
}
SecondDurations {
    init: 1.5430348,
    generate: 0.0068996,
    compile: 0.0259545,
    alloc: 0.0002998,
    eval: 0.0021519,
    retrieve: 0.0378382,
    export: 0.0038969,
    total: 1.6200757,
}
```
