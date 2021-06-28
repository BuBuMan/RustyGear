#![allow(non_snake_case)]

use sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use std::collections::HashSet;

struct Input {
    currentPressedKeys: HashSet<sdl2::keyboard::Scancode>,
    previousPressedKeys: HashSet<sdl2::keyboard::Scancode>,
}

impl Input {
    pub fn new(eventPump: &sdl2::EventPump) -> Input {
        Input {
            currentPressedKeys: eventPump.keyboard_state().pressed_scancodes().collect(),
            previousPressedKeys: eventPump.keyboard_state().pressed_scancodes().collect()
        }
    }

    pub fn Update(&mut self, newKeyboardState: &sdl2::keyboard::KeyboardState) {
        std::mem::swap(&mut self.currentPressedKeys, &mut self.previousPressedKeys);
        self.currentPressedKeys = newKeyboardState.pressed_scancodes().collect();
    }

    pub fn IsKeyPressed(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.currentPressedKeys.contains(&key)
    }

    pub fn IsKeyDown(&self, key: sdl2::keyboard::Scancode) -> bool {
        self.currentPressedKeys.contains(&key) && !self.previousPressedKeys.contains(&key)
    }

    pub fn IsKeyUp(&self, key: sdl2::keyboard::Scancode) -> bool {
        !self.currentPressedKeys.contains(&key) && self.previousPressedKeys.contains(&key)
    }
}

struct AppState {
    input: Input,
    startOfFrame: Instant,
    timeElapsed: f64,
    deltaTime: f64,
    targetFPS: u16,
    exitApp: bool
}

impl AppState {
    pub fn new(input: Input, targerFPS: Option<u16>) -> AppState {
        let fps = targerFPS.unwrap_or(60);
        AppState {
            input: input,
            startOfFrame: Instant::now(),
            timeElapsed: 0.0,
            targetFPS: fps,
            deltaTime: 1.0/(fps as f64),
            exitApp: false
        }
    }

    pub fn TargetRefreshRate(&self) -> f64 {
        1.0/(self.targetFPS as f64)
    }
}

fn UpdateGame(appState: &AppState, color: &mut Color) {
    if appState.input.IsKeyDown(sdl2::keyboard::Scancode::A) && color.r > 1 {
        color.r -= 127;
    }

    if appState.input.IsKeyPressed(sdl2::keyboard::Scancode::S) {
        color.g = 0;
    }

    if appState.input.IsKeyUp(sdl2::keyboard::Scancode::D) {
        color.b = 0;
    }

    if appState.input.IsKeyUp(sdl2::keyboard::Scancode::R) {
        color.r = 255;
        color.g = 255;
        color.b = 255;
    }
}

fn EnterFrame(eventPump: &mut sdl2::EventPump, appState: &mut AppState) {
    appState.startOfFrame = Instant::now();
    appState.input.Update(&eventPump.keyboard_state());

    for event in eventPump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => 
            {
                appState.exitApp = true;
            },
            _ => {}
        }
    }
}

fn ExitFrame(appState: &mut AppState) {
    'lockFPS: loop {
        if appState.startOfFrame.elapsed().as_secs_f64() >= appState.TargetRefreshRate() {
            break 'lockFPS;
        }
    }

    appState.deltaTime = appState.startOfFrame.elapsed().as_secs_f64();
    appState.timeElapsed += appState.deltaTime;

    // println!("Application has been running for: {:?} seconds", appState.timeElapsed);
    // println!("Application FPS: {:?} ", 1.0/appState.deltaTime);
}

fn main() {
    let sdlContext = sdl2::init().unwrap();
    let videoSubsystem = sdlContext.video().unwrap();
    let window = videoSubsystem
        .window("Sample", 800, 800)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut eventPump = sdlContext.event_pump().unwrap();
    let mut appState = AppState::new(Input::new(&eventPump), None);
    let mut color = Color::RGB(255, 255, 255); 

    'gameloop: loop {
        EnterFrame(&mut eventPump, &mut appState);

        if appState.exitApp {
            break 'gameloop;
        }

        UpdateGame(&appState, &mut color);

        canvas.set_draw_color(color);
        canvas.clear();
        canvas.present();

        ExitFrame(&mut appState);
    }
}
