use super::*;
use strum::IntoEnumIterator;

#[test]
fn test_memory_get_set_basic() {
    let mut memory = Memory::default();
    memory.set(0x200, 42);
    assert_eq!(memory.get(0x200), 42);
}

#[test]
fn test_memory_get_set_lower_bound() {
    let mut memory = Memory::default();
    memory.set(0x000, 1);
    assert_eq!(memory.get(0x000), 1);
}

#[test]
fn test_memory_get_set_upper_bound() {
    let mut memory = Memory::default();
    assert_eq!(memory.0.len(), 4096);
    memory.set(0xFFF, 255);
    assert_eq!(memory.get(0xFFF), 255);
}

#[test]
#[should_panic]
fn test_memory_get_out_of_bounds() {
    let memory = Memory::default();
    memory.get(0x1000); // 4096
}

#[test]
#[should_panic]
fn test_memory_set_out_of_bounds() {
    let mut memory = Memory::default();
    memory.set(0x1000, 255); // 4096
}

#[test]
fn test_registers_get_set_basic() {
    let mut registers = Registers::default();
    registers.set(Register::V0, 10);
    assert_eq!(registers.get(Register::V0), 10);
}

#[test]
fn test_registers_get_set_vf() {
    let mut registers = Registers::default();
    registers.set(Register::VF, 255);
    assert_eq!(registers.get(Register::VF), 255);
}

#[test]
fn test_registers_get_unset_is_zero() {
    let registers = Registers::default();
    assert_eq!(registers.get(Register::V1), 0);
}

#[test]
fn test_registers_get_mut() {
    let mut registers = Registers::default();
    *registers.get_mut(Register::V5) = 55;
    assert_eq!(registers.get(Register::V5), 55);
}

#[test]
fn test_timer_initialization() {
    let timer = Timer::default();
    assert_eq!(timer.get(), 0);
}

#[test]
fn test_timer_set_get() {
    let mut timer = Timer::default();
    timer.set(10);
    assert_eq!(timer.get(), 10);
}

#[test]
fn test_timer_tick_decrements() {
    let mut timer = Timer::default();
    timer.set(10);
    timer.tick();
    assert_eq!(timer.get(), 9);
}

#[test]
fn test_timer_tick_to_zero() {
    let mut timer = Timer::default();
    timer.set(2);
    timer.tick();
    assert_eq!(timer.get(), 1);
    timer.tick();
    assert_eq!(timer.get(), 0);
}

#[test]
fn test_timer_tick_no_underflow() {
    let mut timer = Timer::default();
    timer.set(0);
    timer.tick();
    assert_eq!(timer.get(), 0);
}

#[test]
fn test_cpu_new_without_rom() {
    let cpu = CPU::new(None);
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.index, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.get_sound_timer(), 0);
    assert_eq!(cpu.get_delay_timer(), 0);
    
    // Assert all 16 registers initialize to 0
    for reg in Register::iter() {
        assert_eq!(cpu.registers.get(reg), 0);
    }
}

#[test]
fn test_cpu_new_with_rom() {
    let rom = vec![0x12, 0x34, 0x56, 0x78];
    let cpu_with_rom = CPU::new(Some(&rom));
    assert_eq!(cpu_with_rom.pc, 0x200);

    assert_eq!(cpu_with_rom.memory.get(0x200), 0x12);
    assert_eq!(cpu_with_rom.memory.get(0x201), 0x34);
    assert_eq!(cpu_with_rom.memory.get(0x202), 0x56);
    assert_eq!(cpu_with_rom.memory.get(0x203), 0x78);
    assert_eq!(cpu_with_rom.memory.get(0x204), 0x00); // Beyond ROM
}

#[test]
fn test_cpu_new_loads_fonts() {
    let cpu = CPU::new(None);
    // Standard font data occupies 0x000 to 0x1FF in many implementations, 
    // ours specifically loads into 0x050 offset.
    assert_eq!(cpu.memory.get(0x050), 0xF0); // Start of '0'
    assert_eq!(cpu.memory.get(0x054), 0xF0); // End of '0'
    assert_eq!(cpu.memory.get(0x09B), 0xF0); // Start of 'F'
    assert_eq!(cpu.memory.get(0x09F), 0x80); // End of 'F'
}

