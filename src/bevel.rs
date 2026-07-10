//! The four Win95 3D edge conventions (light always from the top-left),
//! ported from BevelDesk's `theme95/bevel.cpp`.
//!
//! Everything is drawn as 1px filled rects for pixel-crisp edges, exactly like
//! the original's `AddRectFilled` strips. In the NeXT Night palette
//! (`chiseled == true`) bevels gain a black keyline and drop to a single inner
//! edge, matching NeXTSTEP.
//!
//! These are the primitives the [`crate::shell`] components are built from;
//! they are public so you can frame your own custom widgets the same way:
//!
//! ```no_run
//! # egui::__run_test_ui(|ui| {
//! use egui_theme_boomer::{bevel, Palette};
//! let pal = Palette::current(ui.ctx());
//! let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 40.0), egui::Sense::hover());
//! bevel::raised(ui.painter(), rect, pal, true);
//! # });
//! ```

use egui::epaint::Shape;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, pos2};

use crate::palette::Palette;

fn strip(min: Pos2, max: Pos2, color: Color32) -> Shape {
    Shape::rect_filled(Rect::from_min_max(min, max), CornerRadius::ZERO, color)
}

/// The 1px two-tone frame as shapes; bottom/right win the corners, matching
/// Win95's `DrawEdge` output.
pub(crate) fn edge_shapes(r: Rect, tl: Color32, br: Color32, out: &mut Vec<Shape>) {
    out.push(strip(r.min, pos2(r.max.x - 1.0, r.min.y + 1.0), tl)); // top
    out.push(strip(r.min, pos2(r.min.x + 1.0, r.max.y - 1.0), tl)); // left
    out.push(strip(pos2(r.min.x, r.max.y - 1.0), r.max, br)); // bottom
    out.push(strip(pos2(r.max.x - 1.0, r.min.y), r.max, br)); // right
}

/// Which of the four Win95 edge conventions to build.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Raised,
    Pressed,
    Sunken,
    WindowFrame,
}

/// The bevel as ready-to-paint shapes: optional fill plus the two 1px rings.
pub(crate) fn shapes(kind: Kind, r: Rect, pal: &Palette, fill: Option<Color32>) -> Vec<Shape> {
    let mut out = Vec::with_capacity(9);
    if let Some(color) = fill {
        out.push(Shape::rect_filled(r, CornerRadius::ZERO, color));
    }
    let (outer, inner) = if pal.chiseled {
        // NeXT: black keyline outside, one bevel ring inside.
        let keyline = (pal.dkshadow, pal.dkshadow);
        match kind {
            Kind::Raised | Kind::WindowFrame => (keyline, (pal.hilight, pal.shadow)),
            Kind::Pressed => (keyline, (pal.shadow, pal.hilight)),
            Kind::Sunken => (keyline, (pal.shadow, pal.light)),
        }
    } else {
        match kind {
            Kind::Raised => ((pal.hilight, pal.dkshadow), (pal.light, pal.shadow)),
            Kind::Pressed => ((pal.dkshadow, pal.hilight), (pal.shadow, pal.light)),
            Kind::Sunken => ((pal.shadow, pal.hilight), (pal.dkshadow, pal.light)),
            Kind::WindowFrame => ((pal.light, pal.dkshadow), (pal.hilight, pal.shadow)),
        }
    };
    edge_shapes(r, outer.0, outer.1, &mut out);
    edge_shapes(r.shrink(1.0), inner.0, inner.1, &mut out);
    out
}

/// Paint a bevel with an explicit fill (e.g. the lighter hover face).
pub(crate) fn paint(kind: Kind, p: &Painter, r: Rect, pal: &Palette, fill: Option<Color32>) {
    p.add(Shape::Vec(shapes(kind, r, pal, fill)));
}

/// 1px frame with distinct top-left / bottom-right colors.
///
/// Bottom/right win the corners, matching Win95's `DrawEdge` output.
pub fn edge(p: &Painter, r: Rect, tl: Color32, br: Color32) {
    let mut out = Vec::with_capacity(4);
    edge_shapes(r, tl, br, &mut out);
    p.add(Shape::Vec(out));
}

/// 2px raised button bevel: outer TL white / BR black, inner TL light / BR shadow.
pub fn raised(p: &Painter, r: Rect, pal: &Palette, fill: bool) {
    p.add(Shape::Vec(shapes(Kind::Raised, r, pal, fill.then_some(pal.face))));
}

/// 2px pressed button bevel (inverse of [`raised`]) — shift the label by +1,+1.
pub fn pressed(p: &Painter, r: Rect, pal: &Palette, fill: bool) {
    p.add(Shape::Vec(shapes(Kind::Pressed, r, pal, fill.then_some(pal.face))));
}

/// 2px sunken field (edit/list view): outer TL shadow / BR white, inner TL black / BR light.
///
/// Pass `Some(color)` to fill the field (usually `pal.winbg`), `None` to only
/// draw the edges.
pub fn sunken_field(p: &Painter, r: Rect, pal: &Palette, fill: Option<Color32>) {
    p.add(Shape::Vec(shapes(Kind::Sunken, r, pal, fill)));
}

/// 2px window frame: outer TL light / BR black, inner TL white / BR shadow.
pub fn window_frame(p: &Painter, r: Rect, pal: &Palette, fill: bool) {
    p.add(Shape::Vec(shapes(Kind::WindowFrame, r, pal, fill.then_some(pal.face))));
}

/// 1px sunken groove (status bar wells, separators).
pub fn thin_sunken(p: &Painter, r: Rect, pal: &Palette) {
    edge(p, r, pal.shadow, pal.hilight);
}

/// 1px raised edge (hover state of flat toolbar buttons).
pub fn thin_raised(p: &Painter, r: Rect, pal: &Palette) {
    edge(p, r, pal.hilight, pal.shadow);
}
