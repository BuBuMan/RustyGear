use sdl2::EventPump;
use sdl2::keyboard::Scancode;
use sdl2::keyboard::KeyboardState;
use std::collections::HashSet;

pub struct Input {
    current_pressed_keys: HashSet<Scancode>,
    previous_pressed_keys: HashSet<Scancode>,
}

impl Input {
    pub fn new(eventPump: &EventPump) -> Self {
        Self {
            current_pressed_keys: eventPump.keyboard_state().pressed_scancodes().collect(),
            previous_pressed_keys: eventPump.keyboard_state().pressed_scancodes().collect()
        }
    }

    pub fn update(&mut self, newKeyboardState: &KeyboardState) {
        std::mem::swap(&mut self.current_pressed_keys, &mut self.previous_pressed_keys);
        self.current_pressed_keys = newKeyboardState.pressed_scancodes().collect();
    }

    #[allow(dead_code)]
    pub fn is_key_pressed(&self, key: Scancode) -> bool {
        self.current_pressed_keys.contains(&key)
    }

    pub fn is_key_down(&self, key: Scancode) -> bool {
        self.current_pressed_keys.contains(&key) && !self.previous_pressed_keys.contains(&key)
    }

    #[allow(dead_code)]
    pub fn is_key_up(&self, key: Scancode) -> bool {
        !self.current_pressed_keys.contains(&key) && self.previous_pressed_keys.contains(&key)
    }
}