#[test]
fn test_display_size() {
    let cpu = CPU::new(None);
    assert_eq!(cpu.screen.0.len(), 32);
    assert_eq!(cpu.screen.0[0].len(), 64);
    assert_eq!(cpu.screen.0[31].len(), 64);
}

#[test]
fn test_keypad_full_mapping() {
    let mut keypad = Keypad::default();
    
    // Assert mapping for all 16 keys (0-F)
    keypad.enable_key(0x0); assert!(keypad.is_pressed(0x0));
    keypad.enable_key(0x1); assert!(keypad.is_pressed(0x1));
    keypad.enable_key(0x2); assert!(keypad.is_pressed(0x2));
    keypad.enable_key(0x3); assert!(keypad.is_pressed(0x3));
    keypad.enable_key(0x4); assert!(keypad.is_pressed(0x4));
    keypad.enable_key(0x5); assert!(keypad.is_pressed(0x5));
    keypad.enable_key(0x6); assert!(keypad.is_pressed(0x6));
    keypad.enable_key(0x7); assert!(keypad.is_pressed(0x7));
    keypad.enable_key(0x8); assert!(keypad.is_pressed(0x8));
    keypad.enable_key(0x9); assert!(keypad.is_pressed(0x9));
    keypad.enable_key(0xA); assert!(keypad.is_pressed(0xA));
    keypad.enable_key(0xB); assert!(keypad.is_pressed(0xB));
    keypad.enable_key(0xC); assert!(keypad.is_pressed(0xC));
    keypad.enable_key(0xD); assert!(keypad.is_pressed(0xD));
    keypad.enable_key(0xE); assert!(keypad.is_pressed(0xE));
    keypad.enable_key(0xF); assert!(keypad.is_pressed(0xF));

    // Release and check again
    keypad.disable_key(0x0); assert!(!keypad.is_pressed(0x0));
    keypad.disable_key(0xF); assert!(!keypad.is_pressed(0xF));
}

#[test]
fn test_cpu_stack_return_addresses() {
    let mut cpu = CPU::new(None);
    cpu.sp = 0;
    cpu.stack[0] = 0xABCD; // Simulate pushing 16-bit address
    cpu.sp = 1;

    let next_pc = cpu.execute(Instruction::Return, Keypad::default(), &Quirks::MODERN);
    assert_eq!(next_pc, 0xABCD);
    assert_eq!(cpu.sp, 0);
}

#[test]
fn test_cpu_fetch() {
    let rom = vec![0x1A, 0x2B, 0x3C, 0x4D];
    let cpu = CPU::new(Some(&rom));

    assert_eq!(cpu.pc, 0x200);
    let inst1 = cpu.fetch();
    assert_eq!(inst1, 0x1A2B); // Combines 0x1A (high) and 0x2B (low)
                               // PC is intentionally NOT incremented in `fetch` – it's the `tick` logic that controls PC progression
    assert_eq!(cpu.pc, 0x200);
}

#[test]
fn test_opcode_00e0_clear_display() {
    let mut cpu = CPU::new(None);
    cpu.screen.0[0][0] = true;
    let next_pc = cpu.execute(
        Instruction::DisplayClear,
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.screen.0[0][0], false);
    assert_eq!(next_pc, cpu.pc + 2);
}

#[test]
fn test_opcode_1nnn_jump() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    let next_pc = cpu.execute(
        Instruction::Jump(0x0600),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, 0x0600);
}

#[test]
fn test_opcode_2nnn_call_and_00ee_return() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.sp = 0;

    let sub_addr = 0x0400;
    let next_pc_call = cpu.execute(
        Instruction::CallSubroutine(sub_addr),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack[0], 0x202);
    assert_eq!(next_pc_call, sub_addr);

    cpu.pc = sub_addr;
    let next_pc_ret = cpu.execute(Instruction::Return, Keypad::default(), &Quirks::MODERN);
    assert_eq!(cpu.sp, 0);
    assert_eq!(next_pc_ret, 0x202);
}

#[test]
fn test_opcode_3xnn_skip_if_vx_eq_nn_true() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::Eq(Register::V0, 0x44)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_3xnn_skip_if_vx_eq_nn_false() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::Eq(Register::V0, 0x99)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}

#[test]
fn test_opcode_4xnn_skip_if_vx_neq_nn_true() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::Neq(Register::V0, 0x99)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_4xnn_skip_if_vx_neq_nn_false() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::Neq(Register::V0, 0x44)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}

