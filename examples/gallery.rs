//! The zero-effort install: `egui_theme_boomer::install()` and every stock
//! egui widget you already have picks up the BevelDesk look.
//!
//! Run with: `cargo run --example gallery`

#[path = "common/mod.rs"]
mod common;

use eframe::egui;
use egui_theme_boomer::Scheme;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-theme-boomer gallery",
        options,
        Box::new(|cc| {
            // The zero-effort part: one call, everything is skinned.
            // (`install` inside; the env override is only for screenshots.)
            common::theme_from_env(&cc.egui_ctx);
            Ok(Box::new(Gallery::default()))
        }),
    )
}

struct Gallery {
    text: String,
    multiline: String,
    checkbox: bool,
    checkbox2: bool,
    radio: usize,
    radio2: usize,
    slider: f32,
    drag: i32,
    combo: usize,
    progress: f32,
    shot: common::Screenshotter,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            text: "The quick brown fox".to_owned(),
            multiline: "It's now safe to turn off\nyour computer.".to_owned(),
            checkbox: true,
            checkbox2: true,
            radio: 0,
            radio2: 0,
            slider: 42.0,
            drag: 1995,
            combo: 0,
            progress: 0.65,
            shot: common::Screenshotter::default(),
        }
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.shot.tick(&ctx);

        egui::Panel::top("scheme-switcher")
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Scheme:");
                    if ui.button("Silver").clicked() {
                        egui_theme_boomer::apply(&ctx, Scheme::Silver);
                    }
                    if ui.button("NeXT Night").clicked() {
                        egui_theme_boomer::apply(&ctx, Scheme::NextNight);
                    }
                    if ui.button("Follow system").clicked() {
                        egui_theme_boomer::install(&ctx);
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // All stock egui widgets — the install() skin makes them
                // pixel-exact Win95 controls, nothing else needed.
                ui.vertical(|ui| {
                    ui.set_width(340.0);
                    ui.heading("Buttons and controls");
                    ui.separator();
                    ui.hyperlink_to(
                        "Original theme source",
                        "https://github.com/marchildmann/BevelDesk",
                    );
                    ui.horizontal(|ui| {
                        let _ = ui.button("A button");
                        let _ = ui.add_enabled(false, egui::Button::new("Disabled"));
                    });
                    ui.checkbox(&mut self.checkbox, "Check me");
                    ui.checkbox(&mut self.checkbox2, "Save settings on exit");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.radio, 0, "One");
                        ui.radio_value(&mut self.radio, 1, "Two");
                        ui.radio_value(&mut self.radio, 2, "Three");
                    });
                    ui.radio_value(&mut self.radio2, 0, "Shut down the computer");
                    ui.radio_value(&mut self.radio2, 1, "Restart the computer");
                    ui.add(egui::Slider::new(&mut self.slider, 0.0..=100.0).text("Slider"));
                    ui.add(egui::DragValue::new(&mut self.drag).prefix("Year: "));
                    ui.add(egui::ProgressBar::new(self.progress));
                    egui::ComboBox::from_label("Drive")
                        .selected_text(["(A:)", "(C:)", "(D:)"][self.combo])
                        .show_ui(ui, |ui| {
                            for (i, label) in ["(A:)", "(C:)", "(D:)"].iter().enumerate() {
                                ui.selectable_value(&mut self.combo, i, *label);
                            }
                        });
                });

                ui.add_space(24.0);

                ui.vertical(|ui| {
                    ui.set_width(340.0);
                    ui.heading("Text and containers");
                    ui.separator();
                    ui.text_edit_singleline(&mut self.text);
                    ui.text_edit_multiline(&mut self.multiline);
                    ui.collapsing("Collapsing header", |ui| {
                        ui.code("C:\\> dir /w");
                    });
                });
            });

            ui.separator();
            ui.label("A striped grid in a scroll area:");
            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                egui::Grid::new("files")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        for i in 0..12 {
                            ui.label(format!("AUTOEXEC{i:02}.BAT"));
                            ui.label(format!("{} KB", 3 + i));
                            ui.label("MS-DOS Batch File");
                            ui.end_row();
                        }
                    });
            });
        });
    }
}
