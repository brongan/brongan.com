use std::time::Duration;

use crate::chip8::cpu::{Keypad, CPU};
use crate::chip8::quirks::Quirks;
use crate::chip8::screen::Screen;

#[derive(Debug, Default)]
pub struct Emulator {
    pub running: bool,
    pub target_ips: u32,
    pub quirks: Quirks,
    cpu: CPU,
    rom: Option<Vec<u8>>,
    vsync_screen: Screen,
    cycle_accumulator: f32,
    timer_accumulator: f32,
    instruction_counter: u64,
}

impl Emulator {
    pub fn new(rom: Option<Vec<u8>>) -> Self {
        Self {
            running: rom.is_some(),
            target_ips: 700,
            quirks: Quirks::MODERN,
            cpu: CPU::new(rom.as_ref()),
            rom,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.instruction_counter = 0;
        self.cpu = CPU::new(None);
        self.cycle_accumulator = 0.0;
        self.timer_accumulator = 0.0;
    }

    pub fn update_rom(&mut self, rom: Vec<u8>) {
        self.rom = Some(rom);
        self.reload_rom();
    }

    pub fn reload_rom(&mut self) {
        self.instruction_counter = 0;
        self.cpu = CPU::new(self.rom.as_ref());
        self.cycle_accumulator = 0.0;
        self.timer_accumulator = 0.0;
        self.running = true;
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn screen(&self) -> &Screen {
        if self.quirks.display_wait {
            &self.vsync_screen
        } else {
            self.cpu.get_screen()
        }
    }

    pub fn instruction_counter(&self) -> u64 {
        self.instruction_counter
    }

    /// Emulate a given amount of time passing.
    pub fn update(&mut self, keypad: Keypad, dt: Duration) {
        if self.target_ips == 0 {
            return;
        }
        let dt = dt.as_secs_f32();
        self.cycle_accumulator += dt;
        let cycle_duration = 1.0 / self.target_ips as f32;

        let cycles = (self.cycle_accumulator / cycle_duration) as u32;
        if cycles > 0 {
            self.step(keypad, cycles);
            self.cycle_accumulator -= cycles as f32 * cycle_duration;
        }
    }

    /// Emulate a given number of instructions.
    pub fn step(&mut self, keypad: Keypad, instructions: u32) {
        if self.target_ips == 0 {
            return;
        }
        let cycle_duration = 1.0 / self.target_ips as f32;
        let timer_step = 1.0 / 60.0;

        for _ in 0..instructions {
            self.cpu.tick(keypad, &self.quirks);
            self.instruction_counter += 1;
            self.timer_accumulator += cycle_duration;

            while self.timer_accumulator >= timer_step {
                self.vsync_screen = self.cpu.get_screen().to_owned();
                self.cpu.tick_timers();
                self.timer_accumulator -= timer_step;
            }
        }
    }
}
