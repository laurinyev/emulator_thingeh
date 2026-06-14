use imgui::Ui;
use crate::EmulatorState;

pub fn draw(ui: &Ui, state: &mut EmulatorState) {
    match &state.backend_state {
        crate::BackendState::NotEmulating => {
            ui.text("Emulator not running, please load a ROM!");
        },
        crate::BackendState::NES(n) => {
            ui.text(format!("Cartrige mapper: {}",n.cart.mapper.stringify()));   
            ui.text(format!("Program ROM size: {} KiB",n.cart.prg_rom.len() / 1024));   
            ui.text(format!("Character ROM size: {} KiB",n.cart.chr_rom.len() / 1024));   
            if n.cart.prg_ram.len() != 0 {
                ui.text(format!("Program RAM size: {} KiB",n.cart.prg_ram.len() / 1024));   
            }
        }
    }
}
