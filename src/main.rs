use eframe::NativeOptions;
use eframe::egui::{Context, FontData, FontDefinitions};
use std::path::Path;

mod gui;

fn main() {
    let options = NativeOptions::default();
    let _ = eframe::run_native(
        "Go Game - SGF Editor",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(gui::GoGui::new()))
        }),
    );
}

fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    if let Ok(font_path) = std::env::var("SYSTEM_FONT_PATH") {
        if let Ok(font_data) = std::fs::read(&font_path) {
            fonts.font_data.insert(
                "system_font".to_owned(),
                FontData::from_static(Box::leak(font_data.into_boxed_slice())).into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "system_font".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "system_font".to_owned());
        }
    } else {
        let possible_paths = if cfg!(windows) {
            vec![
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\simsun.ttc",
                "C:\\Windows\\Fonts\\simhei.ttf",
                "C:\\Windows\\Fonts\\YaHei.ttf",
                "C:\\Windows\\Fonts\\msyhbd.ttc",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
                "/Library/Fonts/Arial Unicode.ttf",
            ]
        } else {
            vec![
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            ]
        };

        for path in possible_paths {
            if Path::new(path).exists() {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "cjk_font".to_owned(),
                        FontData::from_static(Box::leak(font_data.into_boxed_slice())).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "cjk_font".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .insert(0, "cjk_font".to_owned());
                    break;
                }
            }
        }
    }

    ctx.set_fonts(fonts);
}
