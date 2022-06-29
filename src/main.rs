
mod parser; 
mod model;
mod camera;
mod bg;
mod state;

use std::time::SystemTime;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use clap::Parser;


#[derive(clap::Parser)]
struct Args {
    filename: String
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window)};
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }
    ).await.unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            }, 
            None
        ).await.unwrap();

    event_loop.run(move |event, _, control_flow|  match event {
        Event::WindowEvent { window_id, ref event} if window_id == window.id() => {
            //handle events here
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput { 
                    input: KeyboardInput { 
                        state: ElementState::Pressed, 
                        virtual_keycode: Some(VirtualKeyCode::Escape), //placeholder quit button
                        ..
                    }, .. 
                } => *control_flow = ControlFlow::Exit,
                _ => {},
            }
        }, 
        Event::RedrawRequested(window_id) if window_id == window.id() => {

        },
        Event::MainEventsCleared => {
            window.request_redraw();
        },
        _ => {},
    });
}

fn main() {
    let start = SystemTime::now();
    cfg_if::cfg_if!{
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
    // let args = Args::parse();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("Vuestl");
    pollster::block_on(run(event_loop, window));
}
