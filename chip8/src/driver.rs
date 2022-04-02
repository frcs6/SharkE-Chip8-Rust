pub const KEY_0: u8 = 0x0;
pub const KEY_1: u8 = 0x1;
pub const KEY_2: u8 = 0x2;
pub const KEY_3: u8 = 0x3;
pub const KEY_4: u8 = 0x4;
pub const KEY_5: u8 = 0x5;
pub const KEY_6: u8 = 0x6;
pub const KEY_7: u8 = 0x7;
pub const KEY_8: u8 = 0x8;
pub const KEY_9: u8 = 0x9;
pub const KEY_A: u8 = 0xA;
pub const KEY_B: u8 = 0xB;
pub const KEY_C: u8 = 0xC;
pub const KEY_D: u8 = 0xD;
pub const KEY_E: u8 = 0xE;
pub const KEY_F: u8 = 0xF;

pub trait Driver {
    fn sound_do_beep(&mut self, frequency: u16, duration: f64);
    fn video_fill_buffer(&mut self, display: &Vec<Vec<usize>>);
    fn input_is_key_down(&mut self, key: u8) -> bool;
    fn input_is_key_up(&mut self, key: u8) -> bool;
    fn input_is_any_key_down(&mut self, key: &mut u8) -> bool;
}
