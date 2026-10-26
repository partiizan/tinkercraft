#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod editor;
mod gpu;
mod palette;
use eframe::egui;
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--version") {
        println!("TinkerCraft {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let args: Vec<_> = std::env::args_os().collect();
    let smoke_report = if args.get(1).is_some_and(|a| a == "--smoke-test") {
        assert_eq!(args.len(), 3, "Usage: tinkercraft --smoke-test REPORT.json");
        Some(std::path::PathBuf::from(&args[2]))
    } else {
        None
    };
    let options = eframe::NativeOptions {
        depth_buffer: 24,
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
    let report_path = smoke_report.clone();
    let result = eframe::run_native(
        "TinkerCraft",
        options,
        Box::new(move |cc| Ok(Box::new(editor::Editor::new(cc, smoke_report)))),
    );
    if let Err(error) = &result {
        if let Some(path) = report_path {
            let _ = std::fs::write(
                path,
                serde_json::json!({"startup_error": error.to_string()}).to_string(),
            );
        } else {
            rfd::MessageDialog::new()
                .set_title("TinkerCraft could not open a window")
                .set_description(format!(
                    "Check that your graphics driver supports modern OpenGL.\n\n{error}"
                ))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
    }
    result
}
