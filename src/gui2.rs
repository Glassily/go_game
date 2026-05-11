use eframe::egui;
use egui::{Color32, Stroke, Vec2};

use go_game::Board;
use go_game::model::{Color, Move, Point};
use go_game::record::{GoRecord, NodeInfo};
use go_game::sgf::{export, parse};

pub struct GoGui {
    record: GoRecord,
    edit_mode: bool,
    show_tree: bool,
    show_coords: bool,
    dark_theme: bool,
    comment_edit: String,
    show_comment_panel: bool,
    context_node: Option<usize>,
    context_pos: egui::Pos2,
    show_context_window: bool,
    info_black: String,
    info_white: String,
    info_date: String,
    info_komi: String,
    info_result: String,
    show_info_window: bool,
    show_error_window: bool,
    error_message: String,
}

impl GoGui {
    pub fn new() -> Self {
        Self {
            record: GoRecord::new(19),
            edit_mode: true,
            show_tree: true,
            show_coords: true,
            dark_theme: false,
            comment_edit: String::new(),
            show_comment_panel: true,
            context_node: None,
            context_pos: egui::pos2(0.0, 0.0),
            show_context_window: false,
            info_black: String::new(),
            info_white: String::new(),
            info_date: String::new(),
            info_komi: String::from("6.5"),
            info_result: String::new(),
            show_info_window: false,
            show_error_window: false,
            error_message: String::new(),
        }
    }
}

impl eframe::App for GoGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|input| {
            if input.key_pressed(egui::Key::ArrowLeft) {
                self.record.go_prev();
            }
            if input.key_pressed(egui::Key::ArrowRight) {
                self.record.go_next();
            }
            if input.key_pressed(egui::Key::Home) {
                self.record.go_first();
            }
            if input.key_pressed(egui::Key::End) {
                self.record.go_last();
            }
            if input.modifiers.ctrl && input.key_pressed(egui::Key::Z) {
                if input.modifiers.shift {
                    self.record.redo();
                } else {
                    self.record.undo();
                }
            }
        });

        self.top_panel(ctx);
        if self.show_tree {
            self.tree_panel(ctx);
        }
        self.central_panel(ctx);
        self.status_bar(ctx);
        self.info_window(ctx);
        self.error_window(ctx);
        self.context_menu(ctx);
    }
}

