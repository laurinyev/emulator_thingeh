use imgui::Ui;
use crate::EmulatorState;
use super::*;

pub struct EmulatorTool<'a> {
    pub name: &'a str,
    pub ui_draw: fn(&Ui, &mut EmulatorState)
}

pub const NUM_TOOLS: usize = 3;
pub const EMU_TOOLS: [EmulatorTool<'_>; NUM_TOOLS] = [
    EmulatorTool{
        name: "Debugger",
        ui_draw: debugger::draw
    },
    EmulatorTool{
        name: "ROM Info",
        ui_draw: rom_info::draw
    },
    EmulatorTool{
        name: "Memory viewer",
        ui_draw: mem_view::draw
    },
];
