use crate::NES::DualBus;

// STRUCTS

// CPU event: since this emulator is meant to be cycle accurate, we must emulate that the
// reset/IRQ/NMI behaviour hapens after the current cycle
enum CpuEvent {
    None,
    Reset,
    IRQ,
    NMI,
    BRK
}

// main CPU device struct
pub struct Cpu6502 {
    pc: u16, // program counter
    sp: u8,  // stack pointer
    a: u8,   // accumulator
    x: u8,   // X register
    y: u8,   // Y register
    flags: u8,

    current_instr: &'static CpuInstruction, // currently fetched instruction
    current_opcode: u8, // might aswell just share code for all the branch instructions lul
    t_state: u8, // the current cycle it's on
    cpu_event: CpuEvent,

    page_crossed: bool,
    amode_addr: u16,
    amode_val: u8
}

// a cycle accurate micro-op representation of an instruction
struct CpuInstruction {
    cycles: [fn(&mut Cpu6502, &mut DualBus); 7],
}

// HELPERS
fn push(cpu: &mut Cpu6502, bus: &mut DualBus, val: u8) {
    bus.abus_write(0x0100 | cpu.sp as u16, val);
    cpu.sp = cpu.sp.wrapping_sub(1);
}

//MICRO OPS

//NOP and yeild
fn mop_nop(cpu: &mut Cpu6502, bus: &mut DualBus) {}

fn mop_yeild(cpu: &mut Cpu6502, bus: &mut DualBus) {
    // always read opcode and increment the program counter
    let mut opcode = bus.abus_read(cpu.pc);
    cpu.pc = cpu.pc.wrapping_add(1);   

    match cpu.cpu_event {
        CpuEvent::None => {
            if opcode == 0 {
                cpu.cpu_event = CpuEvent::BRK; // supress the next pc 
            }
        },
        _ => opcode = 0 // BRK
    }

    if cpu.t_state == 1 {
        return; // extra yeild in a BRK instruction, don't store anything 
    }
  
    cpu.current_opcode = opcode;
    cpu.current_instr = &INSTR_TABLE[opcode as usize];
    cpu.t_state = 0; // T state 0 is this micro-op, it will get incremented afterwards
}

