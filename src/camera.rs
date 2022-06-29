use nalgebra_glm as glm;
use glm::{Vec2, Vec3, Vec4, Mat4};
use winit::event::MouseButton;

enum MouseState {
    Unknown,
    Free(Vec2),
    Rotate(Vec2),
    Pan(Vec2, Vec3),
}

pub struct Camera {
    width: f64,
    height: f64,
    pitch: f64,
    yaw: f64,
    scale: f64,
    center: Vec3,
    mouse: MouseState,
}

impl Camera {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width, height,
            pitch: 0.0,
            yaw: 0.0,
            scale: 1.0,
            center: Vec3::zeros(),
            mouse: MouseState::Unknown,
        }
    }

    pub fn set_size(&mut self) {

    }

    pub fn view_matrix(&self) -> Mat4 {

    }

    pub fn model_matrix(&self) -> Mat4 {

    }
}