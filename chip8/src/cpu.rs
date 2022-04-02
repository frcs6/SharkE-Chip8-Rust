use super::constants::*;
use super::driver::Driver;
use super::threading::Processor;
use super::timers::*;
use std::cell::RefCell;
use std::rc::Rc;

const FONTS: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

const KB: usize = 1024;
const V_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const MEMORY_SIZE: usize = 4 * KB;
const PROGRAM_START: usize = 0x200;

pub struct Cpu {
    i: u16,
    v: Vec<u8>,
    program_counter: usize,
    stack: Vec<u16>,
    stack_pointer: u16,
    pub display: Vec<Vec<usize>>, // TODO: pub only for cpu_test !!!
    memory: Vec<u8>,
    delay_timer: Rc<RefCell<CpuTimer>>,
    sound_timer: Rc<RefCell<SoundTimer>>,
    driver: Rc<RefCell<dyn Driver>>,
    current_opcode: u16,
    rom: Vec<u8>,
}

impl Cpu {
    pub fn new(
        delay_timer: Rc<RefCell<CpuTimer>>,
        sound_timer: Rc<RefCell<SoundTimer>>,
        driver: Rc<RefCell<dyn Driver>>,
    ) -> Self {
        let mut cpu = Self {
            i: 0,
            v: vec![0; V_SIZE],
            program_counter: PROGRAM_START,
            stack: vec![0; STACK_SIZE],
            stack_pointer: 0,
            display: vec![vec![0; Y_SIZE]; X_SIZE],
            memory: vec![0; MEMORY_SIZE],
            delay_timer: delay_timer,
            sound_timer: sound_timer,
            driver: driver,
            current_opcode: 0,
            rom: Vec::new(),
        };
        cpu.initialize_memory();
        return cpu;
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        self.rom = rom;
        self.reset();
    }

    fn initialize_memory(&mut self) {
        self.memory = vec![0; MEMORY_SIZE];
        let mut index = 0;
        for font in &FONTS {
            self.memory[index] = *font;
            index += 1;
        }
        index = PROGRAM_START;
        for data in &self.rom {
            self.memory[index] = *data;
            index += 1;
        }
    }

