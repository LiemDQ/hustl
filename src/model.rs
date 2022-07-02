use wgpu::util::DeviceExt;
use nalgebra_glm as glm;
use glm::{Vec3, Vec4, Mat4};
use crate::loader::Vertex;
use crate::camera::Camera;

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    num_indices: u32,
    num_vertices: u32,
}

impl Model {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, vertices: &[Vertex], indices: &[u32]) -> Self {
        
        let num_vertices = vertices.len() as u32;
        println!("{}", num_vertices);

        println!("{:?}", vertices);

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Model vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let camera_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Camera buffer"),
                size: std::mem::size_of::<glm::Mat4>() as wgpu::BufferAddress * 3,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None, 
                        },
                        count: None
                    },
                ],
                label: Some("Camera bind group layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("Camera bind group"),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Model render pipeline layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[]
            }
        );

        let vertex_buf_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //positions
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ]
        };

        let shader = device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some("Module shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("model.wgsl").into()), //TODO: embed this at compile time?
            }
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Model render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_buf_layout],
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill, //this may change depending on the settings
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        Self {
            render_pipeline,
            index_buffer,
            vertex_buffer,
            camera_buffer,
            camera_bind_group,
            num_vertices,
            num_indices: indices.len() as u32
        }

    }

    pub fn draw(&self, 
        camera: &Camera, 
        frame: &wgpu::SurfaceTexture, 
        depth_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder, 
        queue: &wgpu::Queue) {
        let view_matrix = camera.view_matrix();
        let model_matrix = camera.model_matrix();
        let proj_matrix = camera.proj_matrix();
        // println!("View matrix: {}", view_matrix);
        // println!("Model matrix: {}", model_matrix);
        // println!("Proj matrix: {}", proj_matrix);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(view_matrix.as_slice()));
        queue.write_buffer(&self.camera_buffer, 
            std::mem::size_of::<Mat4>() as wgpu::BufferAddress, 
            bytemuck::cast_slice(model_matrix.as_slice())
        );
        queue.write_buffer(&self.camera_buffer, 
            std::mem::size_of::<Mat4>() as wgpu::BufferAddress*2, 
            bytemuck::cast_slice(proj_matrix.as_slice())
        );

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Model render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some (
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true
                        }),
                        stencil_ops: None,
                    }
                )
            }
        );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        //TODO: replace with indexed draw call
        render_pass.draw(0..self.num_vertices, 0..1);
    }

    pub fn get_depth_texture(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) 
        -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Model depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let tex = device.create_texture(&desc);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), //needed to render depth texture
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        (tex, view, sampler)
    }
}