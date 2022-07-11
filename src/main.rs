mod loader; 
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

use crate::state::State;

#[derive(clap::Parser)]
struct Args {
    filename: Option<String>
}

async fn run(start_time: SystemTime, filename: Option<String>, event_loop: EventLoop<()>, window: Window) {
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
    let device_start_time = SystemTime::now();
    let dt = device_start_time.duration_since(start_time).expect("Negative start time calculated?");
    println!("GPU startup in {:?}", dt);
        

    let mut state = State::new(start_time, filename, size, adapter, surface, device).await;

    event_loop.run(move |event, _, control_flow|  {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { window_id, ref event} if window_id == window.id() => {
                //handle events here
                if !state.window_event(event) {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput { 
                            input: KeyboardInput { 
                                state: ElementState::Pressed, 
                                virtual_keycode: Some(VirtualKeyCode::Escape), //placeholder quit button
                                ..
                            }, .. 
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        },
                        WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {},
                    }
                } else {
                    window.request_redraw();
                }
            }, 
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                match state.render(&queue) {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::DeviceEvent { event,.. } => state.device_event(&event),
            // Event::DeviceEvent { event, ..} => state.
            _ => {},
        }
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
    let args = Args::parse();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("hustl");
    pollster::block_on(run(start, args.filename, event_loop, window));
}
