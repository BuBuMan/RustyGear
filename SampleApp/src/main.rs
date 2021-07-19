#![allow(non_snake_case)]

use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use futures::executor::block_on;

mod graphics;
mod input;
mod resources;
mod component;
mod ecs;
mod entity;
mod texture;

#[path= "components\\transform.rs"]
mod transform;
#[path= "components\\controller.rs"]
mod controller;
#[path= "components\\camera.rs"]
mod camera;

use crate::transform::Transform;
use crate::controller::Controller;
use crate::camera::Camera;
use graphics::Graphics;
use input::Input;
use resources::Resources;
use ecs::*;

fn control_system(input: &Input, delta_time: f32, ecs: &EntityComponentSystem) {
    let mut transforms = ecs.get_component_set::<Transform>().unwrap().borrow_mut();
    let mut controllers = ecs.get_component_set::<Controller>().unwrap().borrow_mut();
    
    for entity in ecs.active_entities() {
        match (transforms.get_mut(&entity), controllers.get_mut(&entity)) {
            (Some(transform), Some(controller)) => {
                let mut acc_dir = transform.rotation*cgmath::Vector3{x: 1.0, y: 0.0, z: 0.0};
                if input.is_key_pressed(Scancode::W) {
                    acc_dir *= 1.0;
                }
                else if input.is_key_pressed(Scancode::S) {
                    acc_dir *= -1.0;
                }
                else {
                    acc_dir *= 0.0;
                }

                cgmath::Deg(1.0);

                let mut rotate_dir = 0.0;
                if input.is_key_pressed(Scancode::A) {
                    rotate_dir = 1.0;
                }
                else if input.is_key_pressed(Scancode::D) {
                    rotate_dir = -1.0;
                }

                controller.velocity = controller.acceleration_speed*delta_time*acc_dir + controller.velocity*0.99;
                transform.position += controller.velocity*delta_time;
                transform.rotation = transform.rotation*cgmath::Quaternion::from(
                    cgmath::Euler {
                        x: cgmath::Deg(0.0), 
                        y: cgmath::Deg(0.0), 
                        z: cgmath::Deg(controller.rotation_speed*rotate_dir*delta_time),
                    });
                    }
            _ => {}
        }
    }
}

fn render_system(graphics: &mut Graphics, ecs: &EntityComponentSystem) -> Result<(), wgpu::SwapChainError> {    
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

                render_pass.set_pipeline(&graphics.pipeline);
                render_pass.set_bind_group(0, &graphics.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &graphics.uniform_bind_group, &[]);
                render_pass.set_bind_group(2, &graphics.instance_bind_group, &[]);
                render_pass.set_vertex_buffer(0, graphics.vertex_buffer.slice(..));
                render_pass.set_index_buffer(graphics.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                let transform_components = ecs.get_component_set::<Transform>().unwrap().borrow();
                let active_entities = ecs.active_entities();

                for entity in active_entities {
                    let transform_component = transform_components.get(entity);
                    match transform_component {
                        Some(transform) => {
                            graphics.instance.update_model_mat(transform.build_model_matrix());
                            graphics.queue.write_buffer(&graphics.instance_buffer, 0, bytemuck::cast_slice(&[graphics.instance]));
                            render_pass.draw_indexed(0..graphics.num_indices, 0, 0..1);
                        }
                        None => {}
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

struct AppState {
    input: Input,
    graphics: Graphics,
    start_of_frame: Instant,
    time_elapsed: f64,
    delta_time: f64,
    target_fps: u16,
    exit_app: bool,
}

impl AppState {
    pub fn new(input: Input, graphics: Graphics, target_fps: Option<u16>) -> AppState {
        let fps = target_fps.unwrap_or(60);
        AppState {
            input: input,
            graphics: graphics,
            start_of_frame: Instant::now(),
            time_elapsed: 0.0,
            target_fps: fps,
            delta_time: 1.0/(fps as f64),
            exit_app: false
        }
    }

    pub fn TargetRefreshRate(&self) -> f64 {
        1.0/(self.target_fps as f64)
    }
}

fn enter_frame(event_pump: &mut sdl2::EventPump, app_state: &mut AppState) {
    app_state.start_of_frame = Instant::now();
    app_state.input.update(&event_pump.keyboard_state());

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>  {
                app_state.exit_app = true;
            },
            Event::Window { win_event : sdl2::event::WindowEvent::Resized(width, height), .. }=> {
                app_state.graphics.resize((width as u32, height as u32));
            },
            _ => {}
        }
    }
}

fn exit_frame(app_state: &mut AppState) {
    'lockFPS: loop {
        if app_state.start_of_frame.elapsed().as_secs_f64() >= app_state.TargetRefreshRate() {
            break 'lockFPS;
        }
    }

    app_state.delta_time = app_state.start_of_frame.elapsed().as_secs_f64();
    app_state.time_elapsed += app_state.delta_time;

    // println!("Application has been running for: {:?} seconds", appState.timeElapsed);
    // println!("Application FPS: {:?} ", 1.0/appState.deltaTime);
}

fn main() {
    let resources = Resources::new();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Sample", 1280, 720)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let graphics = block_on(Graphics::new(&window));
    let mut app_state = AppState::new(Input::new(&event_pump), graphics, None);
    
    let mut ecs = EntityComponentSystem::new(10_000);
    ecs.create_entity(serde_json::from_str(&resources.prefabs["spaceship.json"]).unwrap(), None, Some(Controller {
        acceleration_speed: 200.0,
        rotation_speed: 90.0,
        velocity: cgmath::Vector3::new(0.0, 0.0, 0.0)
    }));

    ecs.create_entity(None, serde_json::from_str(&resources.prefabs["ortho_camera.json"]).unwrap(), None);

    'game_loop: loop {
        enter_frame(&mut event_pump, &mut app_state);

        if app_state.exit_app {
            break 'game_loop;
        }

        // update_game(&app_state, &mut game_state);

        control_system(&app_state.input, app_state.delta_time as f32, &ecs);

        match render_system(&mut app_state.graphics, &ecs) {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => app_state.graphics.resize(app_state.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => app_state.exit_app = true,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        exit_frame(&mut app_state);
    }
}
