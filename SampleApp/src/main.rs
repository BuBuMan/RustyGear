#![allow(non_snake_case)]

use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use futures::executor::block_on;
use std::fmt::Debug;
use cgmath::prelude::*;

mod graphics;
mod input;
mod resources;

use graphics::Graphics;
use input::Input;
use resources::Resources;

impl graphics::Graphics {
    fn render(&mut self, game_state: &GameState) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

        self.uniforms.update_view_proj(game_state.camera.build_view_projection_matrix());
        self.instance.update_model_mat(game_state.ship.build_model_matrix());
        // For optimization create a separate buffer and copy it's contents to the uniform buffer.
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&[self.instance]));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(game_state.camera.clear_color),
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.instance_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        // Finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

// The coordinate system in Wgpu is based on DirectX, and Metal's coordinate systems. 
// That means that in normalized device coordinates the x axis and y axis are in the range of -1.0 to +1.0, and the z axis is 0.0 to +1.0. 
// The cgmath crate (as well as most game math crates) are built for OpenGL's coordinate system. 
// This matrix will scale and translate our scene from OpenGL's coordinate sytem to WGPU's
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Orthographic {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Perspective {
    aspect : f32,
    fovy: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum CameraProperties {
    Ortho(Orthographic),
    Persp(Perspective),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    properties: CameraProperties,
    znear: f32,
    zfar: f32,
    clear_color: wgpu::Color,
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

struct AppState {
    input: Input,
    graphics: Graphics,
    start_of_frame: Instant,
    time_elapsed: f64,
    delta_time: f64,
    target_fps: u16,
    exit_app: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Transform {
    position: cgmath::Vector3<f32>,
    scale: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Controller {
    acceleration_speed: f32,
    velocity: cgmath::Vector3<f32>, 
}

impl Transform {
    pub fn build_model_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position)*cgmath::Matrix4::from(self.rotation)*cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

impl Controller {
    fn Update(&mut self, transform: &mut Transform, appState: &AppState) {
        let mut acc_dir = cgmath::Vector3::new(0.0, 0.0, 0.0);
        if appState.input.is_key_pressed(sdl2::keyboard::Scancode::W) {
            acc_dir.x = 1.0;
        }
        else if appState.input.is_key_pressed(sdl2::keyboard::Scancode::S) {
            acc_dir.x = -1.0;
        }
        self.velocity = self.acceleration_speed*(appState.delta_time as f32)*acc_dir + self.velocity*0.99;
        transform.position += self.velocity*(appState.delta_time as f32);
    }
}

struct GameState {
    camera: Camera,
    ship: Transform,
    controller: Controller
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

fn update_game(app_state: &AppState, game_state: &mut GameState) {
    game_state.controller.Update(&mut game_state.ship, app_state);
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

    // let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let graphics = block_on(Graphics::new(&window));
    let mut app_state = AppState::new(Input::new(&event_pump), graphics, None);
    let mut game_state = GameState {
        camera: serde_json::from_str(&resources.prefabs["ortho_camera.json"]).unwrap(),
        ship: serde_json::from_str(&resources.prefabs["spaceship.json"]).unwrap(),
        controller: Controller {
            acceleration_speed: 200.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0)
        }
    };

    'game_loop: loop {
        enter_frame(&mut event_pump, &mut app_state);

        if app_state.exit_app {
            break 'game_loop;
        }

        update_game(&app_state, &mut game_state);

        match app_state.graphics.render(&game_state) {
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
