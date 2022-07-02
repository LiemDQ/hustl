use std::borrow::Cow;

pub struct Background {
    render_pipeline: wgpu::RenderPipeline,
}

impl Background {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> Self {

        let shader = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("bg.wgsl"))),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }]
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState{
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            }
        );

        Self { 
            render_pipeline
        }
    }

    pub fn draw(&self, frame: &wgpu::SurfaceTexture,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError> {
        //create a TextureView with default settings. This controls how the render code interacts with the texture.
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: true,
                        }
                    }],
                    depth_stencil_attachment: Some (
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.0),
                                store: true
                            }),
                            stencil_ops: None,
                        }
                    )
                }
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }
        

        Ok(())
    }
}