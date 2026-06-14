use std::path::*;

mod cartrige;
pub use cartrige::*;

pub struct EmulState {
    pub cart: Cartrige 
}

impl EmulState {
    pub fn init(rom: &[u8]) -> Option<Self> {
        Some(EmulState { 
            cart: Cartrige::load_rom(rom)?
        })
    } 
}

