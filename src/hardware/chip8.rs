use rand::Rng;

use super::instruction_decoder::Instruction;
use std::fmt::Debug;

pub struct Chip8 {
    program_counter: u16,
    registers: [u8; REGISTER_COUNT],
    stack: Vec<u16>,
    i: u16,
    memory: [u8; MEMORY_SIZE],
    sound_timer: u8,
    delay_timer: u8,
}

const PROGRAM_START_ADDRESS: usize = 0x200;
const REGISTER_COUNT: usize = 16;
const MEMORY_SIZE: usize = 4096;
const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const SPRITE_WIDTH: u16 = 8;

const PIXEL_ON: u8 = 255;
const PIXEL_OFF: u8 = 0;

const FONT: [u8; 80] = [
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

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0u8; MEMORY_SIZE];
        memory[0..FONT.len()].copy_from_slice(&FONT);

        Chip8 {
            program_counter: 0x200,
            registers: [0; REGISTER_COUNT],
            stack: vec![],
            i: 0,
            memory,
            sound_timer: 0,
            delay_timer: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory[PROGRAM_START_ADDRESS..(PROGRAM_START_ADDRESS + rom.len())]
            .copy_from_slice(rom);
    }

    pub fn update_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        if self.sound_timer != 0 {
            // TODO: Play the noise
        }
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn step(&mut self, pixels: &mut [u8], key_states: &[bool; 16]) {
        let opcode = self.get_opcode();

        match Instruction::decode(opcode) {
            Instruction::NoOp => (),
            Instruction::Clear => {
                for pixel in pixels.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[PIXEL_OFF; 4]);
                }
            }
            Instruction::Ret => {
                self.program_counter = self.stack.pop().unwrap() - 2;
            }
            Instruction::Jmp(address) => self.program_counter = address - 2,
            Instruction::Call(address) => {
                self.stack.push(self.program_counter + 2);
                self.program_counter = address - 2;
            }
            Instruction::RegEqVal(register, value) => {
                if self.get_register(register) == value {
                    self.program_counter += 2;
                }
            }
            Instruction::RegNeqVal(register, value) => {
                if self.get_register(register) != value {
                    self.program_counter += 2;
                }
            }
            Instruction::RegEqReg(register_x, register_y) => {
                if self.get_register(register_x) == self.get_register(register_y) {
                    self.program_counter += 2;
                }
            }
            Instruction::SetRegVal(register, value) => self.set_register(register, value),
            Instruction::AddRegVal(register, value) => {
                self.set_register(register, self.get_register(register).wrapping_add(value))
            }
            Instruction::SetRegReg(register_x, register_y) => {
                self.set_register(register_x, self.get_register(register_y))
            }
            Instruction::SetRegOrReg(register_x, register_y) => {
                self.set_register(
                    register_x,
                    self.get_register(register_x) | self.get_register(register_y),
                );
            }
            Instruction::SetRegAndReg(register_x, register_y) => {
                self.set_register(
                    register_x,
                    self.get_register(register_x) & self.get_register(register_y),
                );
            }
            Instruction::SetRegXorReg(register_x, register_y) => {
                self.set_register(
                    register_x,
                    self.get_register(register_x) ^ self.get_register(register_y),
                );
            }
            Instruction::AddRegReg(register_x, register_y) => {
                let x = self.get_register(register_x);
                let y = self.get_register(register_y);

                let (new_x, is_carry) = x.overflowing_add(y);

                self.set_register(register_x, new_x);
                self.set_register(0xF, if is_carry { 1 } else { 0 });
            }
            Instruction::SubRegReg(register_x, register_y) => {
                let x = self.get_register(register_x);
                let y = self.get_register(register_y);

                let (new_x, is_borrow) = x.overflowing_sub(y);

                self.set_register(register_x, new_x);
                self.set_register(0xF, if is_borrow { 0 } else { 1 });
            }
            Instruction::ShiftRegRight(register) => {
                self.set_register(0xF, self.get_register(register) & 1);
                self.registers[register as usize] >>= 1;
            }
            Instruction::RevRegSubReg(register_x, register_y) => {
                let x = self.get_register(register_x);
                let y = self.get_register(register_y);

                let (new_x, is_borrow) = y.overflowing_sub(x);

                self.set_register(register_x, new_x);
                self.set_register(0xF, if is_borrow { 0 } else { 1 });
            }
            Instruction::ShiftRegLeft(register) => {
                self.set_register(0xF, (self.get_register(register) & 0x80) >> 7);
                self.registers[register as usize] <<= 1;
            }
            Instruction::RegNeqReg(register_x, register_y) => {
                if self.get_register(register_x) != self.get_register(register_y) {
                    self.program_counter += 2;
                }
            }
            Instruction::SetI(address) => self.i = address,
            Instruction::SetRegRand(register, value) => {
                self.set_register(register, rand::thread_rng().gen::<u8>() & value);
            }
            Instruction::JmpOffset(address) => {
                self.program_counter = (self.get_register(0) as u16) + address - 2
            }
            Instruction::Draw(register_x, register_y, sprite_height) => {
                let (origin_x, origin_y) =
                    (self.get_register(register_x), self.get_register(register_y));

                let mut collision = false;

                for row in 0..(sprite_height as u16) {
                    for col in 0..SPRITE_WIDTH {
                        let pixel_to_xor = self.get_sprite_pixel((row as u8, col as u8));
                        let x = ((col + origin_x as u16) as u16) % SCREEN_WIDTH;
                        let y = (row + origin_y as u16) % SCREEN_HEIGHT;

                        // scale each coord to handle 4 byte pixels
                        let index = (4 * x + y * 4 * 64) as usize;

                        for pixel in pixels[index..(index + 4)].iter_mut() {
                            if *pixel != 0 && pixel_to_xor {
                                collision = true;
                            }
                            *pixel ^= if pixel_to_xor { PIXEL_ON } else { PIXEL_OFF };
                        }
                    }
                }

                self.registers[0xF] = if collision { 1 } else { 0 };
            }
            Instruction::KeyDown(register) => {
                if key_states[register as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::KeyUp(register) => {
                if !key_states[register as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::SetRegDelay(register) => {
                self.set_register(register, self.delay_timer);
            }
            Instruction::SetRegKey(register) => {
                match key_states.iter().enumerate().find(|(_, &key)| key) {
                    Some((value, _)) => self.set_register(register, value as u8),
                    None => self.program_counter -= 2,
                }
            }
            Instruction::SetDelayReg(register) => {
                self.delay_timer = self.get_register(register);
            }
            Instruction::SetSoundReg(register) => {
                self.sound_timer = self.get_register(register);
            }
            Instruction::AddIReg(register) => {
                self.i = self.i.wrapping_add(self.get_register(register) as u16);
            }
            Instruction::SetISpriteReg(register) => {
                self.i = 5 * self.get_register(register) as u16;
            }
            Instruction::BCD(register) => {
                let mut value = self.get_register(register);
                self.memory[(self.i as usize) + 2] = value % 10;
                value /= 10;
                self.memory[(self.i as usize) + 1] = value % 10;
                value /= 10;
                self.memory[(self.i as usize)] = value % 10;
            }
            Instruction::Dump(register) => {
                let address = self.i as usize;
                self.memory[address..=(address + (register as usize))]
                    .copy_from_slice(&self.registers[0..=(register as usize)]);
            }
            Instruction::Load(register) => {
                let address = self.i as usize;
                self.registers[0..=(register as usize)]
                    .copy_from_slice(&self.memory[address..=(address + (register as usize))]);
            }
        }

        self.program_counter += 2;
    }

    #[inline]
    fn get_sprite_pixel(&self, (row, col): (u8, u8)) -> bool {
        let index = self.i + row as u16;
        self.memory[index as usize].reverse_bits() & (1 << col) != 0
    }

    #[inline(always)]
    fn get_register(&self, register: u8) -> u8 {
        self.registers[register as usize]
    }

    #[inline(always)]
    fn set_register(&mut self, register: u8, value: u8) {
        self.registers[register as usize] = value;
    }

    #[inline(always)]
    fn get_opcode(&self) -> u16 {
        (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize]) as u16
    }
}

impl Debug for Chip8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let start_memory_view_address = (self.program_counter - 5) as usize;
        let end_memory_view_address = (self.program_counter + 5) as usize;

        let memory_slice = &self.memory[start_memory_view_address..=end_memory_view_address];

        f.debug_struct("CPU")
            .field("program_counter", &self.program_counter)
            .field("i", &self.i)
            .field("registers", &self.registers)
            .field("Opcode", &self.get_opcode())
            .field("Instruction", &Instruction::decode(self.get_opcode()))
            .field("local_mem", &memory_slice)
            .finish()
    }
}
