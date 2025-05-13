use super::Common;

use wgpu::util::DeviceExt;

pub struct Affine<'a> {
    common: &'a Common,
    module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    settings_buffer: wgpu::Buffer,
}

impl<'a> Affine<'a> {
    pub fn from_common(common: &'a Common) -> Self {
        use wgpu::*;

        // Shader module
        let module = common.load_wgsl("shaders/affine.wgsl");

        // Bind group layout
        let bind_group_layout =
            common
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        // input buffer
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // output buffer
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // settings buffer
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
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

        // Settings buffer
        let settings: [u64; 2] = [2, 100];
        let settings_buffer = common
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[settings]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        Self {
            common,
            module,
            bind_group_layout,
            pipeline_layout,
            settings_buffer,
        }
    }

    pub fn run(
        &self,
        input_buffer: &wgpu::Buffer,
        output_buffer: &wgpu::Buffer,
        input_size: usize,
    ) {
        // Bind group
        let bind_group = self
            .common
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.settings_buffer.as_entire_binding(),
                    },
                ],
            });

        // Pipeline
        let pipeline =
            self.common
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&self.pipeline_layout),
                    module: &self.module,
                    entry_point: Some("kernel_double"),
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
            assert!(input_size != 0);
            let dispatch_max = 1 << 15;
            let dispatch_xx = if input_size < dispatch_max {
                input_size
            } else {
                dispatch_max
            };
            let dispatch_yy = if input_size < dispatch_max {
                1
            } else {
                input_size / dispatch_max + 1
            };
            compute_pass.dispatch_workgroups(dispatch_xx as u32, dispatch_yy as u32, 1);
        }

        self.common.queue.submit([encoder.finish()]);
    }
}
