//! Best-effort period fonts, mirroring BevelDesk's `platform.cpp` strategy:
//! try well-known system font paths per OS, silently fall back to egui's
//! bundled fonts when none exist. Nothing is embedded in the crate.
//!
//! - Windows: Microsoft Sans Serif (`micross.ttf` — the real deal), Tahoma, Arial
//! - macOS: Tahoma, Verdana, Arial (all in `/System/Library/Fonts/Supplemental`)
//! - Linux: DejaVu Sans, Liberation Sans
//!
//! A matching bold face is registered under the [`BOLD_FAMILY`] font family
//! (used by the shell components for captions and the Start button). If no
//! bold file is found the family falls back to the regular fonts and the
//! shell fakes bold with a 1px double-strike, exactly like BevelDesk does
//! when `FontBold` is missing.

use egui::{Context, FontFamily, Id};

/// Name of the bold [`egui::FontFamily`] registered by [`install_fonts`].
pub const BOLD_FAMILY: &str = "egui-theme-boomer-bold";

/// The bold font family. Always resolvable after [`install_fonts`] ran
/// (falls back to regular fonts when no bold system font was found).
pub fn bold_family() -> FontFamily {
    FontFamily::Name(BOLD_FAMILY.into())
}

fn bold_flag_id() -> Id {
    Id::new("egui-theme-boomer-bold-is-real")
}

/// True when a real bold system font was found and registered.
pub(crate) fn has_real_bold(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp(bold_flag_id()).unwrap_or(false))
}

#[cfg(not(target_arch = "wasm32"))]
const REGULAR: &[&str] = &[
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Tahoma.ttf",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Verdana.ttf",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\micross.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\tahoma.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\arial.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

#[cfg(not(target_arch = "wasm32"))]
const BOLD: &[&str] = &[
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Tahoma Bold.ttf",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Verdana Bold.ttf",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\tahomabd.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\arialbd.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];

#[cfg(not(target_arch = "wasm32"))]
const MONO: &[&str] = &[
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Menlo.ttc",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Monaco.ttf",
    #[cfg(target_os = "macos")]
    "/System/Library/Fonts/Supplemental/Courier New.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\consola.ttf",
    #[cfg(target_os = "windows")]
    "C:\\Windows\\Fonts\\cour.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
    #[cfg(all(unix, not(target_os = "macos")))]
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
];

#[cfg(not(target_arch = "wasm32"))]
fn first_existing(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

/// Install the period fonts into the context and register the bold family.
///
/// Called automatically by [`crate::install`] and [`crate::apply`]; only call
/// this directly if you are managing styles yourself. On the web (wasm) this
/// keeps egui's bundled fonts and only registers the bold-family fallback.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut bold_real = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::Arc;

        if let Some(bytes) = first_existing(REGULAR) {
            fonts.font_data.insert(
                "boomer-sans".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "boomer-sans".to_owned());
        }
        if let Some(bytes) = first_existing(BOLD) {
            fonts.font_data.insert(
                "boomer-sans-bold".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            bold_real = true;
        }
        if let Some(bytes) = first_existing(MONO) {
            fonts.font_data.insert(
                "boomer-mono".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "boomer-mono".to_owned());
        }
    }

    // The bold family always resolves: the bold face if found, then the
    // regular proportional stack as fallback.
    let mut bold_stack: Vec<String> = Vec::new();
    if bold_real {
        bold_stack.push("boomer-sans-bold".to_owned());
    }
    bold_stack.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    fonts.families.insert(bold_family(), bold_stack);

    ctx.set_fonts(fonts);
    ctx.data_mut(|d| d.insert_temp(bold_flag_id(), bold_real));
}