#[test]
fn test_opcode_5xy0_skip_if_vx_eq_vy_true() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    cpu.registers.set(Register::V1, 0x44);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::EqReg(Register::V0, Register::V1)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_5xy0_skip_if_vx_eq_vy_false() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    cpu.registers.set(Register::V1, 0x55);
    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::EqReg(Register::V0, Register::V1)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}
#[test]
fn test_opcode_6xnn_set_vx() {
    let mut cpu = CPU::new(None);
    cpu.execute(
        Instruction::SetRegister(Register::V0, 0x42),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x42);
}

#[test]
fn test_opcode_7xnn_add_to_vx_no_overflow() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x10);
    cpu.execute(
        Instruction::Add(Register::V0, 0x05),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x15);
}

#[test]
fn test_opcode_7xnn_add_to_vx_unmodified_vf() {
    let mut cpu = CPU::new(None);
    // Carry flag (VF) remains completely unmodified.
    cpu.registers.set(Register::VF, 1);
    cpu.registers.set(Register::V0, 0xFF);
    cpu.execute(
        Instruction::Add(Register::V0, 0x01),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x00);
    assert_eq!(cpu.registers.get(Register::VF), 1); // Remained 1
    
    cpu.registers.set(Register::VF, 0);
    cpu.execute(
        Instruction::Add(Register::V0, 0x01),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::VF), 0); // Remained 0
}

#[test]
fn test_opcode_8xy0_set_vx_to_vy() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V1, 0x99);
    cpu.execute(
        Instruction::Assign(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x99);
}

#[test]
fn test_opcode_8xy1_vx_or_vy() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0b1010);
    cpu.registers.set(Register::V1, 0b0101);
    cpu.execute(
        Instruction::Or(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0b1111);
}

#[test]
fn test_opcode_8xy2_vx_and_vy() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0b1100);
    cpu.registers.set(Register::V1, 0b1010);
    cpu.execute(
        Instruction::And(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0b1000);
}

#[test]
fn test_opcode_8xy3_vx_xor_vy() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0b1100);
    cpu.registers.set(Register::V1, 0b1010);
    cpu.execute(
        Instruction::Xor(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0b0110);
}

#[test]
fn test_opcode_8xy4_vx_add_vy_with_carry() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0xFF);
    cpu.registers.set(Register::V1, 0x01);
    cpu.execute(
        Instruction::AddReg(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x00);
    assert_eq!(cpu.registers.get(Register::VF), 1);
}

#[test]
fn test_opcode_8xy4_vx_add_vy_without_carry() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x10);
    cpu.registers.set(Register::V1, 0x10);
    cpu.execute(
        Instruction::AddReg(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x20);
    assert_eq!(cpu.registers.get(Register::VF), 0);
}

#[test]
fn test_opcode_8xy5_vx_sub_vy_no_borrow() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x10);
    cpu.registers.set(Register::V1, 0x05);
    cpu.execute(
        Instruction::Subtract(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x0B);
    assert_eq!(cpu.registers.get(Register::VF), 1); // No borrow (VX > VY)
}

#[test]
fn test_opcode_8xy5_vx_sub_vy_with_borrow() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x05);
    cpu.registers.set(Register::V1, 0x10);
    cpu.execute(
        Instruction::Subtract(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0xF5);
    assert_eq!(cpu.registers.get(Register::VF), 0); // Borrow (VX < VY)
}

#[test]
fn test_opcode_8xy7_vy_sub_vx_with_borrow() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x05);
    cpu.registers.set(Register::V1, 0x10);
    cpu.execute(
        Instruction::SubtractOther(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0x0B);
    assert_eq!(cpu.registers.get(Register::VF), 1); // No borrow (VY > VX)
}

#[test]
fn test_opcode_8xy6_shift_right_modern_lsb_1() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V1, 0b1011);
    cpu.execute(
        Instruction::ShiftRight(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0b0101);
    assert_eq!(cpu.registers.get(Register::VF), 1);
}

#[test]
fn test_opcode_8xy6_shift_right_modern_lsb_0() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V1, 0b1010);
    cpu.execute(
        Instruction::ShiftRight(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::VF), 0);
}

#[test]
fn test_opcode_8xye_shift_left_modern_msb_1() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V1, 0b1011_0101);
    cpu.execute(
        Instruction::ShiftLeft(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 0b0110_1010);
    assert_eq!(cpu.registers.get(Register::VF), 1);
}

