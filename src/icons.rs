//! BevelDesk's hand-drawn pixel icons, ported from `theme95/icons95.cpp`.
//!
//! Icons come in two sizes: large (32×32, desktop) and small (14×14, captions,
//! taskbar buttons). They are drawn with painter primitives so they recolor
//! with the palette, exactly like the original.

use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2, vec2};
use egui::epaint::{CornerRadius, EllipseShape, Shape};

use crate::palette::{BLACK, GREEN, NAVY, Palette};

/// One of the built-in pixel icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    MyComputer,
    RecycleBin,
    Folder,
    Document,
    DosPrompt,
}

impl Icon {
    /// Paint the 32×32 desktop-sized version at `pos` (top-left corner).
    pub fn paint_large(self, p: &Painter, pos: Pos2, pal: &Palette) {
        match self {
            Self::MyComputer => my_computer_32(p, pos, pal),
            Self::RecycleBin => recycle_bin_32(p, pos, pal),
            Self::Folder => folder_32(p, pos, pal),
            Self::Document => document_32(p, pos, pal),
            Self::DosPrompt => dos_prompt_32(p, pos, pal),
        }
    }

    /// Paint the 14×14 caption/taskbar-sized version at `pos`.
    pub fn paint_small(self, p: &Painter, pos: Pos2, pal: &Palette) {
        match self {
            Self::MyComputer => mini_computer_14(p, pos, pal),
            Self::RecycleBin => mini_recycle_14(p, pos, pal),
            Self::Folder => folder_14(p, pos, pal),
            Self::Document => document_14(p, pos, pal),
            Self::DosPrompt => mini_dos_14(p, pos, pal),
        }
    }
}

// ---- drawing helpers, all in icon-local pixel coordinates -------------------

fn a(p: Pos2, x: f32, y: f32) -> Pos2 {
    pos2(p.x + x, p.y + y)
}

fn fill(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    pt.rect_filled(Rect::from_min_max(a(p, x0, y0), a(p, x1, y1)), CornerRadius::ZERO, c);
}

/// 1px outline just inside the rect, like ImGui's `AddRect`.
fn outline(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    fill(pt, p, x0, y0, x1, y0 + 1.0, c); // top
    fill(pt, p, x0, y1 - 1.0, x1, y1, c); // bottom
    fill(pt, p, x0, y0, x0 + 1.0, y1, c); // left
    fill(pt, p, x1 - 1.0, y0, x1, y1, c); // right
}

// Crate-internal aliases used by the shell's caption glyphs.
pub(crate) fn fill_px(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    fill(pt, p, x0, y0, x1, y1, c);
}
pub(crate) fn outline_px(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    outline(pt, p, x0, y0, x1, y1, c);
}
pub(crate) fn line_px(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    line(pt, p, x0, y0, x1, y1, c);
}

/// 1px line covering pixels from (x0,y0) to (x1,y1) inclusive, like `AddLine`.
fn line(pt: &Painter, p: Pos2, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    if y0 == y1 {
        fill(pt, p, x0.min(x1), y0, x0.max(x1) + 1.0, y0 + 1.0, c);
    } else if x0 == x1 {
        fill(pt, p, x0, y0.min(y1), x0 + 1.0, y0.max(y1) + 1.0, c);
    } else {
        pt.line_segment(
            [a(p, x0 + 0.5, y0 + 0.5), a(p, x1 + 0.5, y1 + 0.5)],
            Stroke::new(1.0, c),
        );
    }
}

// ---- 32×32 icons -------------------------------------------------------------

pub fn my_computer_32(pt: &Painter, p: Pos2, pal: &Palette) {
    // monitor
    fill(pt, p, 3.0, 0.0, 29.0, 19.0, pal.face);
    outline(pt, p, 3.0, 0.0, 29.0, 19.0, BLACK);
    line(pt, p, 4.0, 1.0, 27.0, 1.0, pal.hilight);
    line(pt, p, 4.0, 1.0, 4.0, 17.0, pal.hilight);
    // screen (navy, with a tiny "desktop" block)
    fill(pt, p, 6.0, 3.0, 26.0, 16.0, NAVY);
    outline(pt, p, 6.0, 3.0, 26.0, 16.0, pal.shadow);
    fill(pt, p, 8.0, 5.0, 16.0, 10.0, pal.desktop);
    fill(pt, p, 9.0, 6.0, 15.0, 7.0, pal.hilight);
    // stand
    fill(pt, p, 13.0, 19.0, 19.0, 21.0, pal.shadow);
    // desktop case
    fill(pt, p, 1.0, 21.0, 31.0, 30.0, pal.face);
    outline(pt, p, 1.0, 21.0, 31.0, 30.0, BLACK);
    line(pt, p, 2.0, 22.0, 29.0, 22.0, pal.hilight);
    line(pt, p, 2.0, 22.0, 2.0, 28.0, pal.hilight);
    line(pt, p, 2.0, 29.0, 30.0, 29.0, pal.shadow);
    // floppy slot + LED + button
    fill(pt, p, 5.0, 24.0, 19.0, 26.0, pal.shadow);
    fill(pt, p, 23.0, 24.0, 26.0, 26.0, GREEN);
    fill(pt, p, 27.0, 24.0, 29.0, 26.0, pal.shadow);
}

