use nalgebra_glm as glm;
use glm::{Vec2, Vec3, Vec4, Mat4};
use winit::event::MouseButton;

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

/// Uses the arcball method of rotations with the mouse.
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