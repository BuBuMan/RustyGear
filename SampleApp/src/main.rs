#![allow(non_snake_case)]

use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use futures::executor::block_on;

mod graphics;
mod input;
mod resources;
mod ecs;
mod texture;
mod entity;
mod graphics_tutorial;

#[path= "components\\component.rs"]
mod component;
#[path= "components\\transform.rs"]
mod transform;
#[path= "components\\controller.rs"]
mod controller;
#[path= "components\\camera.rs"]
mod camera;

#[path= "systems\\render.rs"]
mod render;
#[path= "systems\\control.rs"]
mod control;

use render::render_system;
use control::control_system;
use graphics::Graphics;
use input::Input;
use resources::Resources;
use ecs::*;

struct AppState {
    input: Input,
    graphics: graphics_tutorial::Graphics,
    start_of_frame: Instant,
    time_elapsed: f64,
    delta_time: f64,
    target_fps: u16,
    exit_app: bool,
}

impl AppState {
    pub fn new(input: Input, graphics: graphics_tutorial::Graphics, target_fps: Option<u16>) -> AppState {
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
    env_logger::init();
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

    // let graphics = block_on(Graphics::new(&window));
    let graphics = block_on(graphics_tutorial::Graphics::new(&window));
    let mut app_state = AppState::new(Input::new(&event_pump), graphics, None);
    
    let mut ecs = EntityComponentSystem::new(10_000);
    ecs.create_entity(&resources.prefabs["spaceship.json"]);
    ecs.create_entity(&resources.prefabs["ortho_camera.json"]);

    'game_loop: loop {
        enter_frame(&mut event_pump, &mut app_state);

        if app_state.exit_app {
            break 'game_loop;
        }

        // update_game(&app_state, &mut game_state);

        control_system(&app_state.input, app_state.delta_time as f32, &ecs);

        match app_state.graphics.render() {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => app_state.graphics.resize(app_state.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => app_state.exit_app = true,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        /*match render_system(&mut app_state.graphics, &ecs) {
            Ok(_) => {}
            // Recreate the swap_chain if lost
            Err(wgpu::SwapChainError::Lost) => app_state.graphics.resize(app_state.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SwapChainError::OutOfMemory) => app_state.exit_app = true,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }*/

        exit_frame(&mut app_state);
    }
}
