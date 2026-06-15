use crate::NES::DualBus;

pub struct Cpu6502 {
    pc: u8, // program counter
    sp: u8, // stack pointer
    a: u8,  // accumulator
    x: u8,  // X register
    y: u8,  // Y register
}

impl Cpu6502 {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
        }
    }
}