pub fn recycle_bin_32(pt: &Painter, p: Pos2, pal: &Palette) {
    // crumpled paper poking out
    fill(pt, p, 13.0, 1.0, 20.0, 6.0, pal.hilight);
    outline(pt, p, 13.0, 1.0, 20.0, 6.0, pal.shadow);
    // basket body (tapered)
    let body = vec![a(p, 7.0, 8.0), a(p, 25.0, 8.0), a(p, 22.0, 30.0), a(p, 10.0, 30.0)];
    pt.add(Shape::convex_polygon(body, pal.face, Stroke::new(1.0, BLACK)));
    // vertical shading stripes
    line(pt, p, 11.0, 10.0, 12.0, 28.0, pal.shadow);
    line(pt, p, 15.0, 10.0, 15.0, 28.0, pal.shadow);
    line(pt, p, 19.0, 10.0, 19.0, 28.0, pal.shadow);
    line(pt, p, 22.0, 10.0, 21.0, 28.0, pal.shadow);
    line(pt, p, 13.0, 10.0, 13.0, 28.0, pal.hilight);
    line(pt, p, 17.0, 10.0, 17.0, 28.0, pal.hilight);
    // elliptical opening with a lighter rim
    let ellipse = |center: Pos2, radius: egui::Vec2, fill: Color32, stroke: Stroke| {
        Shape::Ellipse(EllipseShape { center, radius, fill, stroke, angle: 0.0 })
    };
    pt.add(ellipse(a(p, 16.0, 8.0), vec2(9.0, 3.0), pal.shadow, Stroke::NONE));
    pt.add(ellipse(a(p, 16.0, 8.0), vec2(9.0, 3.0), Color32::TRANSPARENT, Stroke::new(1.0, BLACK)));
    pt.add(ellipse(a(p, 16.0, 7.0), vec2(9.0, 3.0), Color32::TRANSPARENT, Stroke::new(1.0, pal.hilight)));
    pt.add(ellipse(a(p, 16.0, 8.0), vec2(6.0, 1.5), Color32::TRANSPARENT, Stroke::new(1.0, pal.dkshadow)));
    // base rim
    line(pt, p, 10.0, 29.0, 22.0, 29.0, pal.shadow);
}

pub fn folder_32(pt: &Painter, p: Pos2, pal: &Palette) {
    // back tab
    fill(pt, p, 1.0, 6.0, 14.0, 11.0, pal.folder);
    outline(pt, p, 1.0, 6.0, 14.0, 11.0, pal.folder_edge);
    // body
    fill(pt, p, 1.0, 9.0, 31.0, 27.0, pal.folder);
    outline(pt, p, 1.0, 9.0, 31.0, 27.0, pal.folder_edge);
    line(pt, p, 2.0, 10.0, 29.0, 10.0, pal.hilight);
    line(pt, p, 2.0, 10.0, 2.0, 25.0, pal.hilight);
    line(pt, p, 2.0, 26.0, 30.0, 26.0, pal.folder_shade);
}

pub fn document_32(pt: &Painter, p: Pos2, pal: &Palette) {
    // page with folded corner
    fill(pt, p, 6.0, 1.0, 26.0, 31.0, pal.hilight);
    outline(pt, p, 6.0, 1.0, 26.0, 31.0, BLACK);
    pt.add(Shape::convex_polygon(
        vec![a(p, 19.0, 1.0), a(p, 26.0, 8.0), a(p, 19.0, 8.0)],
        pal.face,
        Stroke::NONE,
    ));
    line(pt, p, 19.0, 1.0, 19.0, 8.0, BLACK);
    line(pt, p, 19.0, 8.0, 26.0, 8.0, BLACK);
    for i in 0..5 {
        line(pt, p, 9.0, 12.0 + i as f32 * 3.0, 22.0, 12.0 + i as f32 * 3.0, pal.shadow);
    }
    line(pt, p, 9.0, 27.0, 17.0, 27.0, pal.shadow);
}

/// MS-DOS Prompt at desktop size — a 2× blow-up of BevelDesk's mini icon.
pub fn dos_prompt_32(pt: &Painter, p: Pos2, pal: &Palette) {
    fill(pt, p, 1.0, 3.0, 29.0, 25.0, pal.face);
    outline(pt, p, 1.0, 3.0, 29.0, 25.0, BLACK);
    fill(pt, p, 5.0, 7.0, 25.0, 21.0, BLACK);
    let gray = Color32::from_rgb(170, 170, 170);
    fill(pt, p, 7.0, 9.0, 13.0, 11.0, gray);
    fill(pt, p, 7.0, 13.0, 17.0, 15.0, gray);
    fill(pt, p, 7.0, 17.0, 11.0, 19.0, pal.hilight);
    // stand
    fill(pt, p, 11.0, 25.0, 19.0, 27.0, pal.shadow);
    fill(pt, p, 7.0, 27.0, 23.0, 30.0, pal.face);
    outline(pt, p, 7.0, 27.0, 23.0, 30.0, BLACK);
}

