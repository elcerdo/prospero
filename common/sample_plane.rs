use super::Common;

pub struct SamplePlane<'a> {
    common: &'a Common,
    module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
}

impl<'a> SamplePlane<'a> {
    pub fn from_common_with_sdf_source(common: &'a Common, sdf_source: &str) -> Self {
        use wgpu::*;

        // Shader module
        let module = common.load_wgsl_with_sdf_source(sdf_source, "shaders/sample_plane.wgsl");

        // Bind group layout
        let bind_group_layout =
            common
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // distance buffer
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // Pipeline layout
        let pipeline_layout = common
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        Self {
            common,
            module,
            bind_group_layout,
            pipeline_layout,
        }
    }

    pub fn run(&self, distance_buffer: &wgpu::Buffer, image_size: usize) {
        // Bind group
        let bind_group = self
            .common
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: distance_buffer.as_entire_binding(),
                }],
            });

        // Pipeline
        let pipeline =
            self.common
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&self.pipeline_layout),
                    module: &self.module,
                    entry_point: Some("kernel_plane_distance"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        let mut encoder = self
            .common
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            assert!(image_size != 0);
            assert!(image_size % 16 == 0);
            let dispatch_xx = image_size / 16;
            let dispatch_yy = dispatch_xx;
            compute_pass.dispatch_workgroups(dispatch_xx as u32, dispatch_yy as u32, 1);
        }

        self.common.queue.submit([encoder.finish()]);
    }
}