    fn clear_display(&mut self) {
        // TODO: Maybe not the most efficient
        self.display = vec![vec![0; Y_SIZE]; X_SIZE];
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        return self.stack[self.stack_pointer as usize];
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn n(&mut self) -> u8 {
        return (self.current_opcode & 0x000F) as u8;
    }

    fn nn(&mut self) -> u8 {
        return (self.current_opcode & 0x00FF) as u8;
    }

    fn nnn(&mut self) -> u16 {
        return self.current_opcode & 0x0FFF;
    }

    fn x(&mut self) -> u8 {
        return ((self.current_opcode & 0x0F00) >> 8) as u8;
    }

    fn y(&mut self) -> u8 {
        return ((self.current_opcode & 0x00F0) >> 4) as u8;
    }

    fn instructions_0(&mut self) {
        let nnn = self.nnn();
        match nnn {
            0x0E0 => self.clear_display(),
            0x0EE => self.program_counter = self.pop() as usize,
            _ => {}
        }
    }

    fn instructions_1(&mut self) {
        self.program_counter = self.nnn() as usize;
    }

    fn instructions_2(&mut self) {
        self.push(self.program_counter as u16);
        self.program_counter = self.nnn() as usize;
    }

    fn instructions_3(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();
        if self.v[x] == nn {
            self.program_counter += 2;
        }
    }

    fn instructions_4(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();
        if self.v[x] != nn {
            self.program_counter += 2;
        }
    }

    fn instructions_5(&mut self) {
        let x = self.x() as usize;
        let y = self.y() as usize;
        if self.v[x] == self.v[y] {
            self.program_counter += 2;
        }
    }

    fn instructions_6(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();
        self.v[x] = nn;
    }

    fn instructions_7(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();
        self.v[x] += nn;
    }

    fn instructions_8(&mut self) {
        let n = self.n();
        let x = self.x() as usize;
        let y = self.y() as usize;
        match n {
            0x0 => self.v[x] = self.v[y],
            0x1 => self.v[x] |= self.v[y],
            0x2 => self.v[x] &= self.v[y],
            0x3 => self.v[x] ^= self.v[y],
            0x4 => {
                self.v[0xF] = if self.v[x] as u16 + self.v[y] as u16 > 0xFF {
                    1
                } else {
                    0
                };
                self.v[x] += self.v[y];
            }
            0x5 => {
                self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                self.v[x] -= self.v[y];
            }
            0x6 => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            0x7 => {
                self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y] - self.v[x];
            }
            0xE => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            }
            _ => {}
        }
    }

    fn instructions_9(&mut self) {
        let x = self.x() as usize;
        let y = self.y() as usize;
        if self.v[x] != self.v[y] {
            self.program_counter += 2;
        }
    }

    fn instructions_a(&mut self) {
        self.i = self.nnn();
    }

    fn instructions_b(&mut self) {
        self.program_counter = self.nnn() as usize + self.v[0] as usize;
    }

    fn instructions_c(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();
        let value = rand::random::<u8>();
        self.v[x] = value & nn;
    }

    fn instructions_d(&mut self) {
        let x = self.x() as usize;
        let y = self.y() as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let n = self.n();

        self.v[0xF] = 0;

        for row in 0..n {
            let pixels = self.memory[(self.i + row as u16) as usize];
            for col in 0..8 {
                if bit_value(pixels, 7 - col) {
                    let dx = (vx + col) as usize;
                    let dy = (vy + row) as usize;

                    if dx >= X_SIZE || dy >= Y_SIZE {
                        continue;
                    }

                    if self.display[dx][dy] == 1 {
                        self.v[0xF] = 1;
                    }

                    self.display[dx][dy] ^= 1;
                }
            }
        }

        self.driver.borrow_mut().video_fill_buffer(&self.display);
    }

    fn instructions_e(&mut self) {
        let x = self.x() as usize;
        let vx = self.v[x];
        let nn = self.nn();

        match nn {
            0x9E => {
                if self.driver.borrow_mut().input_is_key_down(vx) {
                    self.program_counter += 2;
                }
            }
            0xA1 => {
                if self.driver.borrow_mut().input_is_key_up(vx) {
                    self.program_counter += 2;
                }
            }
            _ => {}
        }
    }

    fn instructions_f(&mut self) {
        let x = self.x() as usize;
        let nn = self.nn();

        match nn {
            0x07 => self.v[x] = self.delay_timer.borrow().value,
            0x0A => {
                let mut key = 0u8;
                if self.driver.borrow_mut().input_is_any_key_down(&mut key) {
                    self.v[x] = key;
                } else {
                    self.program_counter += 2;
                }
            }
            0x15 => self.delay_timer.borrow_mut().value = self.v[x],
            0x18 => self.sound_timer.borrow_mut().cpu_timer.value = self.v[x],
            0x1E => self.i += self.v[x] as u16,
            0x29 => self.i = self.v[x] as u16 * 5,
            0x33 => {
                self.memory[(self.i + 0) as usize] = ((self.v[x] as u32 / 100) % 10) as u8;
                self.memory[(self.i + 1) as usize] = ((self.v[x] as u32 / 10) % 10) as u8;
                self.memory[(self.i + 2) as usize] = (self.v[x] as u32 % 10) as u8;
            }
            0x55 => {
                for i in 0..=x {
                    self.memory[self.i as usize + i] = self.v[i];
                }
            }
            0x65 => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i as usize + i];
                }
            }
            _ => {}
        }
    }
}

impl Processor for Cpu {
    fn execute(&mut self) -> u8 {
        self.current_opcode = (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16;
        self.program_counter += 2;
        let instructions_index = self.current_opcode >> 12;
        match instructions_index {
            0x0 => self.instructions_0(),
            0x1 => self.instructions_1(),
            0x2 => self.instructions_2(),
            0x3 => self.instructions_3(),
            0x4 => self.instructions_4(),
            0x5 => self.instructions_5(),
            0x6 => self.instructions_6(),
            0x7 => self.instructions_7(),
            0x8 => self.instructions_8(),
            0x9 => self.instructions_9(),
            0xa => self.instructions_a(),
            0xb => self.instructions_b(),
            0xc => self.instructions_c(),
            0xd => self.instructions_d(),
            0xe => self.instructions_e(),
            0xf => self.instructions_f(),
            _ => {}
        }
        return 1;
    }

    fn reset(&mut self) {
        self.current_opcode = 0;
        self.i = 0;
        self.program_counter = PROGRAM_START;
        self.stack_pointer = 0;
        self.v = vec![0; V_SIZE];
        self.stack = vec![0; STACK_SIZE];
        self.clear_display();
        self.initialize_memory();
    }
}

fn bit_value(value: u8, position: u8) -> bool {
    return (value & (1 << position)) != 0;
}