#[test]
fn test_opcode_8xye_shift_left_modern_msb_0() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V1, 0b0011_0101);
    cpu.execute(
        Instruction::ShiftLeft(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::VF), 0);
}

#[test]
fn test_opcode_9xy0_skip_if_vx_neq_vy_true() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    cpu.registers.set(Register::V1, 0x55);

    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::NeqReg(Register::V0, Register::V1)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_9xy0_skip_if_vx_neq_vy_false() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0x44);
    cpu.registers.set(Register::V1, 0x44);

    let next_pc = cpu.execute(
        Instruction::CondSkip(Cond::NeqReg(Register::V0, Register::V1)),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}
#[test]
fn test_opcode_annn_set_index() {
    let mut cpu = CPU::new(None);
    cpu.execute(
        Instruction::SetIndex(0x0500),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.index, 0x0500);
}

#[test]
fn test_opcode_bnnn_jump_offset_v0() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x42);
    let next_pc = cpu.execute(
        Instruction::JumpOffset(0x0600, 0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, 0x0642);
}

#[test]
fn test_opcode_cxnn_random() {
    let mut cpu = CPU::new(None);
    cpu.execute(
        Instruction::Rand(Register::V0, 0x0F),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0) & 0xF0, 0);
}

#[test]
fn test_opcode_dxyn_draw_basic_no_collision() {
    let mut cpu = CPU::new(None);
    cpu.memory.set(0x0500, 0b1000_0000);
    cpu.index = 0x0500;
    cpu.registers.set(Register::V0, 0);
    cpu.registers.set(Register::V1, 0);

    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.screen.0[0][0], true);
    assert_eq!(cpu.registers.get(Register::VF), 0);
    assert_eq!(cpu.index, 0x0500); // register I remains unmodified after the draw completes
}

#[test]
fn test_opcode_dxyn_draw_basic_collision() {
    let mut cpu = CPU::new(None);
    cpu.memory.set(0x0500, 0b1000_0000);
    cpu.index = 0x0500;
    cpu.registers.set(Register::V0, 0);
    cpu.registers.set(Register::V1, 0);

    cpu.screen.0[0][0] = true;
    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.screen.0[0][0], false);
    assert_eq!(cpu.registers.get(Register::VF), 1);
    assert_eq!(cpu.index, 0x0500); // register I remains unmodified
}

#[test]
fn test_opcode_ex9e_skip_if_key_pressed_true() {
    let mut cpu = CPU::new(None);
    let mut keypad = Keypad::default();
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0xA);

    keypad.enable_key(0xA);
    let next_pc = cpu.execute(
        Instruction::SkipIfKey(Register::V0),
        keypad,
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_ex9e_skip_if_key_pressed_false() {
    let mut cpu = CPU::new(None);
    let mut keypad = Keypad::default();
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0xA);

    keypad.disable_key(0xA);
    let next_pc = cpu.execute(
        Instruction::SkipIfKey(Register::V0),
        keypad,
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}

#[test]
fn test_opcode_exa1_skip_if_not_key_pressed_true() {
    let mut cpu = CPU::new(None);
    let mut keypad = Keypad::default();
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0xA);

    keypad.disable_key(0xA);
    let next_pc = cpu.execute(
        Instruction::SkipIfNotKey(Register::V0),
        keypad,
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 4);
}

#[test]
fn test_opcode_exa1_skip_if_not_key_pressed_false() {
    let mut cpu = CPU::new(None);
    let mut keypad = Keypad::default();
    cpu.pc = 0x200;
    cpu.registers.set(Register::V0, 0xA);

    keypad.enable_key(0xA);
    let next_pc = cpu.execute(
        Instruction::SkipIfNotKey(Register::V0),
        keypad,
        &Quirks::MODERN,
    );
    assert_eq!(next_pc, cpu.pc + 2);
}

