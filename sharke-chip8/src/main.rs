mod sdl2_drivers;

use std::env;
use chip8::cpu::Cpu;
use chip8::threading::*;
use chip8::timers::*;
use sdl2_drivers::*;
use std::cell::RefCell;
use std::fs::read;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

const CPU_FREQ: f64 = 500.0;
const TIMER_FREQ: f64 = 60.0;

fn main() {
    let args: Vec<String> = env::args().collect();
    run(&args[1]);
}

fn run(rom_path: &String) {
    let driver = Rc::new(RefCell::new(Sd2lDriver::new()));

    let delay_timer = Rc::new(RefCell::new(CpuTimer::new()));
    let sound_timer = Rc::new(RefCell::new(SoundTimer::new(TIMER_FREQ, driver.clone())));
    let cpu = Rc::new(RefCell::new(Cpu::new(
        delay_timer.clone(),
        sound_timer.clone(),
        driver.clone(),
    )));

    let rom: Vec<u8> = read(rom_path).unwrap();
    cpu.borrow_mut().load(rom);
    println!("'{}' loaded", rom_path);

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

    let sdl_context = sdl2::init().unwrap();

    let rom_name = Path::new(&rom_path).file_name().unwrap().to_str().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window(rom_name, SCREEN_W, SCREEN_H).build().unwrap();
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let timer_subsystem = sdl_context.timer().unwrap();
    let mut start_counter = timer_subsystem.ticks();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    break 'main;
                }
                sdl2::event::Event::KeyDown { keycode, .. } => match keycode.unwrap() {
                    sdl2::keyboard::Keycode::Escape => {
                        break 'main;
                    }
                    _ => {}
                },
                _ => {}
            }

            driver.borrow_mut().pool_event(&event);
        }

        let end_counter = timer_subsystem.ticks();
        let elapsed = end_counter - start_counter;

        runner.tick(Duration::from_millis(elapsed as u64));
        
        canvas.clear();
        driver.borrow_mut().draw(&mut canvas);
        canvas.present();        

        let fps = 1000.0 / elapsed as f32;
        let title = format!("{} - {} fps", rom_name, fps);
        canvas.window_mut().set_title(title.as_str()).unwrap();
        start_counter = end_counter;
    }
}
