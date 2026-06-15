use imgui::Ui;
use std::any::Any;
use crate::{EmulatorState, NES::Cartrige};
use downcast_rs::Downcast;

pub fn draw(ui: &Ui, state: &mut EmulatorState) {
    match &state.backend_state {
        crate::BackendState::NotEmulating => {
            ui.text("Emulator not running, please load a ROM!");
        },
        crate::BackendState::NES(n) => {
            let any_box: &dyn Any = n.bus.devices[0].as_ref();
            if let Some(cart) = any_box.downcast_ref::<Cartrige>() {
                ui.text(format!("Cartrige mapper: {}",cart.mapper.mapper_name()));   
                ui.text(format!("Program ROM size: {} KiB",cart.prg_rom.len() / 1024));   
                ui.text(format!("Character ROM size: {} KiB",cart.chr_rom.len() / 1024));   
                if cart.prg_ram.len() != 0 {
                    ui.text(format!("Program RAM size: {} KiB",cart.prg_ram.len() / 1024));   
                }
            } else {
                ui.text("Couldn't get cartrige device from bus");
            }
            
        }
    }
}
