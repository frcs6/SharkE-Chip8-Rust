// TODO: Don't forget to not initialize driver for cpu test !?

pub fn sound_do_beep(frequency: u16, duration: f64) {
    // TODO
}

pub fn video_fill_buffer(display: &Vec<Vec<usize>>) {
    // TODO
}

pub fn input_is_key_down(key: u8) -> bool {
    // TODO
    return false;
}

pub fn input_is_key_up(key: u8) -> bool {
    // TODO
    return true;
}

pub fn input_is_any_key_down(key: &mut u8) -> bool {
    // TODO
    *key = 0;
    return false;
}
