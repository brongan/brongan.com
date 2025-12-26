use rand::prelude::*;
use std::fmt::{self, Display};

use super::quirks::Quirks;
use super::screen::Screen;

#[derive(Default, Debug, Clone)]
pub struct CPU {
    pc: u16,
    index: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: Timer,
    sound_timer: Timer,
    registers: Registers,
    memory: Memory,
    screen: Screen,
    keypad_waiting: Option<u8>,
    vblank_waiting: bool,
}

impl CPU {
    pub fn new(rom: Option<&Vec<u8>>) -> Self {
        let font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let mut memory = Memory::default();
        memory.0[0x50..=0x9f].copy_from_slice(&font);
        if let Some(rom) = rom {
            let len = std::cmp::min(rom.len(), memory.0.len() - 0x200);
            memory.0[0x200..len + 0x200].copy_from_slice(&rom[0..len]);
        }
        Self {
            memory,
            pc: 0x200,
            ..Default::default()
        }
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer.get()
    }

    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer.get()
    }

    pub fn is_beep(&self) -> bool {
        self.sound_timer.get() > 0
    }

    pub fn get_registers(&self) -> &Registers {
        &self.registers
    }

    pub fn get_register(&self, register: Register) -> u8 {
        self.registers.get(register)
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_index(&self) -> u16 {
        self.index
    }

    pub fn get_stack(&self) -> [u16; 16] {
        self.stack
    }

    pub fn get_sp(&self) -> usize {
        self.sp
    }

    pub fn get_memory(&self) -> &[u8] {
        &self.memory.0
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }

    pub fn fetch(&self) -> u16 {
        let pc = self.pc as usize;
        (self.memory.0[pc] as u16) << 8 | self.memory.0[pc + 1] as u16
    }

    fn display(&mut self, x: Register, y: Register, height: u8, clipping: bool) {
        let vx = self.registers.get(x) as u16;
        let vy = self.registers.get(y) as u16;
        let start_x = vx % 64;
        let start_y = vy % 32;

        let vf = self.registers.get_mut(Register::VF);
        *vf = 0;
        for row in 0..height {
            let sprite_byte = self.memory.get(self.index + row as u16);
            let target_y = start_y + row as u16;
            let draw_y = if clipping {
                if target_y >= 32 {
                    break;
                }
                target_y
            } else {
                target_y % 32
            };

            for col in 0..8 {
                if (sprite_byte & (0x80 >> col)) != 0 {
                    let target_x = start_x + col as u16;
                    let draw_x = if clipping {
                        if target_x >= 64 {
                            continue;
                        }
                        target_x
                    } else {
                        target_x % 64
                    };

                    if self.screen.0[draw_y as usize][draw_x as usize] {
                        self.screen.0[draw_y as usize][draw_x as usize] = false;
                        *vf = 1;
                    } else {
                        self.screen.0[draw_y as usize][draw_x as usize] = true;
                    }
                }
            }
        }
    }

    fn execute(&mut self, instruction: Instruction, keypad: Keypad, quirks: &Quirks) -> u16 {
        use Instruction::*;
        use Register::VF;
        match instruction {
            Add(vx, val) => {
                *self.registers.get_mut(vx) = self.get_register(vx).wrapping_add(val);
            }
            AddIndex(vx) => self.index = self.index.wrapping_add(self.get_register(vx) as u16),
            AddReg(vx, vy) => {
                let vy = self.get_register(vy);
                let vx = self.registers.get_mut(vx);
                let (val, overflow) = vx.overflowing_add(vy);
                *vx = val;
                self.registers.set(VF, overflow as u8);
            }
            And(vx, vy) => {
                *self.registers.get_mut(vx) &= self.get_register(vy);
                if quirks.vf_reset {
                    self.registers.set(VF, 0);
                }
            }
            Assign(vx, vy) => self.registers.set(vx, self.get_register(vy)),
            BinaryDecimalConversion(vx) => {
                let val = self.get_register(vx);
                let hundreds = val / 100;
                let tens = (val / 10) % 10;
                let ones = val % 10;
                self.memory.set(self.index, hundreds);
                self.memory.set(self.index + 1, tens);
                self.memory.set(self.index + 2, ones);
            }
            Call(addr) => return addr,
            CallSubroutine(addr) => {
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                return addr;
            }
            CondSkip(cond) => {
                let cond = match cond {
                    Cond::Eq(vx, nn) => self.get_register(vx) == nn,
                    Cond::Neq(vx, nn) => self.get_register(vx) != nn,
                    Cond::EqReg(vx, vy) => self.get_register(vx) == self.get_register(vy),
                    Cond::NeqReg(vx, vy) => self.get_register(vx) != self.get_register(vy),
                };
                if cond {
                    return self.pc + 4;
                }
            }
            Display(x, y, height) => {
                self.display(x, y, height, quirks.clipping);
                if quirks.display_wait {
                    self.vblank_waiting = true;
                }
            }
            DisplayClear => self.screen = Screen::default(),
            FontCharacter(vx) => self.index = 0x50 + 5 * (self.get_register(vx) & 0xF) as u16,
            GetDelay(vx) => self.registers.set(vx, self.delay_timer.0),
            GetKey(vx) => {
                return match self.keypad_waiting {
                    Some(key) if !keypad.is_pressed(key) => {
                        self.keypad_waiting = None;
                        self.registers.set(vx, key);
                        self.pc + 2
                    }
                    Some(_) => self.pc,
                    None => {
                        for key in 0..16 {
                            if keypad.is_pressed(key) {
                                self.keypad_waiting = Some(key);
                                return self.pc;
                            }
                        }
                        self.pc
                    }
                };
            }
            Jump(addr) => return addr,
            JumpOffset(addr, vx) => {
                let offset = self.get_register(if quirks.jumping {
                    Register::V0
                } else {
                    Register::from_repr(vx).unwrap()
                }) as u16;
                return addr.wrapping_add(offset);
            }
            LoadMemory(x) => {
                let x = x as u8;
                for i in 0..=x {
                    let register = Register::from_repr(i).unwrap();
                    self.registers
                        .set(register, self.memory.get(self.index + i as u16));
                }
                if quirks.memory_increment {
                    self.index = self.index.wrapping_add(x as u16 + 1);
                }
            }
            Or(vx, vy) => {
                *self.registers.get_mut(vx) |= self.get_register(vy);
                if quirks.vf_reset {
                    self.registers.set(VF, 0);
                }
            }
            Rand(vx, nn) => self.registers.set(vx, rand::rng().random::<u8>() & nn),
            Return => {
                self.sp -= 1;
                return self.stack[self.sp];
            }
            SetDelay(vx) => self.delay_timer.set(self.get_register(vx)),
            SetIndex(val) => self.index = val,
            SetRegister(vx, val) => self.registers.set(vx, val),
            SetSound(vx) => self.sound_timer.set(self.get_register(vx)),
            ShiftLeft(vx, vy) => {
                let val = if quirks.shift_vy {
                    self.get_register(vy)
                } else {
                    self.get_register(vx)
                };
                let (val, overflow) = val.overflowing_mul(2);
                self.registers.set(vx, val);
                self.registers.set(Register::VF, overflow as u8);
            }
            ShiftRight(vx, vy) => {
                let val = if quirks.shift_vy {
                    self.get_register(vy)
                } else {
                    self.get_register(vx)
                };
                self.registers.set(vx, val >> 1);
                self.registers.set(Register::VF, val & 0b1);
            }
            SkipIfKey(vx) => {
                let key_index = self.get_register(vx) & 0xF;
                if keypad.is_pressed(key_index) {
                    return self.pc + 4;
                }
            }
            SkipIfNotKey(vx) => {
                let key_index = self.get_register(vx) & 0xF;
                if !keypad.is_pressed(key_index) {
                    return self.pc + 4;
                }
            }
            StoreMemory(x) => {
                let x = x as u8;
                for i in 0..=x {
                    let register = Register::from_repr(i).unwrap();
                    self.memory
                        .set(self.index + i as u16, self.get_register(register));
                }
                if quirks.memory_increment {
                    self.index = self.index.wrapping_add(x as u16 + 1);
                }
            }
            Subtract(vx, vy) => {
                let vx_val = self.get_register(vx);
                let vy_val = self.get_register(vy);
                let (val, underflow) = vx_val.overflowing_sub(vy_val);
                self.registers.set(vx, val);
                self.registers.set(Register::VF, !underflow as u8);
            }
            SubtractOther(vx, vy) => {
                let vx_val = self.get_register(vx);
                let vy_val = self.get_register(vy);
                let (val, underflow) = vy_val.overflowing_sub(vx_val);
                self.registers.set(vx, val);
                self.registers.set(Register::VF, !underflow as u8);
            }
            Xor(vx, vy) => {
                *self.registers.get_mut(vx) ^= self.get_register(vy);
                if quirks.vf_reset {
                    self.registers.set(VF, 0);
                }
            }
        }
        self.pc + 2
    }

    pub fn tick(&mut self, keypad: Keypad, quirks: &Quirks) {
        if self.vblank_waiting {
            return;
        }
        let instruction = self.fetch();
        let instruction = Instruction::decode(instruction).unwrap();
        self.pc = std::cmp::min(
            self.execute(instruction, keypad, quirks),
            self.memory.0.len() as u16 - 2,
        );
    }

    /// The caller should tick the timers at a 60hz frequency.
    pub fn tick_timers(&mut self) {
        self.delay_timer.tick();
        self.sound_timer.tick();
        self.vblank_waiting = false;
    }
}

