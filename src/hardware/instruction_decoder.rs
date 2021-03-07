type Register = u8;
type Address = u16;

#[derive(Debug)]
pub enum Instruction {
    Clear,
    NoOp,
    Call(Address),
    Ret,
    Jmp(Address),
    RegEqVal(Register, u8),
    RegNeqVal(Register, u8),
    RegEqReg(Register, Register),
    SetRegVal(Register, u8),
    AddRegVal(Register, u8),
    SetRegReg(Register, Register),
    SetRegOrReg(Register, Register),
    SetRegAndReg(Register, Register),
    SetRegXorReg(Register, Register),
    AddRegReg(Register, Register),
    SubRegReg(Register, Register),
    ShiftRegRight(Register),
    RevRegSubReg(Register, Register),
    ShiftRegLeft(Register),
    RegNeqReg(Register, Register),
    SetI(Address),
    SetRegRand(Register, u8),
    JmpOffset(Address),
    Draw(Register, Register, u8),
    KeyUp(Register),
    KeyDown(Register),
    SetRegDelay(Register),
    SetRegKey(Register),
    SetDelayReg(Register),
    SetSoundReg(Register),
    AddIReg(Register),
    SetISpriteReg(Register),
    BCD(Register),
    Dump(Register),
    Load(Register),
}

impl Instruction {
    /// Given an opcode, decodes the instruction into the relevant parts
    pub fn decode(opcode: u16) -> Instruction {
        let register_x: Register = ((opcode & 0x0F00) >> 8) as u8;
        let register_y: Register = ((opcode & 0x00F0) >> 4) as u8;
        let address: Address = opcode & 0x0FFF;

        let nn = (opcode & 0xFF) as u8;
        let n = (opcode & 0xF) as u8;

        match opcode & 0xF000 {
            0 => match address {
                0xE0 => Instruction::Clear,
                0xEE => Instruction::Ret,
                _address => Instruction::NoOp, // Would be a machine specific subroutine on actual hardware
            },
            0x1000 => Instruction::Jmp(address),
            0x2000 => Instruction::Call(address),
            0x3000 => Instruction::RegEqVal(register_x, nn),
            0x4000 => Instruction::RegNeqVal(register_x, nn),
            0x5000 => Instruction::RegEqReg(register_x, register_y),
            0x6000 => Instruction::SetRegVal(register_x, nn),
            0x7000 => Instruction::AddRegVal(register_x, nn),
            0x8000 => match n {
                0x0 => Instruction::SetRegReg(register_x, register_y),
                0x1 => Instruction::SetRegOrReg(register_x, register_y),
                0x2 => Instruction::SetRegAndReg(register_x, register_y),
                0x3 => Instruction::SetRegXorReg(register_x, register_y),
                0x4 => Instruction::AddRegReg(register_x, register_y),
                0x5 => Instruction::SubRegReg(register_x, register_y),
                0x6 => Instruction::ShiftRegRight(register_x),
                0x7 => Instruction::RevRegSubReg(register_x, register_y),
                0xE => Instruction::ShiftRegLeft(register_x),
                _ => unimplemented!(),
            },
            0x9000 => Instruction::RegNeqReg(register_x, register_y),
            0xA000 => Instruction::SetI(address),
            0xB000 => Instruction::JmpOffset(address),
            0xC000 => Instruction::SetRegRand(register_x, nn),
            0xD000 => Instruction::Draw(register_x, register_y, n),
            0xE000 => match nn {
                0x9E => Instruction::KeyDown(register_x),
                0xA1 => Instruction::KeyUp(register_x),
                _ => unreachable!(),
            },
            0xF000 => match nn {
                0x07 => Instruction::SetRegDelay(register_x),
                0x0A => Instruction::SetRegKey(register_x),
                0x15 => Instruction::SetDelayReg(register_x),
                0x18 => Instruction::SetSoundReg(register_x),
                0x1E => Instruction::AddIReg(register_x),
                0x29 => Instruction::SetISpriteReg(register_x),
                0x33 => Instruction::BCD(register_x),
                0x55 => Instruction::Dump(register_x),
                0x65 => Instruction::Load(register_x),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
