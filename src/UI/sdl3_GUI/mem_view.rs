use imgui::{TreeNodeFlags, Ui};
use crate::EmulatorState;

pub fn draw(ui: &Ui, state: &mut EmulatorState) {
    match &state.backend_state {
        crate::BackendState::NotEmulating => {
            ui.text("Not emulating anything, load a rom to begin!")
        },
        crate::BackendState::NES(s) => {
            if ui.collapsing_header("CPU memory", TreeNodeFlags::empty()) {
                for i in 0..(0x10000/0x10) as usize{
                    let mut string = format!("{:04x} | ", i*0x10);
                    for j in 0..0x10 {
                        let val = s.bus.abus_read(((i*0x10)+j) as u16);
                        string.push_str(format!("{val:02x} ").as_str());
                    }
                    string.push('\n');
                    ui.text(string);
                }
            }
            if ui.collapsing_header("PPU memory", TreeNodeFlags::empty()) {
                for i in 0..(0x10000/0x10) as usize{
                    let mut string = format!("{:04x} | ", i*0x10);
                    for j in 0..0x10 {
                        let val = s.bus.bbus_read(((i*0x11)+j) as u16);
                        string.push_str(format!("{val:02x} ").as_str());
                    }
                    string.push('\n');
                    ui.text(string);
                }
            }
        }
    }
    
    

}
