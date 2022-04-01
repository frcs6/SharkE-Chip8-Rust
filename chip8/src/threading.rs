use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const SYNC_DURATION: Duration = Duration::from_secs(1);

#[derive(Copy, Clone)]
pub struct Frequency {
    value: f64,
    divider: f64,
}

impl Frequency {
    pub fn new(frequency: f64, divider: f64) -> Self {
        return Self {
            value: frequency,
            divider: divider,
        };
    }

    pub fn get_sub_frequency(&self, frequency: f64, divider: f64) -> Self {
        return Frequency::new(frequency, divider * self.value / frequency);
    }
}

pub trait Processor {
    fn execute(&mut self) -> u8;
    fn reset(&mut self);
}

pub struct Thread {
    clock: f64,
    frequency: Frequency,
    processor: Rc<RefCell<dyn Processor>>,
}

impl Thread {
    pub fn new(frequency: Frequency, processor: Rc<RefCell<dyn Processor>>) -> Self {
        return Self {
            clock: 0.0,
            frequency: frequency,
            processor: processor,
        };
    }

    fn tick(&mut self) {
        let tick = self.processor.borrow_mut().execute();
        self.clock += tick as f64 * self.frequency.divider;
    }

    fn reset(&mut self) {
        self.processor.borrow_mut().reset();
        self.clock = 0.0;
    }

    fn synchronize_clock(&mut self, main_clock: f64) {
        self.clock -= main_clock;
    }
}

pub struct ThreadRunner {
    clock: f64,
    elapsed: Duration,
    incomplete_tick: f64,
    frequency: Frequency,
    threads: Vec<Thread>,
}

impl ThreadRunner {
    pub fn new(frequency: Frequency, threads: Vec<Thread>) -> Self {
        return Self {
            clock: 0.0,
            elapsed: Duration::ZERO,
            incomplete_tick: 0.0,
            frequency: frequency,
            threads: threads,
        };
    }

    pub fn tick(&mut self, elapsed: Duration) {
        self.incomplete_tick += self.frequency.value * elapsed.as_secs_f64();
        let complete_tick_f64 = self.incomplete_tick.trunc();
        self.incomplete_tick -= complete_tick_f64;
        let complete_tick = complete_tick_f64 as u64;

        for _i in 0..complete_tick {
            let next_clock = self.clock + self.frequency.divider;

            let mut latest_processor_clock = f64::MAX;
            loop {
                latest_processor_clock = f64::MAX;

                for thread in self.threads.iter_mut() {
                    if thread.clock < next_clock {
                        thread.tick();
                    }

                    latest_processor_clock = latest_processor_clock.min(thread.clock);
                }

                if latest_processor_clock >= next_clock {
                    break;
                }
            }

            self.clock = next_clock;
        }

        self.elapsed += elapsed;
        if self.elapsed < SYNC_DURATION {
            return;
        }

        self.elapsed -= SYNC_DURATION;

        for thread in self.threads.iter_mut() {
            thread.synchronize_clock(self.clock);
        }

        self.clock = 0.0;
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.incomplete_tick = 0.0;
        for thread in self.threads.iter_mut() {
            thread.reset();
        }
    }
}

#[cfg(test)]
mod frequency_tests {
    use super::Frequency;

    #[test]
    fn given_parameters_when_new_frequency_should_set_properties() {
        let frequency = Frequency::new(200.0, 2.0);

        assert_eq!(frequency.divider, 2.0);
        assert_eq!(frequency.value, 200.0);
    }

    #[test]
    fn given_frequency_when_get_sub_frequency_should_compute_divider() {
        let frequency = Frequency::new(200.0, 2.0);

        let sub_frequency = frequency.get_sub_frequency(50.0, 1.0);

        assert_eq!(sub_frequency.divider, 4.0);
        assert_eq!(sub_frequency.value, 50.0);
    }
}

#[cfg(test)]
mod thread_tests {
    use super::Frequency;
    use super::Processor;
    use super::Thread;
    use super::ThreadRunner;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    const EXECUTE_STEP: u8 = 2;

    struct FakeProcessor {
        execute_call_count: u8,
    }

    impl FakeProcessor {
        fn new() -> Self {
            return Self {
                execute_call_count: 0,
            };
        }
    }

    impl Processor for FakeProcessor {
        fn execute(&mut self) -> u8 {
            self.execute_call_count += 1;
            return EXECUTE_STEP;
        }

