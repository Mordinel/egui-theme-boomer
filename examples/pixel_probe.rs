//! Renders one `widgets::button` at 1:1 pixel scale and screenshots it, for
//! pixel-exact comparison against authentic Windows 95 screenshots.
//!
//! `BOOMER_SHOT=probe.png cargo run --example pixel_probe`
//! Prints `BUTTON_RECT x y w h` (in screenshot pixel coordinates) on stderr.

#[path = "common/mod.rs"]
mod common;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([220.0, 120.0]),
        ..Default::default()
    };
    eframe::run_native(
        "pixel probe",
        options,
        Box::new(|cc| {
            common::theme_from_env(&cc.egui_ctx);
            Ok(Box::new(Probe {
                shot: common::Screenshotter::default(),
            }))
        }),
    )
}

struct Probe {
    shot: common::Screenshotter,
}

impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Force 1 point == 1 physical pixel so the screenshot is 1:1.
        let ppp = ctx.pixels_per_point();
        if (ppp - 1.0).abs() > 0.001 {
            ctx.set_zoom_factor(ctx.zoom_factor() / ppp);
        }
        self.shot.tick(&ctx);
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                // Same size as the "No" button in the Win95 Shut Down dialog,
                // as a stock egui button (beveled by the skin plugin).
                let stock = ui.add_sized([78.0, 23.0], egui::Button::new("No"));
                let r = stock.rect;
                eprintln!(
                    "STOCK_RECT {} {} {} {}",
                    r.min.x.round(),
                    r.min.y.round(),
                    r.width().round(),
                    r.height().round()
                );
            });
        });
        ctx.request_repaint();
    }
}
