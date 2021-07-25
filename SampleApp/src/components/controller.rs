#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Controller {
    pub acceleration_speed: f32,
    pub rotation_speed: f32,
    pub velocity: cgmath::Vector3<f32>,
}
