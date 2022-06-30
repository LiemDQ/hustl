use wgpu::Queue;
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::model::Model;
use crate::bg::Background;
pub struct State {
    pub start_time: std::time::SystemTime,
    surface: wgpu::Surface,
    device: wgpu::Device,
    camera: Camera,
    model: Option<Model>,
    pub size: PhysicalSize<u32>,
    background: Background,
    depth: (wgpu::Texture, wgpu::TextureView),
}

impl State {
    pub fn new(start_time: std::time::SystemTime, size: PhysicalSize<u32>, 
            adapter: wgpu::Adapter, surface: wgpu::Surface, device: wgpu::Device) -> Self {
        
        let depth = State::get_depth(size, &device);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        let background = Background::new(&device, &config);
        
        Self { 
            start_time, 
            surface, 
            device,
            camera: Camera::new(size.width as f32, size.height as f32), 
            model: None,
            size,
            depth,
            background
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>){
        self.size = size;
        self.depth = Self::get_depth(size, &self.device);
    }

    pub fn redraw(&mut self, queue: &wgpu::Queue) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None});
        
        self.background.draw(&self.surface, &self.depth.1, &mut encoder)?;
        if let Some(model) = &self.model {

        }
        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn device_event() {

    }

    pub fn window_event() {

    }

    fn get_depth(size: PhysicalSize<u32>, device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth tex"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };
        let tex = device.create_texture(&desc);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }
}