//addressing modes
fn mop_amode_zpg(cpu: &mut Cpu6502, bus: &mut DualBus) {
    cpu.amode_addr = bus.abus_read(cpu.pc) as u16;     
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_amode_add_x_zp(cpu: &mut Cpu6502, bus: &mut DualBus) {
    cpu.amode_addr = cpu.amode_addr.wrapping_add(cpu.x.into()) & 0xFF;
}

fn mop_amode_rel(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let mut rel = bus.abus_read(cpu.pc) as i8;
    cpu.pc = cpu.pc.wrapping_add(1);   
    cpu.amode_addr = (cpu.pc as i16).wrapping_add(rel as i16) as u16;
    cpu.page_crossed = cpu.pc & 0xFF00 != cpu.amode_addr & 0xFF00;
}

fn mop_amode_abs_loadh(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let mut h = bus.abus_read(cpu.pc) as u16;
    if cpu.amode_addr > 0xFF {
        cpu.page_crossed = true; // our higher byte won't be fucked up but we still need to apply
                                 // the penalty
    }
    cpu.amode_addr = cpu.amode_addr.wrapping_add(h << 8); // in case an a,x or a,y produced a low
                                                          // value higher than 0xFF
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_amode_abs_loadl(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let mut l = bus.abus_read(cpu.pc) as u16;
    cpu.amode_addr = l;
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_amode_abs_loadl_add_x(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let mut l = bus.abus_read(cpu.pc) as u16;
    cpu.amode_addr = l + cpu.x as u16; // wrapping_add not needed cuz it mathematically can NOT get
                                       // that large
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_amode_abs_loadl_add_y(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let mut l = bus.abus_read(cpu.pc) as u16;
    cpu.amode_addr = l + cpu.y as u16; // wrapping_add not needed cuz it mathematically can NOT get
                                       // that large
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_amode_load_val(cpu: &mut Cpu6502, bus: &mut DualBus) {
    cpu.amode_val = bus.abus_read(cpu.amode_addr);
}

// work-doing micro-ops
fn mop_load_a_imm(cpu: &mut Cpu6502, bus: &mut DualBus) {
    cpu.a = bus.abus_read(cpu.pc);     
    cpu.pc = cpu.pc.wrapping_add(1);   
}

fn mop_load_a_mem(cpu: &mut Cpu6502, bus: &mut DualBus) {
    cpu.a = bus.abus_read(cpu.amode_addr);
}

fn mop_push_pch(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let pch = (cpu.pc >> 8) as u8;
    push(cpu,bus,pch); 
}

fn mop_push_pcl(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let pcl = cpu.pc as u8;
    push(cpu,bus,pcl); 
}

fn mop_push_flags(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let flags = match cpu.cpu_event {
        CpuEvent::BRK => cpu.flags | 0x30,
        _ => cpu.flags | 0x20,
    };

    push(cpu,bus,flags);
}

fn mop_fetch_pch(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let addr = match cpu.cpu_event {
        CpuEvent::BRK | CpuEvent::IRQ => 0xFFFF,
        CpuEvent::Reset => 0xFFFD,
        CpuEvent::NMI => 0xFFFB,
        CpuEvent::None => unreachable!() // this micro-op is exclusively for times when a CPU event
                                         // is active
    };

    cpu.pc = ((bus.abus_read(addr) as u16) << 8) | cpu.pc & 0xFF;
    cpu.cpu_event = CpuEvent::None; //clear the CPU event as it has been handled
}

fn mop_fetch_pcl(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let addr = match cpu.cpu_event {
        CpuEvent::BRK | CpuEvent::IRQ => 0xFFFE,
        CpuEvent::Reset => 0xFFFC,
        CpuEvent::NMI => 0xFFFA,
        CpuEvent::None => unreachable!() // this micro-op is exclusively for times when a CPU event
                                         // is active
    };

    cpu.pc = (bus.abus_read(addr) as u16) | cpu.pc & 0xFF00;
    cpu.flags |= 0x04; // set I flag
}


fn mop_branch(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let should_branch = match cpu.current_opcode {
        0x10 => cpu.flags & 0x80 == 0, // BPL
        0x30 => cpu.flags & 0x80 != 0, // BMI
        0x90 => cpu.flags & 0x01 == 0, // BCC
        0xB0 => cpu.flags & 0x01 != 0, // BCS
        0xD0 => cpu.flags & 0x02 == 0, // BNE
        0xF0 => cpu.flags & 0x02 != 0, // BEQ
        _ => unreachable!() 
    };

    if should_branch {
        cpu.pc = cpu.amode_addr; 
    } else {
        mop_yeild(cpu, bus); // nullify this cycle 
    }
}

fn mop_page_cross_penalty(cpu: &mut Cpu6502, bus: &mut DualBus) {
    // if NO page boundary was crossed, nullify this cycle
    if !cpu.page_crossed {
        mop_yeild(cpu, bus);
    } else {
        cpu.page_crossed = false;
    }
}

// INSTRUCTION DEFINITIONS
const INSTR_POWER_ON: CpuInstruction = CpuInstruction {
    cycles: [mop_yeild, mop_push_pch, mop_push_pcl, mop_push_flags, mop_fetch_pcl, mop_fetch_pch, mop_yeild]
};

const INSTR_BRK_IMP: CpuInstruction = CpuInstruction {
    cycles: [mop_yeild, mop_push_pch, mop_push_pcl, mop_push_flags, mop_fetch_pcl, mop_fetch_pch, mop_yeild]
};

const INSTR_NOP_IMP: CpuInstruction = CpuInstruction {
    cycles: [mop_nop, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

// All branch instructions have the same flow
const INSTR_BXX_REL: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_rel, mop_branch, mop_page_cross_penalty, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

// loading instructions
const INSTR_LDA_IMM: CpuInstruction = CpuInstruction {
    cycles: [mop_load_a_imm, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

const INSTR_LDA_ZPG: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_zpg, mop_load_a_mem, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

const INSTR_LDA_ZPX: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_zpg, mop_amode_add_x_zp, mop_load_a_mem, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

const INSTR_LDA_ABS: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_abs_loadl, mop_amode_abs_loadh, mop_load_a_mem, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
};

const INSTR_LDA_ABX: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_abs_loadl_add_x, mop_amode_abs_loadh, mop_load_a_mem, mop_page_cross_penalty, mop_yeild, mop_yeild, mop_yeild]
};

const INSTR_LDA_ABY: CpuInstruction = CpuInstruction {
    cycles: [mop_amode_abs_loadl_add_y, mop_amode_abs_loadh, mop_load_a_mem, mop_page_cross_penalty, mop_yeild, mop_yeild, mop_yeild]
};

// TODO; not in the mood to figure these out today
//const INSTR_LDA_IDX: CpuInstruction = CpuInstruction {
//    cycles: [mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
//};

//const INSTR_LDA_IDY: CpuInstruction = CpuInstruction {
//    cycles: [mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild, mop_yeild]
//};


const INSTR_TABLE: [CpuInstruction; 256] = [
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 00 
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 08
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 10
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 18
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 20
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 28
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 30
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 38
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 40
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 48
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 50
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 58
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 60
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 68
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 70
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 78
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 80
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 88
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 90
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // 98
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_LDA_ZPG, INSTR_BRK_IMP, INSTR_BRK_IMP, // A0
    INSTR_BRK_IMP, INSTR_LDA_IMM, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_LDA_ABS, INSTR_BRK_IMP, INSTR_BRK_IMP, // A8
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_LDA_ZPX, INSTR_BRK_IMP, INSTR_BRK_IMP, // B0
    INSTR_BRK_IMP, INSTR_LDA_ABY, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_LDA_ABX, INSTR_BRK_IMP, INSTR_BRK_IMP, // B8
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // C0
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // C8
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // D0
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // D8
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // E0
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_NOP_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // E8
    INSTR_BXX_REL, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // F0
    INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, INSTR_BRK_IMP, // F8
];

impl Cpu6502 {
    pub fn new() -> Self {
        Self {
            pc: 0, 
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            flags: 0x4,

            current_instr: &INSTR_POWER_ON,
            current_opcode: 0xFF,       // uninitialized
            t_state: 1,                 // these states are 1-indexed
            cpu_event: CpuEvent::Reset, // so it reads from the reset vector
        
            page_crossed: false,
            amode_addr: 0x0000,
            amode_val: 0x00
        }
    }

    pub fn reset(&mut self) {
        self.cpu_event = CpuEvent::Reset;
    }

    pub fn tick(&mut self,bus: &mut DualBus) {
        self.current_instr.cycles[(self.t_state - 1) as usize](self, bus);
        self.t_state += 1;
    }
}
