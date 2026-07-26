//! Floating Alfred-style search window.

use crate::config::Theme;
use crate::engine::Engine;
use crate::providers::actions::action_label;
use eframe::egui::{self, Color32, Key, RichText};

pub fn run_launcher() -> eframe::Result<()> {
    let engine = Engine::new().expect("engine init");
    let theme = engine.config.theme.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme.window_width, theme.window_height])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_title("alfredrs"),
        ..Default::default()
    };
    eframe::run_native(
        "alfredrs",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx, &theme);
            Ok(Box::new(LauncherApp { engine, input: String::new() }))
        }),
    )
}

fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(theme.background[0], theme.background[1], theme.background[2]);
    visuals.override_text_color = Some(Color32::from_rgb(
        theme.foreground[0],
        theme.foreground[1],
        theme.foreground[2],
    ));
    visuals.selection.bg_fill = Color32::from_rgb(theme.selection[0], theme.selection[1], theme.selection[2]);
    visuals.window_fill = visuals.panel_fill;
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(theme.font_size),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(theme.font_size + 6.0),
    );
    ctx.set_style(style);
}

struct LauncherApp {
    engine: Engine,
    input: String,
}

impl eframe::App for LauncherApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(text) = self.engine.large_type.clone() {
            self.draw_large_type(ctx, &text);
            return;
        }

        let accent = {
            let t = &self.engine.config.theme;
            Color32::from_rgb(t.accent[0], t.accent[1], t.accent[2])
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("alfredrs")
                        .color(accent)
                        .size(self.engine.config.theme.font_size + 4.0)
                        .strong(),
                );
                ui.add_space(8.0);
            });

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .desired_width(f32::INFINITY)
                    .hint_text("Search apps, files, web, workflows…")
                    .font(egui::TextStyle::Heading),
            );
            response.request_focus();

            if response.changed() {
                self.engine.set_query(self.input.clone());
            }

            ui.add_space(8.0);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, item) in self.engine.results.iter().enumerate() {
                    let selected = idx == self.engine.selected;
                    let bg = if selected {
                        let t = &self.engine.config.theme;
                        Color32::from_rgb(t.selection[0], t.selection[1], t.selection[2])
                    } else {
                        Color32::TRANSPARENT
                    };
                    egui::Frame::NONE
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&item.title).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{:?}", item.kind))
                                            .small()
                                            .color(accent),
                                    );
                                });
                            });
                            if !item.subtitle.is_empty() {
                                ui.label(RichText::new(&item.subtitle).weak().small());
                            }
                            if selected {
                                if let Some(action) = item.primary_action() {
                                    ui.label(
                                        RichText::new(format!("⏎ {}", action_label(action)))
                                            .small()
                                            .color(accent),
                                    );
                                }
                            }
                        });
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(
                    RichText::new("↑↓ navigate · ⏎ open · ⌥⏎ actions · Esc dismiss · keywords: find clip snip wf music stats")
                        .small()
                        .weak(),
                );
            });
        });

        self.handle_keys(ctx);
    }
}

impl LauncherApp {
    fn handle_keys(&mut self, ctx: &egui::Context) {
        let mut quit = false;
        let mut activate = false;
        let mut actions = false;
        let mut up = false;
        let mut down = false;
        let mut escape = false;

        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                escape = true;
            }
            if i.key_pressed(Key::ArrowUp) {
                up = true;
            }
            if i.key_pressed(Key::ArrowDown) {
                down = true;
            }
            if i.key_pressed(Key::Enter) {
                if i.modifiers.alt {
                    actions = true;
                } else {
                    activate = true;
                }
            }
            if i.key_pressed(Key::Q) && i.modifiers.ctrl {
                quit = true;
            }
        });

        if escape {
            if self.engine.large_type.is_some() {
                self.engine.large_type = None;
            } else if self.engine.actions_mode {
                self.engine.exit_actions_mode();
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if up {
            self.engine.move_selection(-1);
        }
        if down {
            self.engine.move_selection(1);
        }
        if actions {
            self.engine.enter_actions_mode();
        }
        if activate {
            let _ = self.engine.activate();
            if self.engine.large_type.is_none() && !self.engine.actions_mode {
                // Keep window open for copy/calc; close for open/run.
                // Heuristic: if query was calculator/snippet copy, stay; else close.
                if !self.input.starts_with('=')
                    && !self.input.starts_with("clip")
                    && !self.input.starts_with("snip")
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn draw_large_type(&mut self, ctx: &egui::Context, text: &str) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(text).size(72.0).strong());
            });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label("Press Esc to dismiss");
            });
        });
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.engine.large_type = None;
        }
    }
}
