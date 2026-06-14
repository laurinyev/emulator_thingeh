
pub struct CartMemRefs<'a>{
    prg_rom: &'a [u8], // program ROM
    chr_mem: &'a mut [u8], // character ROM/RAM
    prg_ram: &'a mut [u8], // program RAM
    nt_ram:  &'a mut [u8]  // CIRAM, emulated as if it was on the cartrige
}

pub enum NametableMirroring {
    FourScreen,
    Horizontal,
    Vertical,
    SingleScreenUpper,
    SingleScreenLower
}

pub enum MapperChip {
   NROM 
}

pub struct Mapper {
    pub chip: MapperChip,
    pub mirroring: NametableMirroring
}

impl Mapper {
    pub fn new(mapper_num: u8, nt_mirroring_vertical: bool, nt_mirroring_alt: bool) -> Option<Self> {
        let chip = match mapper_num {
            0 => Some(MapperChip::NROM),
            _ => None
        };

        if chip.is_none() {
            println!("ERROR! Bad or unsupported mapper(iNES mapper no. {mapper_num})");
            return None;
        }

        let mirroring = 
            if nt_mirroring_alt { NametableMirroring::FourScreen} else {
                if nt_mirroring_vertical { NametableMirroring:: Vertical} else { NametableMirroring::Horizontal }
            };

        Some(Mapper{
            chip: chip?,
            mirroring: mirroring
        })
    }

    pub fn abus_read(&self, addr: u16, romset: &CartMemRefs) -> u8 {
        0 // TODO: emulate mapper
    }
    pub fn abus_write(&mut self, addr: u16, val: u8, romset: &mut CartMemRefs) {
        //TODO: emulate mapper
    }

    pub fn bbus_read(&self, addr: u16, romset: &CartMemRefs) -> u8 {
        0 // TODO: emulate mapper but as if CIRAM was on the cartrige
    }

    pub fn bbus_write(&mut self, addr: u16, val: u8, romset: &mut CartMemRefs) {
        // TODO: emulate mapper but as if CIRAM was on the cartrige
    }

    pub fn stringify(&self) -> String {
        match self.chip {
            MapperChip::NROM => "NROM".to_string()
        }
    }
}

pub struct Cartrige {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub nt_ram:  Vec<u8>,
    pub mapper: Mapper,
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

            if let Some(mapper) = Mapper::new(mapper_num, nt_mirroring_vertical, nt_mirroring_alt) {
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
