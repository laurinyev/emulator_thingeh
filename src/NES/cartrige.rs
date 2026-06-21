use crate::NES::BusDevice;

pub struct CartMemRefsMut<'a>{
    prg_rom: &'a     [u8], // program ROM
    chr_mem: &'a mut [u8], // character ROM/RAM
    prg_ram: &'a mut [u8], // program RAM
    nt_ram:  &'a mut [u8]  // CIRAM, emulated as if it was on the cartrige
}

pub struct CartMemRefs<'a>{
    prg_rom: &'a [u8], // program ROM
    chr_mem: &'a [u8], // character ROM/RAM
    prg_ram: &'a [u8], // program RAM
    nt_ram:  &'a [u8]  // CIRAM, emulated as if it was on the cartrige
}

pub enum NametableMirroring {
    FourScreen,
    Horizontal,
    Vertical,
    SingleScreenUpper,
    SingleScreenLower
}

pub trait Mapper {
    fn abus_read(&self, addr: u16, refs: CartMemRefs) -> u8;
    fn abus_write(&mut self, addr: u16, val: u8, refs: CartMemRefsMut);
    fn bbus_read(&self, addr: u16, refs: CartMemRefs) -> u8;
    fn bbus_write(&mut self, addr: u16, val: u8, refs: CartMemRefsMut);
    fn mapper_name(&self) -> String;
}

pub struct NromMapper {
    pub mirroring: NametableMirroring
}

impl NromMapper{
    pub fn new(nt_mirroring_vertical: bool, nt_mirroring_alt: bool) -> Option<Self> {
        let mirroring = 
            if nt_mirroring_alt { NametableMirroring::FourScreen} else {
                if nt_mirroring_vertical { NametableMirroring:: Vertical} else { NametableMirroring::Horizontal }
            };

        Some(Self{
            mirroring: mirroring
        })
    }
}

impl Mapper for NromMapper {
    fn abus_read(&self, addr: u16, refs: CartMemRefs) -> u8 {
        if addr >= 0x6000 && addr <= 0x7FFF {
            if refs.prg_ram.len() == 0 {
                return 0;
            }
            let realaddr = (addr - 0x6000) as usize % refs.prg_ram.len();
            return refs.prg_ram[realaddr];
        } else if addr >= 0x8000 {
            let realaddr = (addr - 0x8000) as usize;

            if realaddr <= refs.prg_rom.len() {
                return refs.prg_rom[realaddr];
            } else {
                return refs.prg_rom[realaddr % refs.prg_rom.len()];
            }
        } 
        return 0;
    }

    fn abus_write(&mut self, addr: u16, val: u8, mut refs: CartMemRefsMut) {
        if addr >= 0x6000 && addr <= 0x7FFF {
            let realaddr = (addr - 0x6000) as usize % refs.prg_ram.len();
            refs.prg_ram[realaddr] = val;
        }
    } 

    fn bbus_read(&self, addr: u16, refs: CartMemRefs) -> u8 {
        if addr <= 0x1FFF {
            let realaddr = addr as usize;

            return refs.chr_mem[realaddr];
        }

        return 0; // TODO: CIRAM 
    }

    // NROM doesn't map anything writable on the bbus
    fn bbus_write(&mut self, addr: u16, val: u8, mut refs: CartMemRefsMut) {} 
    
    fn mapper_name(&self) -> String {
        "NROM".to_string()
    }
}

fn new_mapper(mapper_num: u8,nt_mirroring_vertical: bool, nt_mirroring_alt: bool) -> Option<Box<dyn Mapper>> {
    match mapper_num {
        0 => Some(Box::new(NromMapper::new(nt_mirroring_vertical, nt_mirroring_alt)?)),
        _ => None
    }
}

pub struct Cartrige {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub nt_ram:  Vec<u8>,
    pub mapper: Box<dyn Mapper>,
}

