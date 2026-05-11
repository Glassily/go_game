use eframe::NativeOptions;

mod gui2;

fn main() {
    let options = NativeOptions::default();
    let _ = eframe::run_native(
        "Go Game - SGF Editor",
        options,
        Box::new(|_cc| Ok(Box::new(gui2::GoGui::new()))),
    );
}
