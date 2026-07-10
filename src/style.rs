//! Maps the BevelDesk palettes onto `egui::Style` — the "skin what you have"
//! layer that restyles every stock egui widget.
//!
//! What this reproduces from BevelDesk's `theme95/style.cpp` and metrics:
//! zero rounding everywhere, chrome-face fills, white/near-black sunken
//! fields, navy selection, gray/black keyline strokes, 16px solid scrollbars,
//! rectangular slider handles, dense Win95 spacing, 12px text, hard
//! unblurred window shadows, and no animations.
//!
//! egui widgets use a single uniform stroke, so stock widgets get the flat
//! keyline reading of the theme; the true two-tone 3D bevels live in
//! [`crate::bevel`] and the [`crate::shell`] components.

use egui::style::{HandleShape, ScrollAnimation, ScrollStyle, Selection, WidgetVisuals, Widgets};
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, Style, TextStyle, Visuals,
    vec2,
};

use crate::Scheme;
use crate::palette::Palette;

/// The "extreme" background (scroll tracks, progress troughs) per scheme;
/// also used by the skin plugin to recognize progress bars.
pub(crate) fn visuals_extreme_bg(pal: &Palette) -> Color32 {
    if pal.chiseled {
        Color32::from_rgb(20, 20, 20)
    } else {
        // BevelDesk's scrollbar-track gray.
        Color32::from_rgb(222, 222, 222)
    }
}

