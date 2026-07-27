//! Floating Apple-glass style launcher window.

use crate::config::Theme;
use crate::engine::Engine;
use crate::providers::actions::action_label;
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontFamily, FontId, Frame, Key, Margin, Pos2, Rect,
    RichText, Sense, Shadow, Stroke, Vec2,
};

pub fn run_launcher() -> eframe::Result<()> {
    let engine = match Engine::new() {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("alfredrs: failed to init engine: {err:#}");
            return Ok(());
        }
    };
    let theme = engine.theme().clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme.window_width, theme.window_height])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_title("alfredrs"),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "alfredrs",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx, &theme);
            Ok(Box::new(LauncherApp {
                engine,
                input: String::new(),
            }))
        }),
    )
}

fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    // Prefer a clean Apple-adjacent proportional face when available.
    let mut fonts = egui::FontDefinitions::default();
    for candidate in [
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/SFNSText.ttf",
        "/Library/Fonts/SF-Pro-Display-Regular.otf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/opentype/noto/NotoSans-Regular.ttf",
    ] {
        if let Ok(bytes) = std::fs::read(candidate) {
            fonts.font_data.insert(
                "glass".into(),
                egui::FontData::from_owned(bytes).into(),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "glass".into());
            break;
        }
    }
    ctx.set_fonts(fonts);

    let mut visuals = egui::Visuals::light();
    visuals.window_fill = Color32::TRANSPARENT;
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.override_text_color = Some(Color32::from_rgb(
        theme.foreground[0],
        theme.foreground[1],
        theme.foreground[2],
    ));
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
        theme.selection[0],
        theme.selection[1],
        theme.selection[2],
        90,
    );
    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.hovered.bg_fill = Color32::from_white_alpha(18);
    visuals.widgets.active.bg_fill = Color32::from_white_alpha(28);
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(theme.font_size, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(theme.font_size + 8.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new((theme.font_size - 3.0).max(11.0), FontFamily::Proportional),
    );
    ctx.set_style(style);
}

struct GlassPalette {
    panel: Color32,
    panel_border: Color32,
    highlight: Color32,
    field: Color32,
    field_border: Color32,
    selection: Color32,
    title: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    radius: f32,
}

impl GlassPalette {
    fn from_theme(theme: &Theme) -> Self {
        let light = theme.background[0] > 160;
        if light {
            Self {
                panel: Color32::from_rgba_unmultiplied(252, 252, 254, 118),
                panel_border: Color32::from_rgba_unmultiplied(255, 255, 255, 230),
                highlight: Color32::from_rgba_unmultiplied(255, 255, 255, 70),
                field: Color32::from_rgba_unmultiplied(255, 255, 255, 95),
                field_border: Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                selection: Color32::from_rgba_unmultiplied(0, 122, 255, 72),
                title: Color32::from_rgb(28, 28, 30),
                text: Color32::from_rgb(theme.foreground[0], theme.foreground[1], theme.foreground[2]),
                muted: Color32::from_rgba_unmultiplied(60, 60, 67, 180),
                accent: Color32::from_rgb(theme.accent[0], theme.accent[1], theme.accent[2]),
                radius: theme.corner_radius,
            }
        } else {
            // Dark liquid glass
            Self {
                panel: Color32::from_rgba_unmultiplied(22, 22, 26, 140),
                panel_border: Color32::from_rgba_unmultiplied(255, 255, 255, 55),
                highlight: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                field: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                field_border: Color32::from_rgba_unmultiplied(255, 255, 255, 48),
                selection: Color32::from_rgba_unmultiplied(10, 132, 255, 90),
                title: Color32::from_rgb(245, 245, 247),
                text: Color32::from_rgb(theme.foreground[0], theme.foreground[1], theme.foreground[2]),
                muted: Color32::from_rgba_unmultiplied(235, 235, 245, 150),
                accent: Color32::from_rgb(theme.accent[0], theme.accent[1], theme.accent[2]),
                radius: theme.corner_radius,
            }
        }
    }
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
        if let Some(text) = self.engine.large_type_text().map(str::to_string) {
            self.draw_large_type(ctx, &text);
            return;
        }

        let theme = self.engine.theme().clone();
        let glass = GlassPalette::from_theme(&theme);

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let pad = 18.0;
                let panel = full.shrink(pad);
                paint_glass_panel(ui, panel, &glass);

