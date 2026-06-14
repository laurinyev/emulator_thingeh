use imgui::Ui;
use super::{
    debugger,
    rom_info
};
use crate::EmulatorState;

pub struct EmulatorTool<'a> {
    pub name: &'a str,
    pub ui_draw: fn(&Ui, &mut EmulatorState)
}

pub const NUM_TOOLS: usize = 2;
pub const EMU_TOOLS: [EmulatorTool<'_>; NUM_TOOLS] = [
    EmulatorTool{
        name: "Debugger",
        ui_draw: debugger::draw
    },
    EmulatorTool{
        name: "ROM Info",
        ui_draw: rom_info::draw
    },
];
