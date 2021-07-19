#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Transform {
    pub fn build_model_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position)*cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)*cgmath::Matrix4::from(self.rotation)
    }
}
