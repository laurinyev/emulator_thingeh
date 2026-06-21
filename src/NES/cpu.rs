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
    sp: u16, // stack pointer
    a: u8,  // accumulator
    x: u8,  // X register
    y: u8,  // Y register
    flags: u8,

    current_instr: &'static CpuInstruction, // currently fetched instruction
    t_state: u8, // the current cycle it's on
    cpu_event: CpuEvent,
}

// a cycle accurate micro-op representation of an instruction
struct CpuInstruction {
    cycles: [fn(&mut Cpu6502, &mut DualBus); 7],
}

// HELPERS
fn push(cpu: &mut Cpu6502, bus: &mut DualBus, val: u8) {
    bus.abus_write(cpu.sp, val);
    cpu.sp -= 1;
}

//MICRO OPS
fn mop_nop(cpu: &mut Cpu6502, bus: &mut DualBus) {}

fn mop_yeild(cpu: &mut Cpu6502, bus: &mut DualBus) {
    // always read opcode and increment the program counter
    let mut opcode = bus.abus_read(cpu.pc);
    cpu.pc += 1;   

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
   
    cpu.current_instr = &INSTR_TABLE[opcode as usize];
    cpu.t_state = 0; // T state 0 is this micro-op, it will get incremented afterwards
}

fn mop_push_pch(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let pch = (cpu.pc >> 8) as u8;
    push(cpu,bus,pch); 
}

fn mop_push_pcl(cpu: &mut Cpu6502, bus: &mut DualBus) {
    let pch = (cpu.pc >> 8) as u8;
    push(cpu,bus,pch); 
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
}

// INSTRUCTION DEFINITIONS
const INSTR_POWER_ON: CpuInstruction = CpuInstruction {
    cycles: [mop_yeild, mop_push_pch, mop_push_pcl, mop_push_flags, mop_fetch_pcl, mop_fetch_pch, mop_yeild]
};

const INSTR_BRK: CpuInstruction = CpuInstruction {
    cycles: [mop_yeild, mop_push_pch, mop_push_pcl, mop_push_flags, mop_fetch_pcl, mop_fetch_pch, mop_yeild]
};

const INSTR_TABLE: [CpuInstruction; 256] = [
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
    INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, INSTR_BRK, 
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
            t_state: 1, // these states are 1-indeyed
            cpu_event: CpuEvent::Reset // so it reads from the reset vector
        }
    }

    pub fn reset(&mut self) {

    }

    pub fn tick(&mut self,bus: &mut DualBus) {
        self.current_instr.cycles[(self.t_state - 1) as usize](self, bus);
        self.t_state += 1;
    }
}
