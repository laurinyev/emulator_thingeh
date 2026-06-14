#![allow(non_snake_case)]
#![allow(unused)]

mod NES;
mod UI;

#[derive(Default)]
pub enum BackendState {
    #[default]
    NotEmulating,
    NES(NES::EmulState)
}

#[derive(Default)]
pub struct EmulatorState {
    ui_state: UI::UiState,
    backend_state: BackendState
}

pub fn main() {
    UI::ui_main(EmulatorState::default());
}
