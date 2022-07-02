use itertools::Itertools;
use nalgebra_glm as glm;
use glm::{Vec2, Vec3, Vec4, Mat4};
use winit::event::MouseButton;
use crate::loader::Vertex;

#[derive(Copy, Clone, Debug)]
enum MouseState {
    Unknown,
    Free(Vec2),
    Rotate(Vec2),
    Pan(Vec2, Vec3),
}

enum Projection {
    Orthographic,
    Perspective,
}

impl Projection {
    pub fn get_coefficient(&self) -> f32 {
        match self {
            Self::Orthographic => 0.0,
            Self::Perspective => 0.5,
        }
    }
}

/// Camera implementation. Taken from Foxtrot 
pub struct Camera {
    width: f32,
    height: f32,
    //pitch as euler angle
    pitch: f32,
    //yaw as euler angle
    yaw: f32,
    scale: f32,
    center: Vec3,
    mouse: MouseState,
    projection_type: Projection,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width, 
            height,
            pitch: 0.0,
            yaw: 0.0,
            scale: 1.0,
            center: Vec3::zeros(),
            mouse: MouseState::Unknown,
            projection_type: Projection::Orthographic,
        }
    }

    pub fn mouse_pressed(&mut self, button: MouseButton) {
        if let MouseState::Free(pos) = &self.mouse {
            match button {
                MouseButton::Left => Some(MouseState::Rotate(*pos)),
                MouseButton::Right => Some(MouseState::Pan(*pos, self.mouse_pos(*pos))),
                _ => None,
            }.map(|m| self.mouse = m);
        }
    }

    pub fn mouse_released(&mut self, button: MouseButton) {
        match &self.mouse {
            MouseState::Rotate(pos) if button == MouseButton::Left => 
                Some(MouseState::Free(*pos)),
            MouseState::Pan(pos, ..) if button == MouseButton::Right => 
                Some(MouseState::Free(*pos)),
            _ => None,
        }.map(|m| self.mouse = m);
    }

    pub fn mat(&self) -> Mat4 {
        self.proj_matrix()*self.view_matrix()*self.model_matrix()
    }

    pub fn mat_i(&self) -> Mat4 {
        (self.proj_matrix()*self.view_matrix()*self.model_matrix()).try_inverse().expect("Failed to invert mouse matrix")
    }
    /// Convert normalized mouse position into 3D
    pub fn mouse_pos(&self, pos_norm: Vec2) -> Vec3 {
        (self.mat_i() * Vec4::new(pos_norm.x, pos_norm.y, 0.0, 1.0)).xyz()
    }

    pub fn mouse_scroll(&mut self, delta: f32) {
        if let MouseState::Free(pos) = self.mouse {
            self.scale(1.0 + delta/200.0, pos);
        }
    }

    pub fn spin(&mut self, dx: f32, dy: f32) {
        self.pitch += dx;
        self.yaw += dy;
    }

    pub fn scale(&mut self, value: f32, pos: Vec2) {
        let start_pos = self.mouse_pos(pos);
        self.scale *= value;
        let end_pos = self.mouse_pos(pos);
        let delta = start_pos - end_pos;
        let mut delta_mouse = (self.mat() * delta.to_homogeneous()).xyz();
        delta_mouse.z = 0.0;

        self.center += (self.mat_i() * delta_mouse.to_homogeneous()).xyz();
    }

    pub fn fit_verts(&mut self, verts: &[Vertex]) {
        let xb = verts.iter().map(|v| v.pos[0]).minmax().into_option().unwrap();
        let yb = verts.iter().map(|v| v.pos[1]).minmax().into_option().unwrap();
        let zb = verts.iter().map(|v| v.pos[2]).minmax().into_option().unwrap();
        let dx = xb.1 - xb.0;
        let dy = yb.1 - yb.0;
        let dz = zb.1 - zb.0;
        self.scale = (1.0 / dx.max(dy).max(dz)) as f32;
        self.center = Vec3::new((xb.0 + xb.1) as f32 / 2.0,
                                (yb.0 + yb.1) as f32 / 2.0,
                                (zb.0 + zb.1) as f32 / 2.0);
    }


    pub fn mouse_move(&mut self, new_pos: Vec2) {
        let x_norm = 2.0 * (new_pos.x / self.width - 0.5);
        let y_norm = -2.0 * (new_pos.y / self.height - 0.5);
        let new_pos = Vec2::new(x_norm, y_norm);

        match &self.mouse {
            MouseState::Pan(_pos, orig)=> {
                let current_pos = self.mouse_pos(new_pos);
                let delta_pos = orig - current_pos;
                self.center += delta_pos;
            },
            MouseState::Rotate(pos) => {
                let delta = new_pos - *pos;
                self.spin(delta.x * 3.0, -delta.y*3.0 * self.height/self.width);
            },
            _ => (),
        }

        match &mut self.mouse {
            MouseState::Free(pos) 
            | MouseState::Pan(pos,..) 
            | MouseState::Rotate(pos) => *pos = new_pos,
            MouseState::Unknown => self.mouse = MouseState::Free(new_pos),
        }
    }


    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
    
    /// Returns a matrix which compensates for window aspect ratio and clipping
    pub fn view_matrix(&self) -> Mat4 {
        let i = Mat4::identity();
        
        // The Z clipping range is 0-1
        glm::translate(&i, &Vec3::new(0.0, 0.0, 0.5))*
        //scale to compensate for aspect ratio and reduce Z scale to improve clipping
        glm::scale(&i, &Vec3::new(1.0, self.width/ self.height, 0.1))

    }

    pub fn model_matrix(&self) -> Mat4 {
        let i = Mat4::identity();

        glm::scale(&i, &Vec3::new(self.scale, self.scale, self.scale)) 
        * glm::rotate_x(&i, self.yaw) * glm::rotate_y(&i,self.pitch)*glm::translate(&i, &-self.center)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        const ROW: usize = 4;
        let mut mat = Mat4::identity();
        let aspect_ratio = self.width/self.height;
        if aspect_ratio > 1.0 {
            mat[0] = 1.0/aspect_ratio;
        } else {
            mat[ROW*1+1] = aspect_ratio;
        }
        mat[ROW*2+2] = self.scale / 2.0;
        mat[ROW*2+3] = self.projection_type.get_coefficient();
        mat
    }

}