use chip8::constants::*;
use chip8::driver::*;
use std::collections::HashMap;

pub const SCREEN_W: u32 = 800;
pub const SCREEN_H: u32 = 600;

const COLOR_0: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const COLOR_1: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

struct InputState {
    state: HashMap<sdl2::keyboard::Keycode, bool>,
}

impl InputState {
    fn new() -> Self {
        return Self {
            state: HashMap::new(),
        };
    }

    fn get(&self, keycode: &sdl2::keyboard::Keycode) -> bool {
        if !self.state.contains_key(keycode) {
            return false;
        }

        return *self.state.get(keycode).unwrap();
    }

    fn set(&mut self, keycode: sdl2::keyboard::Keycode, value: bool) {
        self.state.insert(keycode, value);
    }
}

pub struct Sd2lDriver {
    key_mappings: HashMap<u8, sdl2::keyboard::Keycode>,
    input_state: InputState,
    pub buffer: Vec<u8>,
}

impl Sd2lDriver {
    pub fn new() -> Self {
        let mut key_mappings = HashMap::new();
        key_mappings.insert(KEY_1, sdl2::keyboard::Keycode::Kp1);
        key_mappings.insert(KEY_2, sdl2::keyboard::Keycode::Kp2);
        key_mappings.insert(KEY_3, sdl2::keyboard::Keycode::Kp3);
        key_mappings.insert(KEY_C, sdl2::keyboard::Keycode::Kp4);
        key_mappings.insert(KEY_4, sdl2::keyboard::Keycode::Q);
        key_mappings.insert(KEY_5, sdl2::keyboard::Keycode::W);
        key_mappings.insert(KEY_6, sdl2::keyboard::Keycode::E);
        key_mappings.insert(KEY_D, sdl2::keyboard::Keycode::R);
        key_mappings.insert(KEY_7, sdl2::keyboard::Keycode::A);
        key_mappings.insert(KEY_8, sdl2::keyboard::Keycode::S);
        key_mappings.insert(KEY_9, sdl2::keyboard::Keycode::D);
        key_mappings.insert(KEY_E, sdl2::keyboard::Keycode::F);
        key_mappings.insert(KEY_A, sdl2::keyboard::Keycode::Z);
        key_mappings.insert(KEY_0, sdl2::keyboard::Keycode::X);
        key_mappings.insert(KEY_B, sdl2::keyboard::Keycode::C);
        key_mappings.insert(KEY_F, sdl2::keyboard::Keycode::V);

        return Self {
            key_mappings: key_mappings,
            input_state: InputState::new(),
            buffer: vec![0; 4 * X_SIZE * Y_SIZE],
        };
    }

    pub fn pool_event(&mut self, event: &sdl2::event::Event) {
        match event {
            sdl2::event::Event::KeyDown { keycode, .. } => {
                let key = keycode.unwrap();
                let value = self.find_key_for_value(key);
                if value.is_some() {
                    self.input_state.set(key, true);
                }
            }
            sdl2::event::Event::KeyUp { keycode, .. } => {
                let key = keycode.unwrap();
                let value = self.find_key_for_value(key);
                if value.is_some() {
                    self.input_state.set(key, false);
                }
            }
            _ => {}
        }
    }

    fn find_key_for_value(&self, value: sdl2::keyboard::Keycode) -> Option<&u8> {
        return self
            .key_mappings
            .iter()
            .find_map(|(key, &val)| if val == value { Some(key) } else { None });
    }

    fn copy_color(&mut self, index: usize, color: &[u8]) {
        for i in 0..=3 {
            self.buffer[index + i] = color[i];
        }
    }

    pub fn draw(&mut self, canvas: &mut sdl2::render::WindowCanvas) {
        let buffer = self.buffer.as_mut_slice();
        let surface = sdl2::surface::Surface::from_data(
            buffer,
            X_SIZE as u32,
            Y_SIZE as u32,
            X_SIZE as u32 * 4,
            sdl2::pixels::PixelFormatEnum::RGBA32,
        )
        .unwrap();

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(surface)
            .unwrap();

        let screen_rect = sdl2::rect::Rect::new(0, 0, SCREEN_W, SCREEN_H);
        let zoom = (SCREEN_H / Y_SIZE as u32).min(SCREEN_W / X_SIZE as u32) as i32;
        let texture_rect = sdl2::rect::Rect::new(
            (SCREEN_W as i32 - zoom * X_SIZE as i32) / 2,
            (SCREEN_H as i32 - zoom * Y_SIZE as i32) / 2,
            zoom as u32 * X_SIZE as u32,
            zoom as u32 * Y_SIZE as u32,
        );

        canvas.copy(&texture, screen_rect, texture_rect).unwrap();
    }
}

impl Driver for Sd2lDriver {
    #[cfg(target_os = "windows")]
    fn sound_do_beep(&mut self, frequency: u32, duration: u32) {
        winconsole::console::beep(frequency, duration);
    }

    #[cfg(not(target_os = "windows"))]
    fn sound_do_beep(&mut self, _frequency: u32, _duration: u32) {
    }

    fn video_fill_buffer(&mut self, display: &Vec<Vec<usize>>) {
        let mut index = 0;
        for y in 0..Y_SIZE {
            for x in 0..X_SIZE {
                self.copy_color(
                    index,
                    if display[x][y] == 0 {
                        &COLOR_0
                    } else {
                        &COLOR_1
                    },
                );
                index += 4;
            }
        }
    }

    fn input_is_key_down(&mut self, keycode: u8) -> bool {
        let mapping = self.key_mappings.get(&keycode).unwrap();
        return self.input_state.get(&mapping);
    }

    fn input_is_key_up(&mut self, keycode: u8) -> bool {
        let mapping = self.key_mappings.get(&keycode).unwrap();
        return !self.input_state.get(&mapping);
    }

    fn input_is_any_key_down(&mut self, keycode: &mut u8) -> bool {
        for (key, val) in self.key_mappings.iter() {
            if self.input_state.get(&val) {
                *keycode = *key;
                return true;
            }
        }
        return false;
    }
}
