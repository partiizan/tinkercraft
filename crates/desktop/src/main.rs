#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod editor;
mod palette;
use eframe::egui;
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--version") {
        println!("TinkerCraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("TinkerCraft")
            .with_icon(egui::IconData {
                rgba: include_bytes!("../../../assets/tinkercraft-256.rgba").to_vec(),
                width: 256,
                height: 256,
            })
            .with_inner_size([1440., 960.])
            .with_min_inner_size([1000., 700.]),
        ..Default::default()
    };
    eframe::run_native(
        "TinkerCraft",
        options,
        Box::new(|cc| Ok(Box::new(editor::Editor::new(cc)))),
    )
}
