//! The BevelDesk color schemes, ported verbatim from `theme95/palette.cpp`.
//!
//! Two palettes ship with the crate:
//! - [`SILVER`] — the exact Windows 95 "Windows Standard" colors.
//! - [`NEXT_NIGHT`] — charcoal chrome, black keylines, muted-steel captions.

use egui::Color32;

/// A complete BevelDesk chrome palette.
///
/// Field names follow the Win95 system-color vocabulary used by BevelDesk:
/// `face` is `COLOR_3DFACE`, `hilight` is the white bevel edge, `dkshadow`
/// the black one, and so on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    /// Button/window chrome face (Silver: the classic 192 gray).
    pub face: Color32,
    /// Inner light bevel edge (`3DLIGHT`).
    pub light: Color32,
    /// Outer light bevel edge (`3DHILIGHT`).
    pub hilight: Color32,
    /// Inner dark bevel edge (`3DSHADOW`).
    pub shadow: Color32,
    /// Outer dark bevel edge (`3DDKSHADOW`).
    pub dkshadow: Color32,

    /// Active title bar gradient, left color.
    pub title_act: Color32,
    /// Active title bar gradient, right color.
    pub title_act2: Color32,
    /// Inactive title bar gradient, left color.
    pub title_inact: Color32,
    /// Inactive title bar gradient, right color.
    pub title_inact2: Color32,
    /// Title bar text.
    pub title_text: Color32,

    /// Desktop background (Silver: the famous teal).
    pub desktop: Color32,
    /// Window/field background (text edits, lists).
    pub winbg: Color32,
    /// Normal text.
    pub text: Color32,
    /// Disabled text.
    pub graytext: Color32,
    /// Selection background (Silver: navy).
    pub sel: Color32,
    /// Selected text.
    pub sel_text: Color32,

    /// Folder icon fill.
    pub folder: Color32,
    /// Folder icon outline.
    pub folder_edge: Color32,
    /// Folder icon inner shade.
    pub folder_shade: Color32,
    /// Small UI accent (Silver: blue; Night: steel).
    pub accent: Color32,

    /// NeXT-style black keyline around bevels (true for [`NEXT_NIGHT`]).
    pub chiseled: bool,
}

/// Semantic black — never changes per scheme.
pub const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
/// "My Computer" power-LED green.
pub const GREEN: Color32 = Color32::from_rgb(0, 128, 0);
/// "My Computer" screen navy.
pub const NAVY: Color32 = Color32::from_rgb(0, 0, 128);

// ---- metrics (96 dpi Win95 defaults) ----------------------------------------

/// Title bar height in points.
pub const TITLEBAR_H: f32 = 18.0;
/// Window sizing-border thickness.
pub const WIN_BORDER: f32 = 4.0;
/// Taskbar height.
pub const TASKBAR_H: f32 = 28.0;
/// Caption button width.
pub const CAPBTN_W: f32 = 16.0;
/// Caption button height.
pub const CAPBTN_H: f32 = 14.0;

/// Silver — the exact Win95 "Windows Standard" values.
pub const SILVER: Palette = Palette {
    face: Color32::from_rgb(192, 192, 192),
    light: Color32::from_rgb(223, 223, 223),
    hilight: Color32::from_rgb(255, 255, 255),
    shadow: Color32::from_rgb(128, 128, 128),
    dkshadow: Color32::from_rgb(0, 0, 0),

    title_act: Color32::from_rgb(0, 0, 128),
    title_act2: Color32::from_rgb(16, 132, 208),
    title_inact: Color32::from_rgb(128, 128, 128),
    title_inact2: Color32::from_rgb(181, 181, 181),
    title_text: Color32::from_rgb(255, 255, 255),

    desktop: Color32::from_rgb(0, 128, 128),
    winbg: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(0, 0, 0),
    graytext: Color32::from_rgb(128, 128, 128),
    sel: Color32::from_rgb(0, 0, 128),
    sel_text: Color32::from_rgb(255, 255, 255),

    folder: Color32::from_rgb(255, 255, 128),
    folder_edge: Color32::from_rgb(128, 128, 0),
    folder_shade: Color32::from_rgb(160, 160, 0),
    accent: Color32::from_rgb(0, 0, 255),

    chiseled: false,
};

/// NeXT Night — charcoal chrome, lighter-gray bevel highlights, black keyline,
/// dark fields, light text, a muted-steel active caption and muted-gold folders.
pub const NEXT_NIGHT: Palette = Palette {
    face: Color32::from_rgb(60, 60, 60),
    light: Color32::from_rgb(86, 86, 86),
    hilight: Color32::from_rgb(110, 110, 110),
    shadow: Color32::from_rgb(37, 37, 37),
    dkshadow: Color32::from_rgb(0, 0, 0),

    title_act: Color32::from_rgb(45, 53, 66),
    title_act2: Color32::from_rgb(74, 92, 116),
    title_inact: Color32::from_rgb(42, 42, 42),
    title_inact2: Color32::from_rgb(53, 53, 53),
    title_text: Color32::from_rgb(224, 224, 224),

    desktop: Color32::from_rgb(43, 43, 43),
    winbg: Color32::from_rgb(27, 27, 27),
    text: Color32::from_rgb(230, 230, 230),
    graytext: Color32::from_rgb(106, 106, 106),
    sel: Color32::from_rgb(59, 90, 138),
    sel_text: Color32::from_rgb(255, 255, 255),

    folder: Color32::from_rgb(192, 172, 102),
    folder_edge: Color32::from_rgb(110, 98, 48),
    folder_shade: Color32::from_rgb(150, 134, 72),
    accent: Color32::from_rgb(126, 146, 176),

    chiseled: true,
};

impl Palette {
    /// The palette matching the context's active visuals: [`SILVER`] when the
    /// current theme is light, [`NEXT_NIGHT`] when it is dark.
    ///
    /// This is how the shell components ([`crate::shell`]) pick their colors,
    /// so they follow whatever [`crate::install`] or [`crate::apply`] set up.
    pub fn current(ctx: &egui::Context) -> &'static Palette {
        if ctx.global_style().visuals.dark_mode {
            &NEXT_NIGHT
        } else {
            &SILVER
        }
    }
}
