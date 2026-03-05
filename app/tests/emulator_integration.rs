use app::chip8::emulator::cpu::Keypad;
use app::chip8::emulator::emulator::Emulator;
use app::chip8::emulator::quirks::Quirks;

#[test]
fn test_emulator_new() {
    let rom = vec![0x12, 0x34];
    let emulator = Emulator::new(Some(rom.clone()));
    
    assert_eq!(Emulator::TARGET_IPS, 700);
    assert_eq!(emulator.quirks, Quirks::MODERN);
    assert_eq!(emulator.instruction_counter(), 0);
    
    assert_eq!(emulator.cpu().get_memory()[0x200], 0x12);
    assert_eq!(emulator.cpu().get_memory()[0x201], 0x34);
}

#[test]
fn test_emulator_step_timers() {
    let rom = vec![0x60, 0x3C, 0xF0, 0x15, 0x12, 0x04]; 
    let mut emulator = Emulator::new(Some(rom));
    let keypad = Keypad::default();

    emulator.step(keypad, Emulator::TARGET_IPS);
    
    assert_eq!(emulator.instruction_counter(), 700);
    assert!(emulator.cpu().get_delay_timer() <= 1);
}

#[test]
fn test_emulator_sound_timer_beep() {
    // ROM:
    // 0x600A - LD V0, 10
    // 0xF018 - LD ST, V0
    // 0x1204 - JP 0x204 (infinite loop)
    let rom = vec![0x60, 0x0A, 0xF0, 0x18, 0x12, 0x04];
    let mut emulator = Emulator::new(Some(rom));
    
    // Initial state: beep should be false (haven't executed LD ST yet)
    assert!(!emulator.is_beep());

    // Step 2 instructions (LD V0, 10; LD ST, V0)
    emulator.step(Keypad::default(), 2);
    assert!(emulator.is_beep());

    // Step enough cycles to cross 10 timer steps (approx 120 cycles)
    emulator.step(Keypad::default(), 120);
    
    assert_eq!(emulator.cpu().get_sound_timer(), 0);
    assert!(!emulator.is_beep());
}

#[test]
fn test_emulator_reset() {
    let mut emulator = Emulator::new(None);
    emulator.step(Keypad::default(), 10);
    assert_eq!(emulator.instruction_counter(), 10);
    
    emulator.reset();
    assert_eq!(emulator.instruction_counter(), 0);
}
