// The coordinate system in Wgpu is based on DirectX, and Metal's coordinate systems. 
// That means that in normalized device coordinates the x axis and y axis are in the range of -1.0 to +1.0, and the z axis is 0.0 to +1.0. 
// The cgmath crate (as well as most game math crates) are built for OpenGL's coordinate system. 
// This matrix will scale and translate our scene from OpenGL's coordinate sytem to WGPU's
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Orthographic {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Perspective {
    aspect : f32,
    fovy: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum CameraProperties {
    Ortho(Orthographic),
    Persp(Perspective),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub properties: CameraProperties,
    pub znear: f32,
    pub zfar: f32,
    pub clear_color: wgpu::Color,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let projection = match &self.properties {
            CameraProperties::Ortho(properties) => cgmath::ortho(properties.left, properties.right, properties.bottom, properties.top, self.znear, self.zfar),
            CameraProperties::Persp(properties) => cgmath::perspective(cgmath::Deg(properties.fovy), properties.aspect, self.znear, self.zfar),
        };

        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        OPENGL_TO_WGPU_MATRIX*projection*view
    }
}