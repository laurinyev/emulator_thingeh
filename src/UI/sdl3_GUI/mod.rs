use sdl3::{
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    gpu::*,
};
use imgui_sdl3::ImGuiSdl3;
use std::time::Duration;
use crate::EmulatorState;

mod tools;
mod debugger;
mod rom_info;
mod main_win;

#[derive(Default)]
pub struct UiState {
    tool_status: [bool; tools::NUM_TOOLS]
}

pub fn ui_main(mut state: EmulatorState)  -> Result<(), Box<dyn std::error::Error>>{
    let mut sdl = sdl3::init()?;
    let video_subsystem = sdl.video()?;

    let window = video_subsystem.window("My custom emulator!", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    
    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;
    
    let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
        ctx.set_ini_filename(None);
        ctx.set_log_filename(None);
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    });

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            // pass all events to imgui platform
            imgui.handle_event(&event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        let mut command_buffer = device.acquire_command_buffer()?;

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&window) {
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::DONT_CARE)
                .with_store_op(StoreOp::STORE)];

            imgui.render(
                &mut sdl,
                &device,
                &window,
                &event_pump,
                &mut command_buffer,
                &color_targets,
                |ui| {
                    let size = ui.io().display_size;
                    ui.window("Background")
                        .position([0.0, 0.0], imgui::Condition::Always)
                        .size(size, imgui::Condition::Always)
                        .flags(
                            imgui::WindowFlags::NO_TITLE_BAR
                                | imgui::WindowFlags::NO_RESIZE
                                | imgui::WindowFlags::NO_MOVE
                                | imgui::WindowFlags::NO_SCROLLBAR
                                | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                                | imgui::WindowFlags::NO_NAV_FOCUS
                        )
                        .menu_bar(true)
                    .build(|| {
                        main_win::draw(ui, &mut state); 
                    });
                
                    for (t, mut e) in (&tools::EMU_TOOLS)
                                .iter()
                                .zip(state.ui_state.tool_status){
                        if e {
                            ui.window(t.name)
                            .opened(&mut e)
                            .build(|| {
                                (t.ui_draw)(ui, &mut state);
                            });
                        }
                    }
                },
            );

            command_buffer.submit()?;
        } else {
            println!("Swapchain unavailable, cancel work");
            command_buffer.cancel();
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    };
    Ok(())
}