                let mut content = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(panel.shrink(22.0))
                        .layout(egui::Layout::top_down(Align::Center)),
                );

                content.add_space(4.0);
                content.label(
                    RichText::new("alfredrs")
                        .color(glass.title)
                        .size(theme.font_size + 2.0)
                        .strong(),
                );
                content.add_space(10.0);

                // Frosted search field
                let field_h = theme.font_size + 28.0;
                let field_rect = content.available_rect_before_wrap();
                let field_rect = Rect::from_min_size(
                    field_rect.min,
                    Vec2::new(field_rect.width(), field_h),
                );
                paint_glass_field(ui, field_rect, &glass);

                content.allocate_ui_at_rect(field_rect, |ui| {
                    ui.add_space(6.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input)
                            .frame(false)
                            .desired_width(f32::INFINITY)
                            .hint_text(
                                RichText::new("Search apps, files, web, workflows…")
                                    .color(glass.muted)
                                    .size(theme.font_size + 4.0),
                            )
                            .text_color(glass.text)
                            .font(FontId::new(theme.font_size + 4.0, FontFamily::Proportional)),
                    );
                    response.request_focus();
                    if response.changed() {
                        self.engine.set_query(self.input.clone());
                        if self.engine.query() != self.input {
                            self.input = self.engine.query().to_string();
                        }
                    }
                });

                content.add_space(14.0);

                let selected_idx = self.engine.selected();
                let results: Vec<_> = self.engine.results().to_vec();
                egui::ScrollArea::vertical()
                    .max_height(panel.height() - 150.0)
                    .show(&mut content, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        for (idx, item) in results.iter().enumerate() {
                            let selected = idx == selected_idx;
                            let row = Frame::NONE
                                .fill(if selected {
                                    glass.selection
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .corner_radius(CornerRadius::same(14))
                                .inner_margin(Margin::symmetric(14, 10))
                                .stroke(if selected {
                                    Stroke::new(1.0_f32, Color32::from_white_alpha(40))
                                } else {
                                    Stroke::NONE
                                })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(&item.title)
                                                .color(glass.text)
                                                .size(theme.font_size)
                                                .strong(),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(kind_label(item.kind))
                                                        .color(glass.accent)
                                                        .size(theme.font_size - 4.0)
                                                        .strong(),
                                                );
                                            },
                                        );
                                    });
                                    if !item.subtitle.is_empty() {
                                        ui.label(
                                            RichText::new(&item.subtitle)
                                                .color(glass.muted)
                                                .size(theme.font_size - 4.0),
                                        );
                                    }
                                    if selected {
                                        if let Some(action) = item.primary_action() {
                                            ui.add_space(2.0);
                                            ui.label(
                                                RichText::new(format!("⏎  {}", action_label(action)))
                                                    .color(glass.accent)
                                                    .size(theme.font_size - 5.0),
                                            );
                                        }
                                    }
                                });
                            let _ = row.response.interact(Sense::click());
                        }
                    });

                // Footer hints
                let footer = format!(
                    "↑↓  navigate    ⏎  open    ⌥⏎  actions    Esc  dismiss"
                );
                content.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(footer)
                            .color(glass.muted)
                            .size((theme.font_size - 5.0).max(11.0)),
                    );
                });
            });

        self.handle_keys(ctx);
    }
}

fn kind_label(kind: crate::model::ItemKind) -> &'static str {
    use crate::model::ItemKind::*;
    match kind {
        App => "App",
        File => "File",
        Web => "Web",
        Calculator => "Calculator",
        Dictionary => "Dictionary",
        System => "System",
        Shell => "Shell",
        Clipboard => "Clipboard",
        Snippet => "Snippet",
        Workflow => "Workflow",
        Bookmark => "Bookmark",
        Contact => "Contact",
        Music => "Music",
        Recent => "Recent",
        LargeType => "Large Type",
        Action => "Action",
        Buffer => "Buffer",
        Stats => "Stats",
        Fallback => "Fallback",
        Preview => "Preview",
    }
}

fn paint_glass_panel(ui: &egui::Ui, rect: Rect, glass: &GlassPalette) {
    let painter = ui.painter();
    let radius = CornerRadius::same(glass.radius as u8);

    // Soft outer shadow stack (Apple-like elevation)
    for (offset, expand, alpha) in [(14.0_f32, 8.0_f32, 12u8), (8.0, 4.0, 22), (3.0, 1.5, 36)] {
        let shadow = rect
            .translate(Vec2::new(0.0, offset * 0.45))
            .expand(expand);
        painter.rect_filled(
            shadow,
            CornerRadius::same((glass.radius + expand) as u8),
            Color32::from_black_alpha(alpha),
        );
    }

    // Frosted body
    painter.rect(
        rect,
        radius,
        glass.panel,
        Stroke::new(1.0_f32, glass.panel_border),
        egui::StrokeKind::Inside,
    );

    // Inner rim for glass edge
    painter.rect_stroke(
        rect.shrink(1.0),
        CornerRadius::same((glass.radius as i32 - 1).max(1) as u8),
        Stroke::new(0.6_f32, Color32::from_white_alpha(90)),
        egui::StrokeKind::Inside,
    );

    // Top specular highlight strip
    let highlight = Rect::from_min_max(
        rect.min + Vec2::new(2.0, 2.0),
        Pos2::new(rect.max.x - 2.0, rect.min.y + rect.height() * 0.28),
    );
    painter.rect_filled(
        highlight,
        CornerRadius {
            nw: (glass.radius - 2.0).max(1.0) as u8,
            ne: (glass.radius - 2.0).max(1.0) as u8,
            sw: 0,
            se: 0,
        },
        glass.highlight,
    );
}

fn paint_glass_field(ui: &egui::Ui, rect: Rect, glass: &GlassPalette) {
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(14),
        glass.field,
        Stroke::new(1.0_f32, glass.field_border),
        egui::StrokeKind::Inside,
    );
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
            if self.engine.large_type_text().is_some() {
                self.engine.clear_large_type();
            } else if self.engine.in_actions_mode() {
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
            if self.engine.large_type_text().is_none() && !self.engine.in_actions_mode() {
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
        let theme = self.engine.theme().clone();
        let glass = GlassPalette::from_theme(&theme);
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let rect = ui.max_rect().shrink(24.0);
                paint_glass_panel(ui, rect, &glass);
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new(text)
                                .size(64.0)
                                .color(glass.title)
                                .strong(),
                        );
                    });
                    ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                        ui.add_space(18.0);
                        ui.label(
                            RichText::new("Press Esc to dismiss")
                                .color(glass.muted)
                                .size(14.0),
                        );
                    });
                });
            });
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.engine.clear_large_type();
        }
    }
}
