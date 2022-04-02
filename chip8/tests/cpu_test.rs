use chip8::constants::*;
use chip8::cpu::*;
use chip8::driver::*;
use chip8::threading::*;
use chip8::timers::*;
use std::cell::RefCell;
use std::fs::read;
use std::fs::read_to_string;
use std::rc::Rc;
use std::time::Duration;
use test_case::test_case;

const CPU_FREQ: f64 = 500.0;
const TIMER_FREQ: f64 = 60.0;

struct FakeDriver {}

impl FakeDriver {
    fn new() -> Self {
        return Self {};
    }
}

impl Driver for FakeDriver {
    fn sound_do_beep(&mut self, _frequency: u32, _duration: u32) {}

    fn video_fill_buffer(&mut self, _display: &Vec<Vec<usize>>) {}

    fn input_is_key_down(&mut self, _key: u8) -> bool {
        return false;
    }

    fn input_is_key_up(&mut self, _key: u8) -> bool {
        return true;
    }

    fn input_is_any_key_down(&mut self, _key: &mut u8) -> bool {
        return false;
    }
}

#[test_case("../test_roms/c8_test.c8" ,"../test_roms/c8_test.json" ; "c8_test")]
#[test_case("../test_roms/test_opcode.ch8" ,"../test_roms/test_opcode.json" ; "test_opcode")]
fn given_test_rom_when_tick_should_wrk(rom_path: &str, buffer_path: &str) {
    let driver = Rc::new(RefCell::new(FakeDriver::new()));
    let delay_timer = Rc::new(RefCell::new(CpuTimer::new()));
    let sound_timer = Rc::new(RefCell::new(SoundTimer::new(TIMER_FREQ, driver.clone())));
    let cpu = Rc::new(RefCell::new(Cpu::new(
        delay_timer.clone(),
        sound_timer.clone(),
        driver.clone(),
    )));

    let rom: Vec<u8> = read(rom_path).unwrap();
    cpu.borrow_mut().load(rom);

    let cpu_frequency = Frequency::new(CPU_FREQ, 1.0);
    let timer_frequency = cpu_frequency.get_sub_frequency(TIMER_FREQ, 1.0);
    let mut runner = ThreadRunner::new(
        cpu_frequency,
        vec![
            Thread::new(cpu_frequency, cpu.clone()),
            Thread::new(timer_frequency, delay_timer),
            Thread::new(timer_frequency, sound_timer),
        ],
    );
    runner.reset();

    let buffer_content = read_to_string(buffer_path).unwrap();
    let buffer_data: Vec<usize> = serde_json::from_str(&buffer_content).unwrap();

    runner.tick(Duration::from_secs(1));

    let mut index = 0;
    let mut test_ok = true;
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            if cpu.borrow().display[x][y] == 1 {
                print!("+",);
            } else {
                print!(" ",);
            }

            if cpu.borrow().display[x][y] != buffer_data[index] {
                test_ok = false;
            }
            index += 1;
        }
        println!("");
    }

    assert_eq!(test_ok, true);
}