        fn reset(&mut self) {
            self.execute_call_count = 0;
        }
    }

    #[test]
    fn given_thread_when_tick_should_execute() {
        let frequency: Frequency = Frequency::new(500.0, 4.0);
        let processor = Rc::new(RefCell::new(FakeProcessor::new()));
        let mut thread = Thread::new(frequency, processor.clone());

        thread.tick();

        assert_eq!(processor.borrow().execute_call_count, 1);
    }

    #[test]
    fn given_thread_when_tick_should_inc_clock() {
        let frequency: Frequency = Frequency::new(500.0, 4.0);
        let processor = Rc::new(RefCell::new(FakeProcessor::new()));
        let mut thread = Thread::new(frequency, processor);
        let expected_clock = EXECUTE_STEP as f64 * frequency.divider;

        thread.tick();

        assert_eq!(thread.clock, expected_clock);
    }

    #[test]
    fn given_thread_when_reset_should_reset_state() {
        let frequency: Frequency = Frequency::new(500.0, 4.0);
        let processor = Rc::new(RefCell::new(FakeProcessor::new()));
        let mut thread = Thread::new(frequency, processor);
        thread.tick();

        thread.reset();

        assert_eq!(thread.clock, 0.0);
    }

    #[test]
    fn given_thread_when_reset_should_adjust_clock() {
        let frequency: Frequency = Frequency::new(500.0, 4.0);
        let main_clock = 50.0;
        let expected_clock = EXECUTE_STEP as f64 * frequency.divider - main_clock;
        let processor = Rc::new(RefCell::new(FakeProcessor::new()));
        let mut thread = Thread::new(frequency, processor);
        thread.tick();

        thread.synchronize_clock(main_clock);

        assert_eq!(thread.clock, expected_clock);
    }

    #[test]
    fn given_runner_when_tick_should_increment_clock() {
        let frequency: Frequency = Frequency::new(250.0, 1.0);
        let mut runner = ThreadRunner::new(frequency, Vec::new());
        let expected_tick = 5.0;
        let expected_clock = expected_tick * frequency.divider;
        let duration_secs = expected_tick / frequency.value;

        runner.tick(Duration::from_secs_f64(duration_secs));

        assert_eq!(runner.clock, expected_clock);
    }

    #[test]
    fn fiven_runner_when_tick_should_tick_threads() {
        let frequency: Frequency = Frequency::new(250.0, 1.0);
        let sub_frequency = frequency.get_sub_frequency(50.0, 1.0);
        let mut runner = ThreadRunner::new(
            frequency,
            vec![
                Thread::new(frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
                Thread::new(sub_frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
            ],
        );
        let expected_tick1 = 6.0;
        let expected_tick2 = 2.0;
        let expected_clock1 = expected_tick1 * frequency.divider;
        let expected_clock2 = expected_tick2 * sub_frequency.divider;
        let duration_secs = expected_tick1 / frequency.value;

        runner.tick(Duration::from_secs_f64(duration_secs));

        assert_eq!(runner.threads[0].clock, expected_clock1);
        assert_eq!(runner.threads[1].clock, expected_clock2);
    }

    #[test]
    fn fiven_runner_when_tick_should_reset_threads() {
        let frequency: Frequency = Frequency::new(250.0, 1.0);
        let sub_frequency = frequency.get_sub_frequency(50.0, 1.0);
        let mut runner = ThreadRunner::new(
            frequency,
            vec![
                Thread::new(frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
                Thread::new(sub_frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
            ],
        );
        let duration_secs = 5.0 / frequency.value;
        runner.tick(Duration::from_secs_f64(duration_secs));

        runner.reset();

        assert_eq!(runner.threads[0].clock, 0.0);
        assert_eq!(runner.threads[1].clock, 0.0);
    }

    #[test]
    fn fiven_runner_when_tick_should_reset_sync_clocks() {
        let frequency: Frequency = Frequency::new(250.0, 1.0);
        let sub_frequency = frequency.get_sub_frequency(50.0, 1.0);
        let mut runner = ThreadRunner::new(
            frequency,
            vec![
                Thread::new(frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
                Thread::new(sub_frequency, Rc::new(RefCell::new(FakeProcessor::new()))),
            ],
        );

        runner.tick(Duration::from_secs(1));

        assert_eq!(runner.threads[0].clock, 0.0);
        assert_eq!(runner.threads[1].clock, 0.0);
    }
}
