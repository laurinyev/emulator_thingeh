use super::BusDevice;

pub struct RAM {
    mem: [u8; 0x800]
}

impl RAM {
    pub fn new() -> Self {
        RAM { mem: [0xFF; 0x800] }
    }
}

impl BusDevice for RAM {
    fn abus_read(&self, addr: u16) -> u8 {
        self.mem[(addr & 0x07FF) as usize]
    }
    fn abus_write(&mut self, addr: u16, val: u8) {
        self.mem[(addr & 0x07FF) as usize] = val
    }
    fn bbus_read(&self, addr: u16) -> u8 { 0 } // no WRAM on the BBUS
    fn bbus_write(&mut self, addr: u16, val: u8) {} // no WRAM on the BBUS
    fn bounds_abus(&self) -> (u16, u16) { (0x0000, 0x1FFF) }
    fn bounds_bbus(&self) -> (u16, u16) { (0,0) }
}
