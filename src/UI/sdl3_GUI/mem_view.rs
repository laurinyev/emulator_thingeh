use imgui::{TreeNodeFlags, Ui};
use crate::EmulatorState;

pub fn draw(ui: &Ui, state: &mut EmulatorState) {
    match &state.backend_state {
        crate::BackendState::NotEmulating => {
            ui.text("Not emulating anything, load a rom to begin!")
        },
        crate::BackendState::NES(s) => {
            if ui.collapsing_header("CPU memory", TreeNodeFlags::empty()) {
                ui.text("this is the CPU memory");
                ui.child_window("rickroll");
                
                for i in 0..(0x10000/0x10) as usize{
                    let mut string = format!("{:x} | ", i*0x10);
                    for j in 0..0x10 {
                        let val = s.bus.abus_read(((i*0x10)+j) as u16);
                        string.push_str(format!("{val:x} ").as_str());
                    }
                    string.push('\n');
                    ui.text(string);
                }
            }
        }
    }
    
    

}
