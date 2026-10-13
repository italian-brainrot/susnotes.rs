use eframe::egui;

mod config;
mod crypto;
mod notes;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "susnotes",
        native_options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )?;
    Ok(())
}
