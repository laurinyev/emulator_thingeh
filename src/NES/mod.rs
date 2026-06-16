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
    fn bounds_abus(&self) -> (u16, u16);
    fn bounds_bbus(&self) -> (u16, u16);
}

pub struct DualBus {
    pub devices: Vec<Box<dyn BusDevice>>
}

impl DualBus {
    fn new(devices: Vec<Box<dyn BusDevice>>) -> Self {
        Self{devices} 
    }

    pub fn abus_read(&self, addr: u16) -> u8 {
        for d in &self.devices{
            let bounds = d.bounds_abus();
            if addr >= bounds.0 && addr <= bounds.1 {
                return d.abus_read(addr);
            }
        }
        return 0;
    }
    pub fn abus_write(&mut self, addr: u16, val: u8) {
        for d in &mut self.devices{
            let bounds = d.bounds_abus();
            if addr >= bounds.0 && addr <= bounds.1 {
                return d.abus_write(addr,val);
            }
        }
    }
    pub fn bbus_read(&self, addr: u16) -> u8 {
        for d in &self.devices{
            let bounds = d.bounds_bbus();
            if addr >= bounds.0 && addr <= bounds.1 {
                return d.bbus_read(addr);
            }
        }
        return 0;
    }
    pub fn bbus_write(&mut self, addr: u16, val: u8) {
        for d in &mut self.devices{
            let bounds = d.bounds_bbus();
            if addr >= bounds.0 && addr <= bounds.1 {
                return d.bbus_write(addr,val);
            }
        }
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