/// The complete [`egui::Style`] for a scheme: visuals, text sizes, spacing.
pub fn style(scheme: Scheme) -> Style {
    let mut style = Style {
        visuals: visuals(scheme),
        ..Style::default()
    };

    style.text_styles = [
        (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Heading, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = vec2(5.0, 4.0);
    spacing.button_padding = vec2(10.0, 3.0);
    spacing.interact_size = vec2(56.0, 22.0);
    spacing.window_margin = Margin::same(6);
    spacing.menu_margin = Margin::same(3);
    spacing.indent = 18.0;
    // Win95 check boxes and radios are 13px.
    spacing.icon_width = 13.0;
    spacing.icon_width_inner = 7.0;
    spacing.icon_spacing = 5.0;
    spacing.slider_rail_height = 4.0;
    spacing.scroll = ScrollStyle::solid();
    spacing.scroll.bar_width = 16.0;
    spacing.scroll.handle_min_length = 16.0;
    spacing.scroll.bar_inner_margin = 2.0;
    spacing.scroll.bar_outer_margin = 0.0;
    // Solid dark handle on the pale track — the keyline look; egui can't
    // draw the beveled Win95 thumb on its own scrollbars.
    spacing.scroll.foreground_color = true;

    // Win95 does not animate.
    style.animation_time = 0.0;
    style.scroll_animation = ScrollAnimation::none();

    style
}

/// Just the [`egui::Visuals`] for a scheme.
pub fn visuals(scheme: Scheme) -> Visuals {
    let p: &Palette = scheme.palette();
    let dark = scheme == Scheme::NextNight;
    let mut v = if dark { Visuals::dark() } else { Visuals::light() };

    let flat = CornerRadius::ZERO;
    let text_stroke = Stroke::new(1.0, p.text);

    v.dark_mode = dark;
    v.override_text_color = None;
    v.weak_text_color = Some(p.graytext);

    v.widgets = Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: p.face,
            weak_bg_fill: p.face,
            // Sentinel: separators and `ui.group()` frames become Win95
            // etched grooves (see `crate::skin`).
            bg_stroke: Stroke::new(1.0, crate::skin::MARK_ETCH),
            corner_radius: flat,
            fg_stroke: text_stroke,
            expansion: 0.0,
        },
        // `bg_fill` feeds "must-have-a-background" parts (slider rail and
        // handle, radio circle, check boxes): Win95 sunken-field white.
        // `weak_bg_fill` feeds button faces: chrome gray in every state —
        // real Win95 buttons don't change color on hover or press, the bevel
        // flips instead. The stroke is a sentinel the skin plugin rewrites
        // into the real two-tone bevel (see `crate::skin`); without the
        // plugin it reads as a plain near-black keyline.
        inactive: WidgetVisuals {
            bg_fill: p.winbg,
            weak_bg_fill: p.face,
            bg_stroke: Stroke::new(1.0, crate::skin::MARK_RAISED),
            corner_radius: flat,
            fg_stroke: text_stroke,
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            // The hover sentinel (visually the field color) lets the skin
            // light the slider thumb while keeping fields/radios unchanged.
            bg_fill: crate::skin::field_hover(p),
            // A touch lighter face on hover — a small modern affordance the
            // bevel rewrite preserves as a raised button.
            weak_bg_fill: p.light,
            bg_stroke: Stroke::new(1.0, crate::skin::MARK_RAISED),
            corner_radius: flat,
            fg_stroke: text_stroke,
            expansion: 0.0,
        },
        active: WidgetVisuals {
            bg_fill: crate::skin::field_active(p),
            weak_bg_fill: p.face,
            bg_stroke: Stroke::new(1.0, crate::skin::MARK_PRESSED),
            corner_radius: flat,
            fg_stroke: text_stroke,
            expansion: 0.0,
        },
        open: WidgetVisuals {
            bg_fill: p.winbg,
            // Selection fill: an open menu-bar title turns navy (with white
            // text via the skin), exactly like Win95.
            weak_bg_fill: p.sel,
            // Pressed marker: an open combo box keeps its arrow depressed.
            bg_stroke: Stroke::new(1.0, crate::skin::MARK_PRESSED),
            corner_radius: flat,
            fg_stroke: text_stroke,
            expansion: 0.0,
        },
    };

    v.selection = Selection {
        bg_fill: p.sel,
        // Near-white sentinel: doubles as the selected-text color (white,
        // like Win95) and lets the skin plugin keep focused text edits
        // sunken instead of egui's focus outline.
        stroke: Stroke::new(1.0, crate::skin::MARK_FOCUS),
    };

    v.hyperlink_color = p.accent;
    v.faint_bg_color = if dark {
        Color32::from_rgb(50, 50, 50)
    } else {
        // BevelDesk's scrollbar-track gray.
        Color32::from_rgb(222, 222, 222)
    };
    // Scroll tracks and other "extreme" fills: BevelDesk's scrollbar-track
    // tone. Text fields keep the true field color via `text_edit_bg_color`.
    v.extreme_bg_color = visuals_extreme_bg(p);
    v.text_edit_bg_color = Some(p.winbg);
    v.code_bg_color = p.winbg;

    v.window_fill = p.face;
    // Sentinel: `egui::Window`s get the Win95 window-frame bevel; menus,
    // tooltips, and popups get the raised menu bevel (see `crate::skin`).
    v.window_stroke = Stroke::new(1.0, crate::skin::MARK_WINDOW);
    v.window_corner_radius = flat;
    // Hard, unblurred offset shadow — crisp like a NeXT window.
    v.window_shadow = Shadow {
        offset: [2, 2],
        blur: 0,
        spread: 0,
        color: Color32::from_black_alpha(96),
    };
    v.popup_shadow = Shadow {
        offset: [1, 1],
        blur: 0,
        spread: 0,
        color: Color32::from_black_alpha(96),
    };
    v.window_highlight_topmost = false;
    v.menu_corner_radius = flat;
    v.panel_fill = p.face;

    v.resize_corner_size = 12.0;
    v.text_cursor.stroke = Stroke::new(1.0, p.text);
    v.text_cursor.blink = true;

    v.striped = false;
    v.slider_trailing_fill = false;
    v.handle_shape = HandleShape::Rect { aspect_ratio: 0.6 };
    v.image_loading_spinners = false;

    v
}
