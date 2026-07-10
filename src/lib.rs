//! # egui-theme-boomer
//!
//! Classic beveled desktop themes for egui: **Silver** (the exact Windows 95
//! "Windows Standard" colors) as your light theme and **NeXT Night**
//! (charcoal chrome, black keylines, steel accents) as your dark theme.
//! Ported from [BevelDesk](https://github.com/marchildmann/BevelDesk) (MIT).
//!
//! ## Zero-effort: skin what you have
//!
//! One call restyles every stock egui widget — colors, fonts, spacing, sharp
//! corners, crisp unfeathered edges — and follows the system light/dark
//! setting:
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! egui_theme_boomer::install(ctx); // Silver when light, NeXT Night when dark
//! # }
//! ```
//!
//! Or pin one scheme regardless of the system theme:
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! egui_theme_boomer::apply(ctx, egui_theme_boomer::Scheme::NextNight);
//! # }
//! ```
//!
//! In an eframe app, call it once in your app constructor with `cc.egui_ctx`.
//!
//! The skinning is pixel-exact for the controls themselves: a plugin
//! rewrites the emitted shapes each pass, so stock buttons, check boxes,
//! radios, combo boxes, sliders, and progress bars come out as true beveled
//! Win95 controls — including the +1,+1 content shift while a button is
//! pressed (see `skin.rs` for how).
//!
//! ## Opt-in: the full desktop
//!
//! The taskbar, desktop icons, and Win95 windowing are components you
//! explicitly import and use — see [`shell`]. The [`bevel`] painter
//! primitives and the pixel [`icons`] everything is built from are public
//! too.

pub mod bevel;
pub mod fonts;
pub mod icons;
pub mod palette;
pub mod shell;

mod skin;
mod style;

pub use fonts::install_fonts;
pub use palette::Palette;
pub use style::{style, visuals};

/// The two BevelDesk schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scheme {
    /// The exact Win95 "Windows Standard" silver-gray look.
    #[default]
    Silver,
    /// Charcoal chrome, black keylines, muted-steel captions, dark fields.
    NextNight,
}

impl Scheme {
    /// This scheme's color palette.
    pub fn palette(self) -> &'static Palette {
        match self {
            Self::Silver => &palette::SILVER,
            Self::NextNight => &palette::NEXT_NIGHT,
        }
    }
}

/// Install both schemes: Silver as the light theme, NeXT Night as the dark
/// theme, plus period fonts and crisp (unfeathered) rendering. The app then
/// follows the system / `egui` theme preference.
pub fn install(ctx: &egui::Context) {
    ctx.set_style_of(egui::Theme::Light, style(Scheme::Silver));
    ctx.set_style_of(egui::Theme::Dark, style(Scheme::NextNight));
    skin::install_plugin(ctx);
    install_fonts(ctx);
    crisp(ctx);
}

/// Force a single scheme regardless of the system light/dark preference,
/// plus period fonts and crisp rendering.
pub fn apply(ctx: &egui::Context, scheme: Scheme) {
    ctx.set_style_of(egui::Theme::Light, style(scheme));
    ctx.set_style_of(egui::Theme::Dark, style(scheme));
    skin::install_plugin(ctx);
    install_fonts(ctx);
    crisp(ctx);
}

/// Disable anti-aliasing, like BevelDesk's `AntiAliasedLines = false` — bevels
/// and pixel icons render hard-edged.
fn crisp(ctx: &egui::Context) {
    ctx.tessellation_options_mut(|t| {
        t.feathering = false;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn silver_matches_beveldesk_palette() {
        let p = Scheme::Silver.palette();
        assert_eq!(p.face, Color32::from_rgb(192, 192, 192));
        assert_eq!(p.desktop, Color32::from_rgb(0, 128, 128));
        assert_eq!(p.sel, Color32::from_rgb(0, 0, 128));
        assert_eq!(p.title_act2, Color32::from_rgb(16, 132, 208));
        assert_eq!(p.folder, Color32::from_rgb(255, 255, 128));
        assert!(!p.chiseled);
    }

    #[test]
    fn next_night_matches_beveldesk_palette() {
        let p = Scheme::NextNight.palette();
        assert_eq!(p.face, Color32::from_rgb(60, 60, 60));
        assert_eq!(p.winbg, Color32::from_rgb(27, 27, 27));
        assert_eq!(p.title_act, Color32::from_rgb(45, 53, 66));
        assert_eq!(p.sel, Color32::from_rgb(59, 90, 138));
        assert_eq!(p.accent, Color32::from_rgb(126, 146, 176));
        assert!(p.chiseled);
    }

    #[test]
    fn styles_are_square_and_instant() {
        for scheme in [Scheme::Silver, Scheme::NextNight] {
            let s = style(scheme);
            assert_eq!(s.visuals.window_corner_radius, egui::CornerRadius::ZERO);
            assert_eq!(s.visuals.menu_corner_radius, egui::CornerRadius::ZERO);
            for w in [
                &s.visuals.widgets.noninteractive,
                &s.visuals.widgets.inactive,
                &s.visuals.widgets.hovered,
                &s.visuals.widgets.active,
                &s.visuals.widgets.open,
            ] {
                assert_eq!(w.corner_radius, egui::CornerRadius::ZERO);
                assert_eq!(w.expansion, 0.0);
            }
            assert_eq!(s.animation_time, 0.0);
            assert_eq!(s.visuals.window_shadow.blur, 0);
            assert_eq!(s.spacing.scroll.bar_width, 16.0);
        }
    }

    #[test]
    fn scheme_maps_to_dark_mode() {
        assert!(!visuals(Scheme::Silver).dark_mode);
        assert!(visuals(Scheme::NextNight).dark_mode);
    }
}
