#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Mesh {
    pub mesh_name: String,
    pub shader_name: String,
    pub diffuse_texture: String,
}