#[derive(Debug, Clone)]
struct Memory([u8; 4096]);

impl Default for Memory {
    fn default() -> Self {
        Self([0; 4096])
    }
}

impl Memory {
    fn get(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn set(&mut self, addr: u16, val: u8) {
        self.0[addr as usize] = val
    }
}

#[derive(Default, Debug, Clone)]
pub struct Registers([u8; 16]);

impl Registers {
    pub fn get(&self, register: Register) -> u8 {
        self.0[register as u8 as usize]
    }

    fn set(&mut self, register: Register, val: u8) {
        self.0[register as u8 as usize] = val
    }

    fn get_mut(&mut self, register: Register) -> &mut u8 {
        &mut self.0[register as u8 as usize]
    }
}

#[derive(Debug, strum::FromRepr, Copy, Clone, strum::EnumIter, strum::Display)]
#[repr(u8)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xa,
    VB = 0xb,
    VC = 0xc,
    VD = 0xd,
    VE = 0xe,
    VF = 0xf,
}

#[derive(Default, Debug, Clone)]
struct Timer(u8);

impl Timer {
    fn tick(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    fn get(&self) -> u8 {
        self.0
    }

    fn set(&mut self, val: u8) {
        self.0 = val
    }
}

#[derive(Debug)]
pub enum Cond {
    /// VX equals NN
    Eq(Register, u8),
    /// VX does not equal NN
    Neq(Register, u8),
    /// VX equals VY
    EqReg(Register, Register),
    /// VX does not equal VY
    NeqReg(Register, Register),
}

impl Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cond::Eq(vx, nn) => write!(f, "{vx} == 0x{nn:02X}"),
            Cond::Neq(vx, nn) => write!(f, "{vx} != 0x{nn:02X}"),
            Cond::EqReg(vx, vy) => write!(f, "{vx} == {vy}"),
            Cond::NeqReg(vx, vy) => write!(f, "{vx} != {vy}"),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    /// Adds NN to VX (carry flag is not changed).
    Add(Register, u8),
    /// Adds VX to I. VF is not affected.
    AddIndex(Register),
    /// Adds VY to VX. VF is set to 1 when there's an overflow, and to 0 when there is not.
    AddReg(Register, Register),
    /// Sets VX to VX and VY. (bitwise AND operation).
    And(Register, Register),
    /// Sets VX to the value of VY.
    Assign(Register, Register),
    /// Stores the binary-coded decimal representation of VX,
    /// with the hundreds digit in memory at location in I,
    /// the tens digit at location I+1, and the ones digit at location I+2
    BinaryDecimalConversion(Register),
    /// Calls machine code routine at address NNN.
    Call(u16),
    /// Calls subroutine at NNN.
    CallSubroutine(u16),
    /// Clears the screen.
    DisplayClear,
    /// Skips the next instruction if Cond
    CondSkip(Cond),
    /// Draws a sprite at coordinate (VX, VY)
    Display(Register, Register, u8),
    /// Sets I to the location of the sprite for the character in VX(only consider the lowest nibble).
    /// Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    FontCharacter(Register),
    /// Sets VX to the value of the delay timer.
    GetDelay(Register),
    /// A key press is awaited, and then stored in VX
    /// (blocking operation, all instruction halted until next key event, delay and sound timers should continue processing)
    GetKey(Register),
    /// Jumps to address NNN.
    Jump(u16),
    /// Jumps to the address NNN plus V0.
    JumpOffset(u16, u8),
    /// Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified.
    LoadMemory(Register),
    /// Sets VX to VX or VY. (bitwise OR operation).
    Or(Register, Register),
    /// Sets VX to the result of a bitwise and
    /// operation on a random number (Typically: 0 to 255) and NN.
    Rand(Register, u8),
    /// Returns from a subroutine.
    Return,
    /// Sets the delay timer to VX.
    SetDelay(Register),
    /// Sets I to the address NNN.
    SetIndex(u16),
    /// Sets VX to NN
    SetRegister(Register, u8),
    /// Sets the sound timer to VX.
    SetSound(Register),
    /// Shifts VX to the left by 1, then sets VF to 1 if the most significant bit
    /// of VX prior to that shift was set, or to 0 if it was unset.
    ShiftLeft(Register, Register),
    /// Shifts VX to the right by 1,
    /// then stores the least significant bit of VX prior to the shift into VF
    ShiftRight(Register, Register),
    /// Skips the next instruction if the key stored in VX(only consider the lowest nibble) is pressed
    SkipIfKey(Register),
    /// Skips the next instruction if the key stored in VX(only consider the lowest nibble) is not pressed
    SkipIfNotKey(Register),
    /// Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
    StoreMemory(Register),
    /// VY is subtracted from VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VX >= VY and 0 if not).
    Subtract(Register, Register),
    /// Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VY >= VX).
    SubtractOther(Register, Register),
    /// Sets VX to VX xor VY
    Xor(Register, Register),
}