#[test]
fn test_opcode_fx0a_wait_for_key() {
    let mut cpu = CPU::new(None);
    cpu.pc = 0x200;
    let mut keypad = Keypad::default();
    
    // 1. Initial execution - no key pressed. Should return same PC.
    let next_pc = cpu.execute(Instruction::GetKey(Register::V0), keypad, &Quirks::MODERN);
    assert_eq!(next_pc, 0x200);
    assert_eq!(cpu.keypad_waiting, None);

    // 2. Press a key. Should still return same PC but mark as waiting.
    keypad.enable_key(0x5);
    let next_pc = cpu.execute(Instruction::GetKey(Register::V0), keypad, &Quirks::MODERN);
    assert_eq!(next_pc, 0x200);
    assert_eq!(cpu.keypad_waiting, Some(0x5));

    // 3. Keep key pressed. Should still return same PC.
    let next_pc = cpu.execute(Instruction::GetKey(Register::V0), keypad, &Quirks::MODERN);
    assert_eq!(next_pc, 0x200);

    // 4. Release key. Should return PC + 2 and store key in V0.
    keypad.disable_key(0x5);
    let next_pc = cpu.execute(Instruction::GetKey(Register::V0), keypad, &Quirks::MODERN);
    assert_eq!(next_pc, 0x202);
    assert_eq!(cpu.registers.get(Register::V0), 0x5);
    assert_eq!(cpu.keypad_waiting, None);
}

#[test]
fn test_opcode_fx07_delay_timer_to_vx() {
    let mut cpu = CPU::new(None);
    cpu.delay_timer.set(42);
    cpu.execute(
        Instruction::GetDelay(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.registers.get(Register::V0), 42);
}

#[test]
fn test_opcode_fx15_set_delay_timer() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 55);
    cpu.execute(
        Instruction::SetDelay(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.delay_timer.get(), 55);
}

#[test]
fn test_opcode_fx18_set_sound_timer() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 55);
    cpu.execute(
        Instruction::SetSound(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.sound_timer.get(), 55);
}

#[test]
fn test_opcode_fx1e_add_vx_to_index() {
    let mut cpu = CPU::new(None);
    cpu.index = 0x0500;
    cpu.registers.set(Register::V0, 0x10);
    cpu.execute(
        Instruction::AddIndex(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.index, 0x0510);
}

#[test]
fn test_opcode_fx29_font_character() {
    let mut cpu = CPU::new(None);
    // The 'A' char
    cpu.registers.set(Register::V0, 0x0A);
    cpu.execute(
        Instruction::FontCharacter(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    // Font chars are stored at 0x50. 0x0A is 10. Chars are length 5. -> 0x50 + (10 * 5) = 0x50 + 50 = 0x50 + 0x32 = 0x82
    assert_eq!(cpu.index, 0x50 + 50);
}

#[test]
fn test_opcode_fx33_bcd() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 254);
    cpu.index = 0x0600;
    cpu.execute(
        Instruction::BinaryDecimalConversion(Register::V0),
        Keypad::default(),
        &Quirks::MODERN,
    );
    assert_eq!(cpu.memory.get(0x0600), 2);
    assert_eq!(cpu.memory.get(0x0601), 5);
    assert_eq!(cpu.memory.get(0x0602), 4);
}

#[test]
fn test_opcode_fx55_store_memory_unmodified_i() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 10);
    cpu.registers.set(Register::V1, 20);
    cpu.index = 0x0700;

    // Use memory_increment: false as per checklist "I is left unmodified"
    let quirks = Quirks { memory_increment: false, ..Quirks::MODERN };
    cpu.execute(
        Instruction::StoreMemory(Register::V1),
        Keypad::default(),
        &quirks,
    );
    assert_eq!(cpu.memory.get(0x0700), 10);
    assert_eq!(cpu.memory.get(0x0701), 20);
    assert_eq!(cpu.index, 0x0700); // Unmodified
}

#[test]
fn test_opcode_fx65_load_memory_unmodified_i() {
    let mut cpu = CPU::new(None);
    cpu.memory.set(0x0700, 10);
    cpu.memory.set(0x0701, 20);
    cpu.index = 0x0700;

    let quirks = Quirks { memory_increment: false, ..Quirks::MODERN };
    cpu.execute(
        Instruction::LoadMemory(Register::V1),
        Keypad::default(),
        &quirks,
    );
    assert_eq!(cpu.registers.get(Register::V0), 10);
    assert_eq!(cpu.registers.get(Register::V1), 20);
    assert_eq!(cpu.index, 0x0700); // Unmodified
}
macro_rules! test_decode {
        ($($name:ident: $opcode:expr => $pattern:pat),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert!(matches!(Instruction::decode($opcode), Some($pattern)));
                }
            )*
        };
    }

