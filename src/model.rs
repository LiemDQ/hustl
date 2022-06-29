use wgpu::util::DeviceExt;
use nalgebra_glm as glm;
use glm::{Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable )]
struct VertexRaw {
    pos: [f32; 4],
    norm: [f32; 4],
}

impl VertexRaw {
    fn from_vertex(v: &Vertex) -> Self {
        Self {
            pos: [v.pos.x, v.pos.y, v.pos.z, 1.0],
            norm: [v.norm.x, v.norm.y, v.norm.z, 1.0],
        }
    }
}

pub struct Vertex {
    pos: Vec3,
    norm: Vec3,
}

pub struct Model {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    num_indices: u32,
}

impl Model {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_data: Vec<VertexRaw> = vertices.into_iter()
            .map(VertexRaw::from_vertex)
            .collect();

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Model vertex buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        // let uniform_buf = device.create_buffer_init(
        //     &wgpu::BufferDescriptor {
        //         label: Some("Uniform buffer"),
        //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     }
        // )

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[]
            }
        );

        let vertex_buf_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                //positions
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                //normals
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<Vec4>() as wgpu::BufferAddress,
                    shader_location: 1,
                }
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
                    targets: &[]
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
                depth_stencil: None,
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
            index_buf,
            vertex_buf,
            num_indices: indices.len() as u32
        }

    }

    pub fn draw(&self,) {

    }
}