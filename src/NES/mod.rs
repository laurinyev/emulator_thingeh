use std::{any::Any, path::*};

mod cartrige;
mod cpu;
pub use cartrige::*;
pub use cpu::*;

pub trait BusDevice: Any {
    fn abus_read(&self, addr: u16) -> u8;
    fn abus_write(&mut self, addr: u16, val: u8);
    fn bbus_read(&self, addr: u16) -> u8;
    fn bbus_write(&mut self, addr: u16, val: u8);
}

pub struct DualBus {
    pub devices: Vec<Box<dyn BusDevice>>
}

impl DualBus {
    fn new(devices: Vec<Box<dyn BusDevice>>) -> Self {
        Self{devices} 
    }
}

pub struct EmulState {
    pub cpu: Cpu6502,
    pub bus: DualBus
}

impl EmulState {
    pub fn init(rom: &[u8]) -> Option<Self> {
        let mut bus_devices = Vec::<Box<dyn BusDevice>>::new();

        bus_devices.push(Box::new(Cartrige::load_rom(rom)?));

        Some(EmulState { 
            bus: DualBus::new(bus_devices),
            cpu: Cpu6502::new()
        })
    } 
}

