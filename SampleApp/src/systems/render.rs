use crate::graphics::Graphics;
use crate::graphics::ModelProperties;
use crate::ecs::EntityComponentSystem;
use crate::transform::Transform;
use crate::camera::Camera;
use crate::mesh::Mesh;

pub fn render_system(graphics: &mut Graphics, ecs: &EntityComponentSystem) -> Result<(), wgpu::SwapChainError> {
    let frame = graphics
        .swap_chain
        .get_current_frame()?
        .output;

    for camera_entity in ecs.cameras() {
        let camera_components = ecs.get_component_set::<Camera>().unwrap().borrow();
        let camera_component = camera_components.get(camera_entity);

        match camera_component {
            Some(camera) => {
                graphics.uniforms.update_view_proj(camera.build_view_projection_matrix());
                graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[graphics.uniforms]));

                let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                            view: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(camera.clear_color),
                                store: true,
                            }
                        }
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_bind_group(1, &graphics.uniform_bind_group, &[]);

                let transform_components = ecs.get_component_set::<Transform>().unwrap().borrow();
                let mesh_components = ecs.get_component_set::<Mesh>().unwrap().borrow();
                let active_entities = ecs.active_entities();

                for entity in active_entities {
                    let transform_component = transform_components.get(entity);
                    let mesh_component = mesh_components.get(entity);
                    match (transform_component, mesh_component)  {
                        (Some(transform), Some(mesh_component)) => {
                            
                            render_pass.set_pipeline(&graphics.pipelines.get(&mesh_component.shader_name).unwrap());
                            let model = graphics.models.get(&mesh_component.mesh_name).unwrap();
                            render_pass.set_bind_group(0, &graphics.textures.get(&mesh_component.diffuse_texture).unwrap(), &[]);
                            render_pass.set_vertex_buffer(0, model.vertex_buffer.as_ref().unwrap().slice(..));
                            render_pass.set_index_buffer(model.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);

                            let model_properties = ModelProperties {
                                model_matrix: transform.build_model_matrix().into(),
                            };

                            render_pass.set_push_constants(wgpu_types::ShaderStage::VERTEX, 0, bytemuck::cast_slice(&[model_properties]));
                            render_pass.draw_indexed(0..model.indices.len() as u32, 0, 0..1);
                        }
                        _ => {}
                    };
                }
            
                drop(render_pass);
            
                // Finish the command buffer, and to submit it to the gpu's render queue.
                graphics.queue.submit(std::iter::once(encoder.finish()));
            }
            None => {}
        }
    }

    Ok(())
}