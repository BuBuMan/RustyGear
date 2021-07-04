#![allow(non_snake_case)]

use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use futures::executor::block_on;

mod graphics;
mod input;

use graphics::Graphics;
use input::Input;

impl graphics::Graphics {
    fn render(&mut self, game_state: &GameState) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

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
                        load: wgpu::LoadOp::Clear(game_state.clear_color),
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        // Finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

struct AppState {
    input: Input,
    graphics: Graphics,
    start_of_frame: Instant,
    time_elapsed: f64,
    delta_time: f64,
    target_fps: u16,
    exit_app: bool
}

struct GameState {
    clear_color: wgpu::Color
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

fn update_game(appState: &AppState, game_state: &mut GameState) {
    if appState.input.is_key_down(sdl2::keyboard::Scancode::R) {
        game_state.clear_color.r = 1.0;
        game_state.clear_color.g = 0.0;
        game_state.clear_color.b = 0.0;
    }

    if appState.input.is_key_down(sdl2::keyboard::Scancode::G) {
        game_state.clear_color.r = 0.0;
        game_state.clear_color.g = 1.0;
        game_state.clear_color.b = 0.0;
    }

    if appState.input.is_key_down(sdl2::keyboard::Scancode::B) {
        game_state.clear_color.r = 0.0;
        game_state.clear_color.g = 0.0;
        game_state.clear_color.b = 1.0;
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
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Sample", 800, 800)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let graphics = block_on(Graphics::new(&window));
    let mut app_state = AppState::new(Input::new(&event_pump), graphics, None);
    let mut game_state = GameState {
        clear_color: wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0
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