impl GoGui {
    /// 菜单栏
    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.file_menu(ui);
                self.edit_menu(ui);
                self.view_menu(ui);
                self.help_menu(ui);
            });

            ui.horizontal(|ui| {
                if ui.button("|<").clicked() {
                    self.record.go_first();
                }
                if ui.button("<").clicked() {
                    self.record.go_prev();
                }
                if ui.button(">").clicked() {
                    self.record.go_next();
                }
                if ui.button("|>").clicked() {
                    self.record.go_last();
                }
                ui.label(format!(
                    "Move: {}/{}",
                    self.record.current_move_number(),
                    self.record.total_moves()
                ));
            });
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                self.record.new_game();
                self.info_black.clear();
                self.info_white.clear();
                self.info_date.clear();
                self.info_komi.clear();
                self.info_result.clear();
                ui.close_menu();
            }
            if ui.button("Open SGF").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    match std::fs::read_to_string(&path) {
                        Ok(s) => match parse(&s) {
                            Ok(tree) => {
                                self.record.load_sgf(tree);
                                let info = self.record.get_game_info();
                                self.info_black = info.black.unwrap_or_default();
                                self.info_white = info.white.unwrap_or_default();
                                self.info_date = info.date.unwrap_or_default();
                                self.info_komi = info.komi.unwrap_or_default();
                                self.info_result = info.result.unwrap_or_default();
                            }
                            Err(e) => {
                                self.show_error_window = true;
                                self.error_message = format!("SGF parse error: {}", e);
                            }
                        },
                        Err(e) => {
                            self.show_error_window = true;
                            self.error_message = format!("Failed to read file: {}", e);
                        }
                    }
                }
                ui.close_menu();
            }
            if ui.button("Save As").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    let s = export(&self.record.tree);
                    let _ = std::fs::write(path, s);
                }
                ui.close_menu();
            }
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Edit", |ui| {
            if ui.button("Undo (Ctrl+Z)").clicked() {
                self.record.undo();
                ui.close_menu();
            }
            if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                self.record.redo();
                ui.close_menu();
            }
            if ui
                .button(format!(
                    "Toggle Edit Mode ({})",
                    if self.edit_mode { "ON" } else { "OFF" }
                ))
                .clicked()
            {
                self.edit_mode = !self.edit_mode;
                ui.close_menu();
            }
        });
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            if ui.checkbox(&mut self.dark_theme, "Dark theme").clicked() {
                if self.dark_theme {
                    ui.ctx().set_visuals(egui::Visuals::dark());
                } else {
                    ui.ctx().set_visuals(egui::Visuals::light());
                }
            }
            ui.checkbox(&mut self.show_tree, "Show game tree");
            ui.checkbox(&mut self.show_coords, "Show coordinates");
            ui.checkbox(&mut self.show_comment_panel, "Show comment panel");
        });
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Help", |ui| {
            ui.label("Go SGF Editor");
        });
    }

    /// 树面板
    fn tree_panel(&mut self, ctx: &egui::Context) {
        let win_w = ctx.available_rect().width();
        let panel_w = (win_w * 0.28).clamp(200.0, 420.0);

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(panel_w)
            .show(ctx, |ui| {
                ui.label("Game Tree");

                let node_views: Vec<(usize, NodeInfo)> = self.record.all_nodes();

                let max_depth = node_views.iter().map(|t| t.1.depth).max().unwrap_or(0) as f32;
                let indent = 18.0;
                let required_w = (max_depth + 2.0) * indent + 160.0;

                egui::ScrollArea::both().show_viewport(ui, |ui, _viewport| {
                    ui.set_min_width(required_w);

                    for (idx, info) in &node_views {
                        ui.horizontal(|ui| {
                            ui.add_space(info.depth as f32 * indent);
                            let dot_size = 12.0;
                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::new(dot_size + 8.0, dot_size + 8.0),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter();
                            let center = rect.center();
                            if self.record.current == Some(*idx) {
                                painter.rect_filled(
                                    rect.expand(4.0),
                                    4.0,
                                    Color32::from_rgb(200, 230, 255),
                                );
                            }
                            match info.kind {
                                1 => {
                                    painter.circle_filled(center, dot_size * 0.45, Color32::BLACK);
                                }
                                2 => {
                                    painter.circle_filled(center, dot_size * 0.45, Color32::WHITE);
                                    painter.circle_stroke(
                                        center,
                                        dot_size * 0.45,
                                        Stroke::new(1.0, Color32::BLACK),
                                    );
                                }
                                _ => {
                                    painter.circle_filled(
                                        center,
                                        dot_size * 0.25,
                                        Color32::from_gray(120),
                                    );
                                }
                            }

                            if resp.clicked() {
                                self.record.go_to(*idx);
                                self.comment_edit = info.comment.clone().unwrap_or_default();
                            }

                            ui.add_space(6.0);
                            if let Some(c) = &info.comment {
                                ui.label(c.clone());
                            }
                        });
                    }

                    if self.show_comment_panel {
                        ui.separator();
                        ui.label("Comment");
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                if let Some(i) = self.record.current {
                                    self.record.set_comment(i, self.comment_edit.clone());
                                }
                            }
                            if ui.button("Clear").clicked() {
                                self.comment_edit.clear();
                            }
                        });
                        ui.add(egui::TextEdit::multiline(&mut self.comment_edit).desired_rows(4));
                    }
                });
            });
    }

    fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_rect_before_wrap();
            let board_size = avail.width().min(avail.height());
            let center = avail.center();
            let min_pos = center - Vec2::splat(board_size * 0.5);
            let board_rect = egui::Rect::from_min_size(min_pos, Vec2::splat(board_size));

            let response = ui.allocate_rect(board_rect, egui::Sense::click());
            let board_rect = response.rect;

            draw_board(ui, board_rect, &self.record.board, self.show_coords);

            if response.clicked() && self.edit_mode {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(pt) = screen_pos_to_point(board_rect, pos, self.record.board_size())
                    {
                        let next_color = self.record.next_to_move();
                        let mv = Move::new(next_color, pt);
                        self.record.add_move(mv);
                    }
                }
            }

            if response.secondary_clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(pt) = screen_pos_to_point(board_rect, pos, self.record.board_size())
                    {
                        self.context_node = self.record.find_move_at_point(pt);
                        self.context_pos = pos;
                        self.show_context_window = true;
                    }
                }
            }
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Move: {}/{}",
                    self.record.current_move_number(),
                    self.record.total_moves()
                ));
                ui.separator();
                ui.label(format!(
                    "Next: {}",
                    if self.record.next_to_move() == Color::Black {
                        "Black"
                    } else {
                        "White"
                    }
                ));
                ui.separator();
                ui.label(format!(
                    "Captures - Black: {}  White: {}",
                    self.record.black_captures, self.record.white_captures
                ));
                ui.separator();
                ui.label(format!("Nodes: {}", self.record.node_count()));
                ui.separator();
                ui.label(format!(
                    "Edit: {}",
                    if self.edit_mode { "ON" } else { "OFF" }
                ));
                ui.separator();
                if ui.button("Comment").clicked() {
                    self.show_comment_panel = true;
                }
                ui.separator();
                if ui.button("Game Info").clicked() {
                    let info = self.record.get_game_info();
                    self.info_black = info.black.unwrap_or_default();
                    self.info_white = info.white.unwrap_or_default();
                    self.info_date = info.date.unwrap_or_default();
                    self.info_komi = info.komi.unwrap_or_default();
                    self.info_result = info.result.unwrap_or_default();
                    self.show_info_window = true;
                }
            });
        });
    }

    fn info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info_window {
            return;
        }

        egui::Window::new("Game Info").show(ctx, |ui| {
            ui.label("Black:");
            ui.text_edit_singleline(&mut self.info_black);
            ui.label("White:");
            ui.text_edit_singleline(&mut self.info_white);
            ui.label("Date:");
            ui.text_edit_singleline(&mut self.info_date);
            ui.label("Komi:");
            ui.text_edit_singleline(&mut self.info_komi);
            ui.label("Result:");
            ui.text_edit_singleline(&mut self.info_result);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let info = go_game::record::GameInfo {
                        black: if self.info_black.is_empty() {
                            None
                        } else {
                            Some(self.info_black.clone())
                        },
                        white: if self.info_white.is_empty() {
                            None
                        } else {
                            Some(self.info_white.clone())
                        },
                        date: if self.info_date.is_empty() {
                            None
                        } else {
                            Some(self.info_date.clone())
                        },
                        komi: if self.info_komi.is_empty() {
                            None
                        } else {
                            Some(self.info_komi.clone())
                        },
                        result: if self.info_result.is_empty() {
                            None
                        } else {
                            Some(self.info_result.clone())
                        },
                    };
                    self.record.set_game_info(&info);
                    self.show_info_window = false;
                }
                if ui.button("Cancel").clicked() {
                    self.show_info_window = false;
                }
            });
        });
    }

    fn error_window(&mut self, ctx: &egui::Context) {
        if !self.show_error_window {
            return;
        }

        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(&self.error_message);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        self.show_error_window = false;
                        self.error_message.clear();
                    }
                });
            });
    }

    fn context_menu(&mut self, ctx: &egui::Context) {
        if !self.show_context_window {
            return;
        }

        let node_idx = self.context_node;

        egui::Window::new("Move Options")
            .fixed_pos(self.context_pos)
            .show(ctx, |ui| {
                if let Some(idx) = node_idx {
                    ui.label(format!("Node: {}", idx));
                    if ui.button("Go to").clicked() {
                        self.record.go_to(idx);
                        self.context_node = None;
                        self.show_context_window = false;
                        ui.close_menu();
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.context_node = None;
                    self.show_context_window = false;
                    ui.close_menu();
                }
            });
    }
}