impl Instruction {
    pub fn decode(opcode: u16) -> Option<Self> {
        use Instruction::*;

        let nib1 = (opcode >> 12) as u8;
        let nib2 = (opcode >> 8) as u8 & 0xF;
        let addr = opcode & 0x0FFF;
        let x = Register::from_repr(nib2)?;
        let y = Register::from_repr((opcode as u8) >> 4)?;

        let nn = opcode as u8;
        let n = nn & 0xF;
        match nib1 {
            0x0 if opcode == 0x00E0 => Some(DisplayClear),
            0x0 if opcode == 0x00EE => Some(Return),
            0x0 => Some(Call(addr)), // Legacy SYS instruction
            0x1 => Some(Jump(opcode & 0x0FFF)),
            0x2 => Some(CallSubroutine(addr)),
            0x3 => Some(CondSkip(Cond::Eq(x, nn))),
            0x4 => Some(CondSkip(Cond::Neq(x, nn))),
            0x5 if n == 0x0 => Some(CondSkip(Cond::EqReg(x, y))),
            0x6 => Some(SetRegister(x, opcode as u8)),
            0x7 => Some(Add(x, opcode as u8)),
            0x8 if n == 0x0 => Some(Assign(x, y)),
            0x8 if n == 0x1 => Some(Or(x, y)),
            0x8 if n == 0x2 => Some(And(x, y)),
            0x8 if n == 0x3 => Some(Xor(x, y)),
            0x8 if n == 0x4 => Some(AddReg(x, y)),
            0x8 if n == 0x5 => Some(Subtract(x, y)),
            0x8 if n == 0x6 => Some(ShiftRight(x, y)),
            0x8 if n == 0x7 => Some(SubtractOther(x, y)),
            0x8 if n == 0xe => Some(ShiftLeft(x, y)),
            0x9 if n == 0x0 => Some(CondSkip(Cond::NeqReg(x, y))),
            0xA => Some(SetIndex(addr)),
            0xB => Some(JumpOffset(addr, nib2)),
            0xC => Some(Rand(x, nn)),
            0xD => Some(Display(x, y, n)),
            0xE if nn == 0x9E => Some(SkipIfKey(x)),
            0xE if nn == 0xA1 => Some(SkipIfNotKey(x)),
            0xF if nn == 0x07 => Some(GetDelay(x)),
            0xF if nn == 0x0A => Some(GetKey(x)),
            0xF if nn == 0x15 => Some(SetDelay(x)),
            0xF if nn == 0x18 => Some(SetSound(x)),
            0xF if nn == 0x1E => Some(AddIndex(x)),
            0xF if nn == 0x29 => Some(FontCharacter(x)),
            0xF if nn == 0x33 => Some(BinaryDecimalConversion(x)),
            0xF if nn == 0x55 => Some(StoreMemory(x)),
            0xF if nn == 0x65 => Some(LoadMemory(x)),
            _ => None,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;

        match self {
            Add(vx, val) => write!(f, "ADD {vx}, 0x{val:02X}"),
            AddIndex(vx) => write!(f, "ADD I, {vx}"),
            AddReg(vx, vy) => write!(f, "ADD {vx}, {vy}"),
            And(vx, vy) => write!(f, "AND {vx}, {vy}"),
            Assign(vx, vy) => write!(f, "LD {vx}, {vy}"),
            BinaryDecimalConversion(vx) => write!(f, "LD B, {vx}"),
            Call(addr) => write!(f, "SYS 0x{addr:03X}"), // CHIP-8 legacy op
            CallSubroutine(addr) => write!(f, "CALL 0x{addr:03X}"),
            CondSkip(cond) => write!(f, "SE {cond}"), // Assuming SE/SNE is handled by Cond
            DisplayClear => write!(f, "CLS"),
            Display(vx, vy, height) => write!(f, "DRW {vx}, {vy}, 0x{height:X}"),
            FontCharacter(vx) => write!(f, "LD F, {vx}"),
            GetDelay(vx) => write!(f, "LD {vx}, DT"),
            GetKey(vx) => write!(f, "LD {vx}, K"),
            Jump(addr) => write!(f, "JP 0x{addr:03X}"),
            JumpOffset(addr, vx) => write!(f, "JP {vx}, 0x{addr:03X}"),
            LoadMemory(vx) => write!(f, "LD {vx}, [I]"),
            Or(vx, vy) => write!(f, "OR {vx}, {vy}"),
            Rand(vx, nn) => write!(f, "RND {vx}, 0x{nn:02X}"),
            Return => write!(f, "RET"),
            SetDelay(vx) => write!(f, "LD DT, {vx}"),
            SetIndex(val) => write!(f, "LD I, 0x{val:03X}"),
            SetRegister(vx, val) => write!(f, "LD {vx}, 0x{val:02X}"),
            SetSound(vx) => write!(f, "LD ST, {vx}"),
            ShiftLeft(vx, _vy) => write!(f, "SHL {vx}"),
            ShiftRight(vx, _vy) => write!(f, "SHR {vx}"),
            SkipIfKey(vx) => write!(f, "SKP {vx}"),
            SkipIfNotKey(vx) => write!(f, "SKNP {vx}"),
            StoreMemory(vx) => write!(f, "LD [I], {vx}"),
            Subtract(vx, vy) => write!(f, "SUB {vx}, {vy}"),
            SubtractOther(vx, vy) => write!(f, "SUBN {vx}, {vy}"),
            Xor(vx, vy) => write!(f, "XOR {vx}, {vy}"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Keypad(pub u16);

impl Keypad {
    pub fn is_pressed(&self, key_index: u8) -> bool {
        (self.0 >> key_index) & 1 == 1
    }

    pub fn enable_key(&mut self, key_index: u8) {
        self.0 |= 1 << key_index;
    }

    pub fn disable_key(&mut self, key_index: u8) {
        self.0 &= !(1 << key_index);
    }
}
