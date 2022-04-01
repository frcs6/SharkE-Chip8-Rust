use super::drivers::*;
use super::threading::Processor;

const BEEP_FREQUENCY: u16 = 800;

pub struct CpuTimer {
    pub value: u8,
}

impl CpuTimer {
    pub fn new() -> Self {
        return Self { value: 0 };
    }
}

impl Processor for CpuTimer {
    fn execute(&mut self) -> u8 {
        if self.value > 0 {
            self.value -= 1;
        }
        return 1;
    }

    fn reset(&mut self) {
        self.value = 0;
    }
}

pub struct SoundTimer {
    beep: bool,
    pub cpu_timer: CpuTimer,
    frequency: f64,
}

impl SoundTimer {
    pub fn new(frequency: f64) -> Self {
        return Self {
            beep: false,
            cpu_timer: CpuTimer::new(),
            frequency: frequency,
        };
    }

    fn do_beep(&self) {
        let duration = 1000.0 * self.cpu_timer.value as f64 / self.frequency;
        sound_do_beep(BEEP_FREQUENCY, duration);
    }
}

impl Processor for SoundTimer {
    fn execute(&mut self) -> u8 {
        if !self.beep && self.cpu_timer.value > 0 {
            self.do_beep();
            self.beep = true;
        }

        let tick = self.cpu_timer.execute();

        if self.cpu_timer.value == 0 {
            self.beep = false;
        }
        return tick;
    }

    fn reset(&mut self) {
        self.cpu_timer.value = 0;
    }
}
