use wgpu::Queue;
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::model::Model;
use crate::bg::Background;
pub struct State {
    pub start_time: std::time::SystemTime,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    camera: Camera,
    model: Option<Model>,
    pub size: PhysicalSize<u32>,
    background: Background,
    depth: (wgpu::Texture, wgpu::TextureView, wgpu::Sampler),
}

impl State {
    pub fn new(start_time: std::time::SystemTime, size: PhysicalSize<u32>, 
            adapter: wgpu::Adapter, surface: wgpu::Surface, device: wgpu::Device) -> Self {
        
            
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        let background = Background::new(&device, &config);
        let depth = Background::get_depth_texture(&config, &device);
        
        Self { 
            start_time, 
            surface, 
            config,
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
        self.config.width = size.width;
        self.config.height = size.height;
        self.depth = Background::get_depth_texture(&self.config, &self.device);
    }

    pub fn redraw(&mut self, queue: &wgpu::Queue) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None});
        let frame = self.surface.get_current_texture()?;
        
        self.background.draw(&frame, &self.depth.1, &mut encoder)?;
        if let Some(model) = &self.model {

        }
        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        
        Ok(())
    }

    pub fn device_event() {

    }

    pub fn window_event() {

    }

}