use super::*;
use crate::chip8::emulator::cpu::{Instruction, Register};

#[test]
fn test_emulator_new() {
    let rom = vec![0x12, 0x34];
    let emulator = Emulator::new(Some(rom.clone()));
    
    // Assert initialized settings
    assert_eq!(emulator.target_ips, 700);
    assert_eq!(emulator.quirks, Quirks::MODERN);
    assert_eq!(emulator.instruction_counter, 0);
    assert_eq!(emulator.cycle_accumulator, 0.0);
    
    // Emulators CPU should have the ROM loaded
    assert_eq!(emulator.cpu().get_memory()[0x200], 0x12);
    assert_eq!(emulator.cpu().get_memory()[0x201], 0x34);
}

#[test]
fn test_emulator_step_timers() {
    // Script:
    // 0x603C - Set V0 to 60 (0x3C)
    // 0xF015 - Set delay timer to V0
    // 0x1204 - Jump to self (0x204) infinitely to stall while timers tick
    let rom = vec![0x60, 0x3C, 0xF0, 0x15, 0x12, 0x04]; 
    let mut emulator = Emulator::new(Some(rom));
    let keypad = Keypad::default();

    // Let's step exactly the target_ips (700 instructions)
    emulator.step(keypad, emulator.target_ips);
    
    // The first instruction sets V0, second sets timer to 60.
    // The remaining 698 instructions are just looping.
    // At 700 ips and 60hz timers, we should have lost exactly 60 logical ticks
    assert_eq!(emulator.instruction_counter, 700);
    // Using <= 1 due to floating point timing accumulation
    assert!(emulator.cpu().get_delay_timer() <= 1);
}

#[test]
fn test_emulator_sound_timer_beep() {
    let mut emulator = Emulator::new(None);
    
    // V0 = 10
    emulator.cpu.execute(
        Instruction::SetRegister(Register::V0, 10),
        Keypad::default(),
        &Quirks::MODERN,
    );
    // ST = V0
    emulator.cpu.execute(
        Instruction::SetSound(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    
    assert!(emulator.is_beep());

    // Step enough cycles to cross 10 timer steps (approx 120 cycles)
    emulator.step(Keypad::default(), 120);
    
    assert_eq!(emulator.cpu().get_sound_timer(), 0);
    assert!(!emulator.is_beep());
}