test_decode! {
    test_decode_00e0_clear: 0x00E0 => Instruction::DisplayClear,
    test_decode_00ee_return: 0x00EE => Instruction::Return,
    test_decode_0nnn_legacy_sys: 0x0123 => Instruction::Call(0x0123),
    test_decode_1nnn_jump: 0x1234 => Instruction::Jump(0x0234),
    test_decode_2nnn_call: 0x2234 => Instruction::CallSubroutine(0x0234),
    test_decode_3xnn_skip_eq: 0x3A55 => Instruction::CondSkip(Cond::Eq(Register::VA, 0x55)),
    test_decode_4xnn_skip_neq: 0x4B66 => Instruction::CondSkip(Cond::Neq(Register::VB, 0x66)),
    test_decode_5xy0_skip_reg_eq: 0x5AB0 => Instruction::CondSkip(Cond::EqReg(Register::VA, Register::VB)),
    test_decode_6xnn_set_reg: 0x6A42 => Instruction::SetRegister(Register::VA, 0x42),
    test_decode_7xnn_add_reg: 0x7A42 => Instruction::Add(Register::VA, 0x42),
    test_decode_8xy0_assign: 0x8120 => Instruction::Assign(Register::V1, Register::V2),
    test_decode_8xy1_or: 0x8121 => Instruction::Or(Register::V1, Register::V2),
    test_decode_8xy2_and: 0x8122 => Instruction::And(Register::V1, Register::V2),
    test_decode_8xy3_xor: 0x8123 => Instruction::Xor(Register::V1, Register::V2),
    test_decode_8xy4_add: 0x8124 => Instruction::AddReg(Register::V1, Register::V2),
    test_decode_8xy5_sub: 0x8125 => Instruction::Subtract(Register::V1, Register::V2),
    test_decode_8xy6_shr: 0x8126 => Instruction::ShiftRight(Register::V1, Register::V2),
    test_decode_8xy7_subn: 0x8127 => Instruction::SubtractOther(Register::V1, Register::V2),
    test_decode_8xye_shl: 0x812E => Instruction::ShiftLeft(Register::V1, Register::V2),
    test_decode_9xy0_skip_reg_neq: 0x9120 => Instruction::CondSkip(Cond::NeqReg(Register::V1, Register::V2)),
    test_decode_annn_set_index: 0xAFFF => Instruction::SetIndex(0x0FFF),
    test_decode_bnnn_jump_offset: 0xB123 => Instruction::JumpOffset(0x0123, 0x1),
    test_decode_cxnn_rand: 0xC123 => Instruction::Rand(Register::V1, 0x23),
    test_decode_dxyn_draw: 0xD123 => Instruction::Display(Register::V1, Register::V2, 3),
    test_decode_ex9e_skip_key: 0xE19E => Instruction::SkipIfKey(Register::V1),
    test_decode_exa1_skip_not_key: 0xE1A1 => Instruction::SkipIfNotKey(Register::V1),
    test_decode_fx07_get_delay: 0xF107 => Instruction::GetDelay(Register::V1),
    test_decode_fx0a_get_key: 0xF10A => Instruction::GetKey(Register::V1),
    test_decode_fx15_set_delay: 0xF115 => Instruction::SetDelay(Register::V1),
    test_decode_fx18_set_sound: 0xF118 => Instruction::SetSound(Register::V1),
    test_decode_fx1e_add_index: 0xF11E => Instruction::AddIndex(Register::V1),
    test_decode_fx29_font_char: 0xF129 => Instruction::FontCharacter(Register::V1),
    test_decode_fx33_bcd: 0xF133 => Instruction::BinaryDecimalConversion(Register::V1),
    test_decode_fx55_store_mem: 0xF155 => Instruction::StoreMemory(Register::V1),
    test_decode_fx65_load_mem: 0xF165 => Instruction::LoadMemory(Register::V1),
}

macro_rules! test_decode_invalid {
        ($($name:ident: $opcode:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert!(matches!(Instruction::decode($opcode), None));
                }
            )*
        };
    }

test_decode_invalid! {
    test_decode_invalid_f000: 0xF000,
    test_decode_invalid_e000: 0xE000,
    test_decode_invalid_5xy1: 0x5AB1, // 5XYN where N != 0
    test_decode_invalid_8xy8: 0x8128, // 8XYN where N is unsupported
    test_decode_invalid_9xy1: 0x9121, // 9XYN where N != 0
}

