use std::path::Path;

use imgui::Ui;
use super::tools;
use crate::*;

use rfd::FileDialog;

pub fn draw(ui: &Ui, state: &mut EmulatorState) {
    ui.menu_bar(||{
        ui.menu("File", || {
            if ui.menu_item("Load ROM") {
                let file = FileDialog::new()
                        .add_filter("iNES / NES 2.0", &["nes"])
                        .pick_file();

                if let Some(rom_path) = file {
                    println!("Loading ROM: {rom_path:?}");

                    if Path::new(&rom_path).exists() {
                        if let Ok(rom) = std::fs::read(rom_path) {
                            if NES::is_valid_rom(&rom){
                                if let Some(nes) = NES::EmulState::init(&rom) {
                                    state.backend_state = BackendState::NES(nes)
                                } else {
                                    println!("ERROR! Failed to initialize NES emulator")
                                }
                            } else {
                                println!("ERROR! Not a valid ROM for any supported system")
                            }
                        } else {
                            println!("ERROR! Failed to read ROM file")
                        }
                    } else {
                        println!("ERROR! ROM does not exist")
                    }

                }
            }
        });
        ui.menu("Emulation", || {});
        ui.menu("Tools", || {
            for (t,e) in (&tools::EMU_TOOLS)
                            .iter()
                            .zip(&mut state.ui_state.tool_status){
                
                ui.menu_item_config(t.name).build_with_ref(e);
            } 
        });
        ui.menu("Settings", || {});
    });
    match state.backend_state {
        BackendState::NotEmulating => {
            ui.text("Not emulating");
        },
        BackendState::NES(_) => {
            ui.text("Emulating an NES(doesn't do anything but you can check ROM info :)");
        }
    }
}