fn draw_board(ui: &mut egui::Ui, rect: egui::Rect, board: &Board, show_coords: bool) {
    let painter = ui.painter_at(rect);
    let size = board.size as usize;

    painter.rect_filled(rect, 0.0, Color32::from_rgb(230, 190, 120));

    let pad = (rect.width() * 0.03).clamp(2.0, 20.0);
    let inner_rect = rect.shrink(2.0);
    let drawing_rect = egui::Rect::from_min_max(
        inner_rect.min + egui::vec2(pad, pad),
        inner_rect.max - egui::vec2(pad, pad),
    );
    let cell = drawing_rect.width() / ((size as f32 - 1.0).max(1.0));

    for i in 0..size {
        let x = drawing_rect.left() + i as f32 * cell;
        let y = drawing_rect.top() + i as f32 * cell;
        painter.line_segment(
            [
                egui::pos2(x, drawing_rect.top()),
                egui::pos2(x, drawing_rect.bottom()),
            ],
            Stroke::new(1.2, Color32::BLACK),
        );
        painter.line_segment(
            [
                egui::pos2(drawing_rect.left(), y),
                egui::pos2(drawing_rect.right(), y),
            ],
            Stroke::new(1.2, Color32::BLACK),
        );
    }

    let star_points = if size == 19 {
        vec![3, 9, 15]
    } else if size == 13 {
        vec![3, 9]
    } else if size == 9 {
        vec![2, 6]
    } else {
        vec![]
    };
    for &sx in &star_points {
        for &sy in &star_points {
            if sx < size && sy < size {
                let cx = drawing_rect.left() + sx as f32 * cell;
                let cy = drawing_rect.top() + sy as f32 * cell;
                let star_radius = cell * 0.08;
                painter.circle_filled(egui::pos2(cx, cy), star_radius, Color32::BLACK);
            }
        }
    }

    for y in 0..size {
        for x in 0..size {
            let pt = Point {
                x: x as u8,
                y: y as u8,
            };
            if let Some(col) = board.get(pt) {
                let cx = drawing_rect.left() + x as f32 * cell;
                let cy = drawing_rect.top() + y as f32 * cell;
                let radius = cell * 0.42;
                match col {
                    Color::Black => {
                        painter.circle_filled(egui::pos2(cx, cy), radius, Color32::from_gray(10));
                    }
                    Color::White => {
                        painter.circle_filled(egui::pos2(cx, cy), radius, Color32::from_gray(240));
                        painter.circle_stroke(
                            egui::pos2(cx, cy),
                            radius,
                            Stroke::new(1.0, Color32::BLACK),
                        );
                    }
                }
            }
        }
    }

    if show_coords {
        let font_id = egui::FontId::proportional((cell * 0.35).max(1.0));
        //下方坐标
        for x in 0..size {
            let cx = drawing_rect.left() + x as f32 * cell;
            let label = if x >= 8 {
                ((b'A' + x as u8 + 1) as char).to_string()
            } else {
                ((b'A' + x as u8) as char).to_string()
            };
            painter.text(
                egui::pos2(cx - 6.0, drawing_rect.bottom() + 6.0),
                egui::Align2::LEFT_TOP,
                label,
                font_id.clone(),
                Color32::BLACK,
            );
        }
        // 右边坐标
        for y in 0..size {
            let cy = drawing_rect.top() + y as f32 * cell;
            let label = (size - y).to_string();
            painter.text(
                egui::pos2(drawing_rect.right() + 6.0, cy - 6.0),
                egui::Align2::LEFT_TOP,
                label,
                font_id.clone(),
                Color32::BLACK,
            );
        }
    }
}

fn screen_pos_to_point(rect: egui::Rect, pos: egui::Pos2, size: u8) -> Option<Point> {
    let pad = (rect.width() * 0.03).clamp(2.0, 20.0);
    let inner_rect = rect.shrink(2.0);
    let drawing_rect = egui::Rect::from_min_max(
        inner_rect.min + egui::vec2(pad, pad),
        inner_rect.max - egui::vec2(pad, pad),
    );
    if !drawing_rect.contains(pos) {
        return None;
    }
    let cell = drawing_rect.width() / ((size as f32 - 1.0).max(1.0));
    let fx = (pos.x - drawing_rect.left()) / cell;
    let fy = (pos.y - drawing_rect.top()) / cell;
    let xi = fx.round() as i32;
    let yi = fy.round() as i32;
    if xi >= 0 && yi >= 0 && (xi as u8) < size && (yi as u8) < size {
        Some(Point {
            x: xi as u8,
            y: yi as u8,
        })
    } else {
        None
    }
}
