mod affine;
mod sample_sdf;
mod sample_plane;

pub use affine::Affine;
pub use sample_sdf::SampleDistance;
pub use sample_plane::SamplePlane;

use bytemuck::Pod;

use log::debug;
use log::info;

pub struct Common {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Common {
    pub fn default() -> Self {
        use wgpu::*;

        // Create instance and adapter
        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .expect("Failed to create adapter");
        info!("adapter {:#?}", adapter.get_info());
        if !adapter
            .get_downlevel_capabilities()
            .flags
            .contains(DownlevelFlags::COMPUTE_SHADERS)
        {
            panic!("Adapter does not support compute shaders");
        }

        // Create device and queue
        let required_features = Features::SHADER_INT64;
        // | Features::SHADER_INT64_ATOMIC_MIN_MAX; // fails on Intel(R) Iris(R) Xe Graphics
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features,
                required_limits: Limits::downlevel_defaults(),
                memory_hints: MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to create device");

        Self {
            adapter,
            device,
            queue,
        }
    }

    pub fn load_wgsl_with_sdf_source(&self, sdf_source: &str, shader_path: &str) -> wgpu::ShaderModule {
        use wgpu::*;

        let source = [
            sdf_source.into(),
            std::fs::read_to_string(shader_path).expect("Can't load shader"),
        ]
        .join("\n////////////\n\n");

        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(std::borrow::Cow::Owned(source)),
        });

        module
    }

    pub fn load_wgsl(&self, path: &str) -> wgpu::ShaderModule {
        use wgpu::*;

        let source = std::fs::read_to_string(path).expect("Can't load shader");

        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(std::borrow::Cow::Owned(source)),
        });

        module
    }

    pub fn load_wgsl_with_sdf(&self, sdf_path: &str, shader_path: &str) -> wgpu::ShaderModule {
        use wgpu::*;

        let source = [
            std::fs::read_to_string(sdf_path).expect("Can't load sdf"),
            std::fs::read_to_string(shader_path).expect("Can't load shader"),
        ]
        .join("\n////////////\n\n");

        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(std::borrow::Cow::Owned(source)),
        });

        module
    }

    pub fn retrieve_buffer<Scalar: Clone + Pod>(&self, buffer: &wgpu::Buffer) -> Vec<Scalar> {
        let buffer_ = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.copy_buffer_to_buffer(&buffer, 0, &buffer_, 0, buffer.size());

        self.queue.submit([encoder.finish()]);

        // We now map the download buffer so we can read it. Mapping tells wgpu that we want to read/write
        // to the buffer directly by the CPU and it should not permit any more GPU operations on the buffer.
        //
        // Mapping requires that the GPU be finished using the buffer before it resolves, so mapping has a callback
        // to tell you when the mapping is complete.
        let buffer_slice_ = buffer_.slice(..);
        buffer_slice_.map_async(wgpu::MapMode::Read, |res| {
            debug!("mapped slice {:?}", res);
        });

        // Wait for the GPU to finish working on the submitted work. This doesn't work on WebGPU, so we would need
        // to rely on the callback to know when the buffer is mapped.
        self.device.poll(wgpu::Maintain::Wait);

        let buffer_view_ = buffer_slice_.get_mapped_range();
        let buffer_view_ = bytemuck::cast_slice(&buffer_view_);

        // Vec::from_iter(buffer_view_.into_iter())
        buffer_view_.to_vec()
    }
}