#[test]
fn test_quirk_vf_reset_disabled() {
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::VF, 1);
    cpu.execute(
        Instruction::Or(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks {
            vf_reset: false,
            ..Quirks::MODERN
        },
    );
    assert_eq!(cpu.registers.get(Register::VF), 1); // Should remain untouched
}

#[test]
fn test_quirk_memory_increment_disabled() {
    let mut cpu = CPU::new(None);
    cpu.index = 0x0500;
    cpu.execute(
        Instruction::StoreMemory(Register::V2),
        Keypad::default(),
        &Quirks {
            memory_increment: false,
            ..Quirks::MODERN
        },
    );
    assert_eq!(cpu.index, 0x0500); // Index is NOT incremented
}

#[test]
fn test_quirk_shift_vy_disabled() {
    // Older COSMAC VIP shifted VX in place, ignores VY.
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0b1010);
    cpu.registers.set(Register::V1, 0b1111);

    cpu.execute(
        Instruction::ShiftRight(Register::V0, Register::V1),
        Keypad::default(),
        &Quirks {
            shift_vy: false,
            ..Quirks::MODERN
        },
    );

    // VX (0b1010) shifted right is 0b0101 (5). If it used VY it would be 0b0111 (7)
    assert_eq!(cpu.registers.get(Register::V0), 0b0101);
}

#[test]
fn test_quirk_jumping_offset_vx() {
    // SUPER-CHIP jumped to NNN + VX rather than NNN + V0
    let mut cpu = CPU::new(None);
    cpu.registers.set(Register::V0, 0x01);
    cpu.registers.set(Register::V2, 0x05); // Target register

    let next_pc = cpu.execute(
        Instruction::JumpOffset(0x0600, 2),
        Keypad::default(),
        &Quirks {
            jumping: false, // Disables the modern V0 behavior
            ..Quirks::MODERN
        },
    );
    assert_eq!(next_pc, 0x0605); // Used V2
}

#[test]
fn test_quirk_display_wait_enabled() {
    let mut cpu = CPU::new(None);
    let quirks = Quirks {
        display_wait: true,
        ..Quirks::MODERN
    };
    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &quirks,
    );

    // When enabled, drawing a sprite should stall execution waiting on a vblank
    assert_eq!(cpu.vblank_waiting, true);
}

#[test]
fn test_quirk_display_wait_disabled() {
    let mut cpu = CPU::new(None);
    let quirks = Quirks {
        display_wait: false,
        ..Quirks::MODERN
    };
    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &quirks,
    );

    // When disabled (modern default), drawing is instantaneous
    assert_eq!(cpu.vblank_waiting, false);
}

#[test]
fn test_display_wrapping_enabled() {
    let mut cpu = CPU::new(None);
    // Turn clipping OFF to enable wrapping.
    let quirks = Quirks {
        clipping: false,
        ..Quirks::MODERN
    };

    // Sprite: 1100_0000 (2 pixels)
    cpu.memory.set(0x0500, 0b1100_0000);
    cpu.index = 0x0500;

    // Draw at X=63, Y=0.
    // Pixel 1: (63, 0)
    // Pixel 2: Should wrap to (0, 0)
    cpu.registers.set(Register::V0, 63);
    cpu.registers.set(Register::V1, 0);

    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &quirks,
    );

    assert_eq!(cpu.screen.0[0][63], true);
    assert_eq!(cpu.screen.0[0][0], true);
}

#[test]
fn test_display_clipping_enabled() {
    let mut cpu = CPU::new(None);
    // Turn clipping ON to prevent wrapping.
    let quirks = Quirks {
        clipping: true,
        ..Quirks::MODERN
    };

    // Sprite: 1100_0000 (2 pixels)
    cpu.memory.set(0x0500, 0b1100_0000);
    cpu.index = 0x0500;

    // Draw at X=63, Y=0.
    // Pixel 1: (63, 0)
    // Pixel 2: Should be CLIPPED off screen, and NOT appear at (0,0)
    cpu.registers.set(Register::V0, 63);
    cpu.registers.set(Register::V1, 0);

    cpu.execute(
        Instruction::Display(Register::V0, Register::V1, 1),
        Keypad::default(),
        &quirks,
    );

    assert_eq!(cpu.screen.0[0][63], true);
    assert_eq!(cpu.screen.0[0][0], false); // Did NOT wrap
}