impl Cartrige {
    pub fn load_rom(rom: &[u8]) -> Option<Self> {
        if rom[0] == 0x4E && rom[1] == 0x45 && rom[2] == 0x53 && rom[3] == 0x1A { // "NES\x1A"
            let mapper_num = ((rom[6] & 0xF0) >> 4) | (rom[7] & 0xF0);

            let nt_mirroring_vertical = rom[6] & 1 != 0;
            let nt_mirroring_alt      = rom[6] & (1<<3) != 0;
            let has_prgram            = rom[6] & (1<<2) != 0;

            let prgrom_size  = 16384 * rom[4] as usize;
            let chrrom_size  = 8192  * rom[5] as usize;
            let trainer_size = if rom[6] & (1<<2) == 0 { 0 } else { 512 };

            let mut prgram_size  = 8192 * rom[7] as usize;
            if has_prgram && prgram_size == 0 {
                prgram_size = 8192;
            }

            if let Some(mapper) = new_mapper(mapper_num, nt_mirroring_vertical, nt_mirroring_alt) {
                let prg_start = 16 + trainer_size;
                let prg_end = prg_start + prgrom_size;

                let chr_start = prg_end;
                let chr_end = chr_start + chrrom_size;

                // let's rather not panic on a corrupted header
                if rom.len() < chr_end {
                    return None;
                }
                
                let mut prgram = Vec::new();
                prgram.reserve(prgram_size);
                let mut ntram = Vec::new(); // RAM for nametable, emulated as if it's on the cartrige
                ntram.reserve(4096); // always reserve full 4 KiB even if mirroring would make it
                                     // unneeded
                return Some(Self{
                    prg_rom: rom[prg_start..prg_end].iter().map(|a| *a).collect(),
                    chr_rom: rom[chr_start..chr_end].iter().map(|a| *a).collect(),
                    prg_ram: prgram,
                    nt_ram: ntram,
                    mapper: mapper
                });
            }
        }
        None 
    }
}

impl BusDevice for Cartrige {
    fn abus_read(&self, addr: u16) -> u8{
        self.mapper.abus_read(addr, CartMemRefs 
            { prg_rom: &self.prg_rom, chr_mem: &self.chr_rom, 
              prg_ram: &self.prg_ram, nt_ram: &self.nt_ram }
        )
    }
    fn abus_write(&mut self, addr: u16, val: u8) {
        self.mapper.abus_write(addr, val, CartMemRefsMut 
            { prg_rom: &self.prg_rom, chr_mem: &mut self.chr_rom,
              prg_ram: &mut self.prg_ram, nt_ram: &mut self.nt_ram }
        )
    }
    fn bbus_read(&self, addr: u16) -> u8 {
        self.mapper.bbus_read(addr, CartMemRefs 
            { prg_rom: &self.prg_rom, chr_mem: &self.chr_rom, 
              prg_ram: &self.prg_ram, nt_ram: &self.nt_ram }
        )
        
    }
    fn bbus_write(&mut self, addr: u16, val: u8) {
        self.mapper.bbus_write(addr, val, CartMemRefsMut 
            { prg_rom: &self.prg_rom, chr_mem: &mut self.chr_rom,
              prg_ram: &mut self.prg_ram, nt_ram: &mut self.nt_ram }
        )
    }
    fn bounds_abus(&self) -> (u16, u16) {
        (0x4020,0xFFFF)
    }
    fn bounds_bbus(&self) -> (u16, u16) {
        (0x0000,0x2FFF)
    }
}

pub fn is_valid_rom(rom: &[u8]) -> bool{
    // "NES\x1A", means its an iNES header
    if rom[0] == 0x4E && rom[1] == 0x45 && rom[2] == 0x53 && rom[3] == 0x1A {
        let mapper_num = ((rom[6] & 0xF0) >> 4) | (rom[7] & 0xF0);
        if mapper_num > 232 {
            return false; // invalid mapper number
        }
        return true;
    }
    return false; // not an NES file
}
