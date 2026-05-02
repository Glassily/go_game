use eframe::NativeOptions;

mod gui;

fn main() {
    let options = NativeOptions::default();
    let _ = eframe::run_native(
        "Go Game - SGF Editor",
        options,
        Box::new(|_cc| Ok(Box::new(gui::GoApp::new()))),
    );
}
