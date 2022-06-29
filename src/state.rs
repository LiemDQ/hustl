use wgpu::Queue;
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::model::Model;
pub struct State {
    start_time: std::time::SystemTime,
    surface: wgpu::Surface,
    device: wgpu::Device,
    camera: Camera,
    model: Option<Model>,
    size: PhysicalSize<u32>,
}

impl State {
    pub fn new(start_time: std::time::SystemTime, size: PhysicalSize<u32>, 
            adapter: wgpu::Adapter, surface: wgpu::Surface, device: wgpu::Device) -> Self {
         
    }

    fn resize(&mut self, size: PhysicalSize<u32>){
        self.size = size;
    }

    pub fn redraw(&mut self, queue: &wgpu::Queue) {

    }
}