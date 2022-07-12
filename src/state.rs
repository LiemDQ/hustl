
use nalgebra_glm::Vec2;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, MouseScrollDelta, WindowEvent, ElementState};

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
    is_first_frame: bool,
}

impl State {
    pub fn new(start_time: std::time::SystemTime, model: Option<Model>, size: PhysicalSize<u32>, 
             surface: wgpu::Surface, device: wgpu::Device, config: wgpu::SurfaceConfiguration) -> Self {
      
            
        surface.configure(&device, &config);
        let background = Background::new(&device, &config);
        let depth = Model::get_depth_texture(&config, &device);

        
        Self { 
            start_time, 
            surface, 
            config,
            device,
            camera: Camera::new(size.width as f32, size.height as f32), 
            model,
            size,
            depth,
            background,
            is_first_frame: true
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>){
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth = Model::get_depth_texture(&self.config, &self.device);
        self.camera.set_size(size.width as f32, size.height as f32);
    }

    pub fn render(&mut self, queue: &wgpu::Queue) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None});
        let frame = self.surface.get_current_texture()?;
        
        self.background.draw(&frame, &self.depth.1, &mut encoder)?;
        if let Some(model) = &self.model {
            model.draw(&self.camera, &frame, &self.depth.1, &mut encoder, &queue);
        }

        if self.model.is_some() && self.is_first_frame {
            let end = std::time::SystemTime::now();
            let dt = end.duration_since(self.start_time).expect("Negative startup time calculated?!");
            println!("First render in {:?}", dt);
            self.is_first_frame = false;
        }
        
        queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        if self.is_first_frame {
            self.is_first_frame = false;
        }
        
        Ok(())
    }

    pub fn device_event(&mut self, e: &DeviceEvent) {
        if let DeviceEvent::MouseWheel { delta } = e {
            if let MouseScrollDelta::PixelDelta(p) = delta {
                self.camera.mouse_scroll(p.y as f32);
            }
        }
    }

    pub fn window_event(&mut self, e: &WindowEvent) -> bool {
        match e {
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => self.camera.mouse_pressed(*button),
                    ElementState::Released => self.camera.mouse_released(*button),
                }
                true
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.camera.mouse_move(Vec2::new(position.x as f32, position.y as f32));
                true
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, verti) = delta {
                    self.camera.mouse_scroll(verti*10.0);
                }
                true
            }
            _ => false,
        }

    }

}