// ---- 14×14 icons -------------------------------------------------------------

pub fn folder_14(pt: &Painter, p: Pos2, pal: &Palette) {
    // back tab
    fill(pt, p, 0.0, 2.0, 6.0, 5.0, pal.folder);
    outline(pt, p, 0.0, 2.0, 6.0, 5.0, pal.folder_edge);
    // body
    fill(pt, p, 0.0, 4.0, 13.0, 12.0, pal.folder);
    outline(pt, p, 0.0, 4.0, 13.0, 12.0, pal.folder_edge);
    line(pt, p, 1.0, 5.0, 12.0, 5.0, pal.hilight);
}

pub fn document_14(pt: &Painter, p: Pos2, pal: &Palette) {
    // page with folded corner
    fill(pt, p, 2.0, 0.0, 11.0, 13.0, pal.hilight);
    outline(pt, p, 2.0, 0.0, 11.0, 13.0, BLACK);
    pt.add(Shape::convex_polygon(
        vec![a(p, 8.0, 0.0), a(p, 11.0, 3.0), a(p, 8.0, 3.0)],
        pal.face,
        Stroke::NONE,
    ));
    line(pt, p, 8.0, 0.0, 8.0, 3.0, BLACK);
    line(pt, p, 8.0, 3.0, 11.0, 3.0, BLACK);
    // text lines
    line(pt, p, 4.0, 5.0, 9.0, 5.0, pal.shadow);
    line(pt, p, 4.0, 7.0, 9.0, 7.0, pal.shadow);
    line(pt, p, 4.0, 9.0, 8.0, 9.0, pal.shadow);
}

pub fn mini_computer_14(pt: &Painter, p: Pos2, pal: &Palette) {
    fill(pt, p, 1.0, 1.0, 13.0, 9.0, pal.face);
    outline(pt, p, 1.0, 1.0, 13.0, 9.0, BLACK);
    fill(pt, p, 3.0, 3.0, 11.0, 7.0, pal.desktop);
    fill(pt, p, 5.0, 9.0, 9.0, 10.0, pal.shadow);
    fill(pt, p, 3.0, 10.0, 11.0, 12.0, pal.face);
    outline(pt, p, 3.0, 10.0, 11.0, 12.0, BLACK);
}

pub fn mini_dos_14(pt: &Painter, p: Pos2, pal: &Palette) {
    // MS-DOS window: gray frame, black screen, gray prompt lines + cursor
    fill(pt, p, 0.0, 1.0, 14.0, 12.0, pal.face);
    outline(pt, p, 0.0, 1.0, 14.0, 12.0, BLACK);
    fill(pt, p, 2.0, 3.0, 12.0, 10.0, BLACK);
    let gray = Color32::from_rgb(170, 170, 170);
    fill(pt, p, 3.0, 4.0, 6.0, 5.0, gray);
    fill(pt, p, 3.0, 6.0, 8.0, 7.0, gray);
    fill(pt, p, 3.0, 8.0, 5.0, 9.0, pal.hilight);
}

/// Small wastebasket (no BevelDesk original at this size; drawn in its style).
pub fn mini_recycle_14(pt: &Painter, p: Pos2, pal: &Palette) {
    let body = vec![a(p, 3.0, 3.0), a(p, 11.0, 3.0), a(p, 10.0, 13.0), a(p, 4.0, 13.0)];
    pt.add(Shape::convex_polygon(body, pal.face, Stroke::new(1.0, BLACK)));
    line(pt, p, 6.0, 5.0, 6.0, 11.0, pal.shadow);
    line(pt, p, 8.0, 5.0, 8.0, 11.0, pal.shadow);
    line(pt, p, 3.0, 3.0, 11.0, 3.0, pal.hilight);
    // paper
    fill(pt, p, 6.0, 0.0, 9.0, 3.0, pal.hilight);
    outline(pt, p, 6.0, 0.0, 9.0, 3.0, pal.shadow);
}

/// The BevelDesk logo: four raised beveled tiles — the mark IS the bevel.
///
/// 13×13; used on the taskbar Start button.
pub fn logo_13(pt: &Painter, p: Pos2, pal: &Palette) {
    let tile = |x: f32, y: f32, c: Color32| {
        fill(pt, p, x, y, x + 6.0, y + 6.0, c);
        fill(pt, p, x, y, x + 5.0, y + 1.0, pal.hilight); // top
        fill(pt, p, x, y, x + 1.0, y + 5.0, pal.hilight); // left
        fill(pt, p, x, y + 5.0, x + 6.0, y + 6.0, pal.dkshadow); // bottom
        fill(pt, p, x + 5.0, y, x + 6.0, y + 6.0, pal.dkshadow); // right
    };
    tile(0.0, 0.0, pal.desktop);
    tile(7.0, 0.0, pal.face);
    tile(0.0, 7.0, pal.face);
    tile(7.0, 7.0, pal.face);
}
