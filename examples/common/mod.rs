//! Shared example plumbing: env-selected scheme and a self-screenshot mode
//! (`BOOMER_SHOT=/path.png [BOOMER_SCHEME=silver|night] cargo run --example …`)
//! used to generate the README images and for headless visual checks.

// Each example compiles this module separately and not all use every item.
#![allow(dead_code)]

use eframe::egui;

/// Apply the scheme from `BOOMER_SCHEME` (silver | night), or follow the
/// system theme when unset.
pub fn theme_from_env(ctx: &egui::Context) {
    match std::env::var("BOOMER_SCHEME").as_deref() {
        Ok("night") => egui_theme_boomer::apply(ctx, egui_theme_boomer::Scheme::NextNight),
        Ok("silver") => egui_theme_boomer::apply(ctx, egui_theme_boomer::Scheme::Silver),
        _ => egui_theme_boomer::install(ctx),
    }
}

/// When `BOOMER_SHOT` is set, waits a few frames for the UI to settle, asks
/// the backend for a screenshot, writes it as PNG, and closes the app.
pub struct Screenshotter {
    path: Option<String>,
    frames: u32,
}

impl Default for Screenshotter {
    fn default() -> Self {
        Self {
            path: std::env::var("BOOMER_SHOT").ok(),
            frames: 0,
        }
    }
}

impl Screenshotter {
    /// True when running in screenshot mode.
    pub fn active(&self) -> bool {
        self.path.is_some()
    }

    /// Call once per frame from `App::ui`.
    pub fn tick(&mut self, ctx: &egui::Context) {
        let Some(path) = &self.path else { return };
        self.frames += 1;
        if self.frames == 15 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let [w, h] = img.size;
            image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec())
                .expect("screenshot buffer size mismatch")
                .save(path)
                .expect("failed to write screenshot PNG");
            eprintln!("wrote {path}");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}
