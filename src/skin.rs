//! The pass that makes *stock* egui widgets pixel-exact Win95 controls.
//!
//! egui widgets can only draw uniform strokes and simple fills, so a `Style`
//! alone can't produce two-tone 3D chrome. Instead, [`crate::style`] paints
//! interactive widget backgrounds with sentinel stroke colors, and an
//! [`egui::Plugin`] registered by [`crate::install`] / [`crate::apply`]
//! rewrites the emitted shapes at the end of every pass:
//!
//! - face-colored marked rects → raised / pressed button bevels, and the
//!   button's contents (label, icons) shift +1,+1 while pressed, like Win95
//! - selected stock buttons → latched pressed bevels; menu selections remain
//!   flat navy rows
//! - field-colored marked rects → sunken fields (text edits stay sunken
//!   when focused; check boxes get the classic pixel check mark)
//! - marked circles → the sunken-ring radio button
//! - combo boxes → sunken field + beveled arrow button
//! - progress bars → sunken trough + segmented selection-colored blocks
//! - sliders → sunken channel + raised trackbar thumb
//! - large dark rounded canvases → square sunken display wells, with their
//!   plots/previews clipped inside the client edge
//!
//! So a plain `ui.button("OK")`, `ui.checkbox(…)`, `egui::ComboBox`,
//! `egui::ProgressBar`, or `egui::Slider` renders the exact Win95 control
//! with no code changes.

use egui::epaint::{ClippedShape, Shape};
use egui::layers::ShapeIdx;
use egui::{Color32, Context, CornerRadius, LayerId, Rect, Stroke, Ui, pos2, vec2};

use crate::bevel;
use crate::palette::Palette;

/// Sentinel stroke colors; imperceptibly off-black so they can double as a
/// plain keyline if the plugin isn't running (e.g. `style()` used alone).
pub(crate) const MARK_RAISED: Color32 = Color32::from_rgb(1, 2, 3);
pub(crate) const MARK_PRESSED: Color32 = Color32::from_rgb(3, 2, 1);
/// Separator/group sentinel (`noninteractive.bg_stroke`): rewritten into the
/// Win95 etched groove (shadow line with a white line one pixel after).
pub(crate) const MARK_ETCH: Color32 = Color32::from_rgb(2, 3, 1);
/// Window/popup frame sentinel (`window_stroke`): menus, tooltips, and combo
/// popups get the raised bevel; `egui::Window`s get the window-frame bevel.
pub(crate) const MARK_WINDOW: Color32 = Color32::from_rgb(2, 1, 3);
/// Focus-frame sentinel (`selection.stroke`). Near-white, because the same
/// color is also used for selected text, which Win95 renders white.
pub(crate) const MARK_FOCUS: Color32 = Color32::from_rgb(255, 254, 253);

/// Hovered `bg_fill` sentinel: imperceptibly off the field color. Lets the
/// rewriter light up the slider thumb (only when the pointer is on the thumb
/// itself) while radio circles and check boxes get normalized back to the
/// true field color.
pub(crate) fn field_hover(pal: &Palette) -> Color32 {
    if pal.chiseled {
        Color32::from_rgb(27, 27, 28)
    } else {
        Color32::from_rgb(255, 255, 254)
    }
}

/// Active (dragged) `bg_fill` sentinel: the thumb is captured, so it stays
/// lit wherever the pointer is.
pub(crate) fn field_active(pal: &Palette) -> Color32 {
    if pal.chiseled {
        Color32::from_rgb(28, 27, 27)
    } else {
        Color32::from_rgb(254, 255, 255)
    }
}

fn is_field_color(fill: Color32, pal: &Palette) -> bool {
    fill == pal.winbg || fill == field_hover(pal) || fill == field_active(pal)
}

/// Sentinel fills back to the true field color.
fn normalize_field(fill: Color32, pal: &Palette) -> Color32 {
    if fill == field_hover(pal) || fill == field_active(pal) {
        pal.winbg
    } else {
        fill
    }
}

pub(crate) fn install_plugin(ctx: &Context) {
    let flag = egui::Id::new("egui-theme-boomer-skin-plugin");
    if !ctx.data(|d| d.get_temp::<bool>(flag).unwrap_or(false)) {
        ctx.add_plugin(BevelSkin);
        ctx.data_mut(|d| d.insert_temp(flag, true));
    }
}

struct BevelSkin;

impl egui::Plugin for BevelSkin {
    fn debug_name(&self) -> &'static str {
        "egui_theme_boomer::BevelSkin"
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let pal = Palette::current(&ctx);
        let pointer = ctx.input(|i| i.pointer.latest_pos());
        let mut layers: Vec<LayerId> =
            ctx.memory(|m| m.areas().visible_layer_ids().into_iter().collect());
        layers.push(LayerId::background());
        ctx.graphics_mut(|graphics| {
            for layer in layers {
                let Some(list) = graphics.get_mut(layer) else {
                    continue;
                };
                let actions = classify(list.all_entries(), pal, pointer, layer.order);
                for (idx, action) in actions {
                    list.mutate_shape(ShapeIdx(idx), |clipped| apply(clipped, &action, pal));
                }
            }
        });
    }
}

/// What to do to one shape, decided by [`classify`], executed by [`apply`].
enum Action {
    /// Marked rect → bevel (raised, pressed, or sunken by fill).
    Bevel(bevel::Kind, Color32),
    /// Marked rect with transparent fill (immutable text edit): no chrome.
    Remove,
    /// Selection-highlight rect that inherited a marker: flat navy, like Win95.
    Flatten,
    /// Marked circle → sunken-ring radio.
    Radio,
    /// Check-mark path inside a check box → the classic pixel check.
    Check(Rect),
    /// Combo rect → sunken field + beveled arrow button.
    ComboField { pressed: bool },
    /// Combo arrow triangle → recentered on the arrow button.
    ComboArrow { btn: Rect, pressed: bool },
    /// Slider rail → sunken channel.
    Channel,
    /// Slider handle → raised trackbar thumb (`lit` while hovered/dragged).
    Thumb { lit: bool },
    /// Progress trough → face fill + thin sunken edge.
    Trough,
    /// Progress fill → segmented blocks.
    Blocks { trough: Rect },
    /// Separator line → etched groove (shadow + white, one pixel apart).
    Etch,
    /// Group frame → etched groove ring.
    EtchRect,
    /// Hovered menu/popup row → flat selection fill.
    MenuRow,
    /// Tooltip frame → flat pale yellow with a 1px black keyline.
    Tooltip,
    /// Text on a selection-colored row → white, like Win95.
    WhiteText,
    /// Content of a pressed button: shift +1,+1.
    Shift,
    /// Text of a latched button: restore normal button text and shift +1,+1.
    PressedText,
    /// Keep custom display contents inside the recessed client edge.
    InsetClip(Rect),
}

fn mark_of(stroke: Stroke) -> Option<bevel::Kind> {
    if stroke.color == MARK_RAISED || stroke.color == MARK_FOCUS {
        Some(bevel::Kind::Raised)
    } else if stroke.color == MARK_PRESSED {
        Some(bevel::Kind::Pressed)
    } else {
        None
    }
}

fn pill_radius(cr: CornerRadius, height: f32) -> bool {
    let half = (height / 2.0) as u8;
    cr.nw >= half.saturating_sub(1) && cr.nw != 0
}

/// A raw, dark canvas painted by an app. These are the usual shape of plots,
/// scopes, previews, and other display surfaces. The size and luminance guards
/// keep small decorative rounded rects out of the widget-chrome pass.
fn is_display_surface(rs: &egui::epaint::RectShape) -> bool {
    let rect = rs.rect;
    rs.stroke == Stroke::NONE
        && rs.corner_radius != CornerRadius::ZERO
        && rect.width() >= 40.0
        && rect.height() >= 40.0
        && rs.fill.a() >= 96
        && rs.fill.r() <= 20
        && rs.fill.g() <= 20
        && rs.fill.b() <= 20
}

fn has_selected_button_text(shapes: &[&Shape], i: usize, rect: Rect) -> bool {
    shapes.iter().skip(i + 1).take(12).any(|shape| {
        let Shape::Text(ts) = shape else {
            return false;
        };
        rect.contains_rect(shape.visual_bounding_rect())
            && (ts.fallback_color == MARK_FOCUS
                || ts.override_text_color == Some(MARK_FOCUS))
    })
}

fn classify<'a>(
    entries: impl Iterator<Item = &'a ClippedShape>,
    pal: &Palette,
    pointer: Option<egui::Pos2>,
    order: egui::Order,
) -> Vec<(usize, Action)> {
    let shapes: Vec<&Shape> = entries.map(|c| &c.shape).collect();
    let mut actions: Vec<(usize, Action)> = Vec::new();
    let mut claimed: Vec<usize> = Vec::new();
    let mut pressed_rects: Vec<Rect> = Vec::new();
    let mut selected_pressed_rects: Vec<Rect> = Vec::new();
    let mut display_wells: Vec<(usize, Rect)> = Vec::new();
    let mut menu_rows: Vec<Rect> = Vec::new();
    let mut pending_check: Option<(Rect, usize)> = None;
    let mut last_rail: Option<Rect> = None;
    let mut last_trough: Option<Rect> = None;
    let mut last_track: Option<Rect> = None;
    let foreground = order == egui::Order::Foreground;
    let tooltip = order == egui::Order::Tooltip;

    for (i, shape) in shapes.iter().enumerate() {
        if claimed.contains(&i) {
            continue;
        }
        match shape {
            Shape::Rect(rs) => {
                let rect = rs.rect;
                // Selection-colored rects: the progress fill, selection
                // highlights, and open menu-bar titles / selected rows —
                // all flat navy with white text, like Win95.
                if rs.fill == pal.sel {
                    if pill_radius(rs.corner_radius, rect.height())
                        && last_trough.is_some_and(|t| t.expand(1.0).contains_rect(rect))
                    {
                        let trough = last_trough.take().unwrap();
                        actions.push((i, Action::Blocks { trough }));
                    } else if let Some(j) = find_combo_arrow(&shapes, i, rect, pal) {
                        // An open combo box (open state borrows the
                        // selection fill): sunken field + depressed arrow.
                        actions.push((i, Action::ComboField { pressed: true }));
                        actions.push((j, Action::ComboArrow { btn: combo_btn(rect), pressed: true }));
                        claimed.push(j);
                    } else if rs.stroke.color == MARK_RAISED
                        || (rs.stroke.color == MARK_PRESSED
                            && has_selected_button_text(&shapes, i, rect))
                    {
                        // A selected stock Button is Win32's BS_PUSHLIKE
                        // checked state: it stays physically depressed. Menu
                        // and combo rows have no stroke because egui applies
                        // `menu_style`, so they still take the flat-selection
                        // branch below.
                        actions.push((
                            i,
                            Action::Bevel(bevel::Kind::Pressed, pal.face),
                        ));
                        pressed_rects.push(rect);
                        selected_pressed_rects.push(rect);
                    } else {
                        actions.push((i, Action::Flatten));
                        menu_rows.push(rect);
                    }
                    continue;
                }
                if rs.stroke.color == MARK_ETCH {
                    actions.push((i, Action::EtchRect));
                    continue;
                }
                if is_display_surface(rs) {
                    actions.push((i, Action::Bevel(bevel::Kind::Sunken, rs.fill)));
                    display_wells.push((i, rect.shrink(2.0)));
                    continue;
                }
                // Window / popup / tooltip frames.
                if rs.stroke.color == MARK_WINDOW {
                    if tooltip {
                        // Win95 tooltip: flat pale yellow, 1px black keyline.
                        actions.push((i, Action::Tooltip));
                    } else if foreground {
                        actions.push((i, Action::Bevel(bevel::Kind::Raised, rs.fill)));
                    } else {
                        actions.push((i, Action::Bevel(bevel::Kind::WindowFrame, rs.fill)));
                    }
                    continue;
                }
                // Hovered/active rows inside menus and popups: egui's menu
                // style strips the marker stroke, so recognize them by
                // shape — flat navy with white text, like Win95.
                if foreground
                    && rs.stroke == Stroke::NONE
                    && (rs.fill == pal.light || rs.fill == pal.face)
                    && rect.height() <= 26.0
                    && rect.width() >= rect.height()
                {
                    actions.push((i, Action::MenuRow));
                    menu_rows.push(rect);
                    continue;
                }
                if let Some(kind) = mark_of(rs.stroke) {
                    if rs.fill == Color32::TRANSPARENT {
                        actions.push((i, Action::Remove));
                    } else if rs.fill == pal.face || rs.fill == pal.light {
                        // A right-side triangle just after = a combo box.
                        if let Some(j) = find_combo_arrow(&shapes, i, rect, pal) {
                            let pressed = matches!(kind, bevel::Kind::Pressed);
                            actions.push((i, Action::ComboField { pressed }));
                            actions.push((j, Action::ComboArrow { btn: combo_btn(rect), pressed }));
                            claimed.push(j);
                        } else {
                            actions.push((i, Action::Bevel(kind, rs.fill)));
                            if matches!(kind, bevel::Kind::Pressed) {
                                pressed_rects.push(rect);
                            }
                        }
                    } else {
                        // Fields never light up on hover: normalize the
                        // sentinel fills back to the true field color.
                        let fill = normalize_field(rs.fill, pal);
                        actions.push((i, Action::Bevel(bevel::Kind::Sunken, fill)));
                        if rect.width() <= 22.0 && rect.height() <= 22.0 {
                            pending_check = Some((rect, i));
                        }
                    }
                    continue;
                }
                // Unmarked rects: slider rail / thumb, progress trough.
                if rs.stroke == Stroke::NONE
                    && rs.fill == pal.winbg
                    && (rect.height() - 4.0).abs() < 0.75
                    && rect.width() >= rect.height() * 4.0
                {
                    actions.push((i, Action::Channel));
                    last_rail = Some(rect);
                } else if rs.stroke.color == pal.text
                    && is_field_color(rs.fill, pal)
                    && last_rail.is_some_and(|r| {
                        (rect.center().y - r.center().y).abs() < 12.0
                            && rect.min.x >= r.min.x - 12.0
                            && rect.max.x <= r.max.x + 12.0
                    })
                {
                    // Dragging keeps the thumb lit (it's captured); hovering
                    // lights it only when the pointer is on the thumb itself,
                    // not just somewhere on the slider.
                    let lit = rs.fill == field_active(pal)
                        || (rs.fill == field_hover(pal)
                            && pointer.is_some_and(|p| rect.contains(p)));
                    actions.push((i, Action::Thumb { lit }));
                } else if rs.fill == crate::style::visuals_extreme_bg(pal)
                    && pill_radius(rs.corner_radius, rect.height())
                {
                    actions.push((i, Action::Trough));
                    last_trough = Some(rect);
                } else if rs.fill == crate::style::visuals_extreme_bg(pal)
                    && rs.corner_radius == CornerRadius::ZERO
                    && rs.stroke == Stroke::NONE
                    && rect.width().min(rect.height()) >= 10.0
                    && rect.width().min(rect.height()) <= 20.0
                    && rect.width().max(rect.height()) >= rect.width().min(rect.height()) * 2.0
                {
                    // A scrollbar track; the handle follows.
                    last_track = Some(rect);
                } else if rs.fill == pal.text
                    && rs.stroke == Stroke::NONE
                    && last_track.is_some_and(|t| t.expand(1.0).contains_rect(rect))
                {
                    // The scroll handle (foreground-color mode): a raised
                    // beveled thumb, lit while the pointer is on it.
                    let lit = pointer.is_some_and(|p| rect.contains(p));
                    actions.push((i, Action::Thumb { lit }));
                }
            }
            Shape::Circle(cs) => {
                if mark_of(cs.stroke).is_some() {
                    actions.push((i, Action::Radio));
                }
            }
            Shape::LineSegment { stroke, .. } => {
                if stroke.color == MARK_ETCH {
                    actions.push((i, Action::Etch));
                } else if pressed_rects
                    .iter()
                    .any(|pr| pr.contains_rect(shape.visual_bounding_rect()))
                {
                    actions.push((i, Action::Shift));
                }
            }
            Shape::Path(ps) => {
                if let Some((box_rect, box_idx)) = pending_check
                    && i - box_idx <= 4
                    && ps.points.len() == 3
                    && box_rect
                        .expand(2.0)
                        .contains_rect(shape.visual_bounding_rect())
                {
                    actions.push((i, Action::Check(box_rect)));
                    pending_check = None;
                    continue;
                }
                if let Some(pr) = pressed_rects.iter().find(|pr| {
                    pr.contains_rect(shape.visual_bounding_rect())
                }) {
                    let _ = pr;
                    actions.push((i, Action::Shift));
                }
            }
            Shape::Text(_) => {
                let bounds = shape.visual_bounding_rect();
                if menu_rows.iter().any(|r| r.contains_rect(bounds)) {
                    actions.push((i, Action::WhiteText));
                } else if selected_pressed_rects
                    .iter()
                    .any(|pr| pr.contains_rect(bounds))
                {
                    actions.push((i, Action::PressedText));
                } else if pressed_rects.iter().any(|pr| pr.contains_rect(bounds)) {
                    actions.push((i, Action::Shift));
                }
            }
            other => {
                // Shift pressed-button contents (labels, icons) by +1,+1.
                let bounds = other.visual_bounding_rect();
                if bounds.is_finite()
                    && pressed_rects.iter().any(|pr| pr.contains_rect(bounds))
                {
                    actions.push((i, Action::Shift));
                }
            }
        }
    }

    // A recessed display's client edge must win over plot lines/bars that
    // were deliberately drawn all the way to the original rounded rect.
    // Tighten their existing clip rather than translating or rescaling data.
    for (well_idx, inner) in display_wells {
        let outer = inner.expand(4.0);
        for (i, shape) in shapes.iter().enumerate().skip(well_idx + 1) {
            let bounds = shape.visual_bounding_rect();
            if bounds.is_finite() && outer.contains_rect(bounds) {
                actions.push((i, Action::InsetClip(inner)));
            }
        }
    }

    actions
}

/// The down-arrow triangle egui paints inside a combo button: a 3-point
/// filled path in the right half of the marked rect, within a few shapes.
fn find_combo_arrow(shapes: &[&Shape], i: usize, rect: Rect, pal: &Palette) -> Option<usize> {
    for (j, shape) in shapes.iter().enumerate().skip(i + 1).take(5) {
        if let Shape::Path(ps) = shape
            && ps.points.len() == 3
            && ps.fill == pal.text
        {
            let bounds = shape.visual_bounding_rect();
            if rect.contains_rect(bounds) && bounds.center().x > rect.center().x {
                return Some(j);
            }
        }
    }
    None
}

/// Where the beveled arrow button sits inside a combo field.
fn combo_btn(rect: Rect) -> Rect {
    Rect::from_min_max(
        pos2(rect.max.x - 2.0 - 16.0, rect.min.y + 2.0),
        pos2(rect.max.x - 2.0, rect.max.y - 2.0),
    )
}

fn apply(clipped: &mut ClippedShape, action: &Action, pal: &Palette) {
    let shape = &mut clipped.shape;
    match action {
        Action::Bevel(kind, fill) => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                *shape = Shape::Vec(bevel::shapes(*kind, rect, pal, Some(*fill)));
            }
        }
        Action::Remove => *shape = Shape::Noop,
        Action::Flatten => {
            if let Shape::Rect(rs) = shape {
                rs.stroke = Stroke::NONE;
                rs.corner_radius = CornerRadius::ZERO;
            }
        }
        Action::MenuRow => {
            if let Shape::Rect(rs) = shape {
                rs.fill = pal.sel;
                rs.stroke = Stroke::NONE;
                rs.corner_radius = CornerRadius::ZERO;
            }
        }
        Action::Tooltip => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                // The iconic "info" yellow in Silver; warm charcoal in Night.
                let fill = if pal.chiseled {
                    Color32::from_rgb(52, 52, 42)
                } else {
                    Color32::from_rgb(255, 255, 225)
                };
                let mut shapes = vec![Shape::rect_filled(rect, CornerRadius::ZERO, fill)];
                bevel::edge_shapes(rect, pal.dkshadow, pal.dkshadow, &mut shapes);
                *shape = Shape::Vec(shapes);
            }
        }
        Action::WhiteText => {
            if let Shape::Text(ts) = shape {
                ts.override_text_color = Some(pal.sel_text);
            }
        }
        Action::Radio => {
            if let Shape::Circle(cs) = shape {
                use std::f32::consts::PI;
                let (c, r) = (cs.center, cs.radius);
                // Radios never light up: normalize the sentinels.
                let fill = normalize_field(cs.fill, pal);
                let mut shapes = vec![Shape::circle_filled(c, r, fill)];
                let ring = |radius: f32, from: f32, to: f32, color: Color32| {
                    let points = (0..=8)
                        .map(|k| {
                            let t = from + (to - from) * k as f32 / 8.0;
                            pos2(c.x + radius * t.cos(), c.y + radius * t.sin())
                        })
                        .collect();
                    Shape::Path(egui::epaint::PathShape::line(points, Stroke::new(1.0, color)))
                };
                shapes.push(ring(r, PI * 0.75, PI * 1.75, pal.shadow));
                shapes.push(ring(r, -PI * 0.25, PI * 0.75, pal.hilight));
                shapes.push(ring(r - 1.0, PI * 0.75, PI * 1.75, pal.dkshadow));
                *shape = Shape::Vec(shapes);
            }
        }
        Action::Check(box_rect) => {
            // The classic 7-wide check, two strokes doubled for thickness,
            // scaled from the 13px reference box.
            let s = box_rect.width() / 13.0;
            let p = box_rect.min;
            let stroke = Stroke::new(1.0, pal.text);
            let seg = |x0: f32, y0: f32, x1: f32, y1: f32| Shape::LineSegment {
                points: [p + vec2(x0 * s, y0 * s), p + vec2(x1 * s, y1 * s)],
                stroke,
            };
            *shape = Shape::Vec(vec![
                seg(3.5, 6.5, 5.5, 8.5),
                seg(3.5, 7.5, 5.5, 9.5),
                seg(5.5, 8.5, 10.0, 4.0),
                seg(5.5, 9.5, 10.0, 5.0),
            ]);
        }
        Action::ComboField { pressed } => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                let mut shapes = bevel::shapes(bevel::Kind::Sunken, rect, pal, Some(pal.winbg));
                let kind = if *pressed { bevel::Kind::Pressed } else { bevel::Kind::Raised };
                shapes.extend(bevel::shapes(kind, combo_btn(rect), pal, Some(pal.face)));
                *shape = Shape::Vec(shapes);
            }
        }
        Action::ComboArrow { btn, pressed } => {
            let bounds = shape.visual_bounding_rect();
            let mut delta = btn.center() - bounds.center();
            if *pressed {
                delta += vec2(1.0, 1.0);
            }
            shape.translate(delta);
        }
        Action::Channel => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                *shape = Shape::Vec(bevel::shapes(bevel::Kind::Sunken, rect, pal, None));
            }
        }
        Action::Thumb { lit } => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                let fill = if *lit { pal.light } else { pal.face };
                *shape = Shape::Vec(bevel::shapes(bevel::Kind::Raised, rect, pal, Some(fill)));
            }
        }
        Action::Trough => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                let mut shapes = vec![Shape::rect_filled(rect, CornerRadius::ZERO, pal.face)];
                bevel::edge_shapes(rect, pal.shadow, pal.hilight, &mut shapes);
                *shape = Shape::Vec(shapes);
            }
        }
        Action::Blocks { trough } => {
            if let Shape::Rect(rs) = shape {
                let inner = trough.shrink(2.0);
                let filled = (rs.rect.width() - 2.0).clamp(0.0, inner.width());
                let block_w = (inner.height() * 0.66).round();
                let gap = 2.0;
                let mut shapes = Vec::new();
                let mut x = inner.min.x;
                while x + block_w <= inner.min.x + filled {
                    shapes.push(Shape::rect_filled(
                        Rect::from_min_max(pos2(x, inner.min.y), pos2(x + block_w, inner.max.y)),
                        CornerRadius::ZERO,
                        pal.sel,
                    ));
                    x += block_w + gap;
                }
                *shape = Shape::Vec(shapes);
            }
        }
        Action::Etch => {
            if let Shape::LineSegment { points, .. } = shape {
                let [a, b] = *points;
                // Perpendicular offset toward bottom/right, like DrawEdge's
                // etched grooves.
                let offset = if (b.y - a.y).abs() <= (b.x - a.x).abs() {
                    vec2(0.0, 1.0)
                } else {
                    vec2(1.0, 0.0)
                };
                *shape = Shape::Vec(vec![
                    Shape::LineSegment { points: [a, b], stroke: Stroke::new(1.0, pal.shadow) },
                    Shape::LineSegment {
                        points: [a + offset, b + offset],
                        stroke: Stroke::new(1.0, pal.hilight),
                    },
                ]);
            }
        }
        Action::EtchRect => {
            if let Shape::Rect(rs) = shape {
                let rect = rs.rect;
                let fill = rs.fill;
                let mut shapes = Vec::with_capacity(9);
                if fill != Color32::TRANSPARENT {
                    shapes.push(Shape::rect_filled(rect, CornerRadius::ZERO, fill));
                }
                bevel::edge_shapes(rect.translate(vec2(1.0, 1.0)), pal.hilight, pal.hilight, &mut shapes);
                bevel::edge_shapes(rect, pal.shadow, pal.shadow, &mut shapes);
                *shape = Shape::Vec(shapes);
            }
        }
        Action::Shift => shape.translate(vec2(1.0, 1.0)),
        Action::PressedText => {
            if let Shape::Text(ts) = shape {
                ts.override_text_color = Some(pal.text);
            }
            shape.translate(vec2(1.0, 1.0));
        }
        Action::InsetClip(rect) => {
            clipped.clip_rect = clipped.clip_rect.intersect(*rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::epaint::{RectShape, StrokeKind};

    fn clipped(shape: Shape) -> ClippedShape {
        ClippedShape {
            clip_rect: Rect::EVERYTHING,
            shape,
        }
    }

    fn selected_rect(stroke: Stroke) -> Shape {
        Shape::Rect(RectShape::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(70.0, 22.0)),
            CornerRadius::ZERO,
            crate::palette::SILVER.sel,
            stroke,
            StrokeKind::Inside,
        ))
    }

    #[test]
    fn selected_stock_button_becomes_persistently_pressed() {
        let entries = [clipped(selected_rect(Stroke::new(1.0, MARK_RAISED)))];
        let actions = classify(
            entries.iter(),
            &crate::palette::SILVER,
            None,
            egui::Order::Middle,
        );

        assert!(actions.iter().any(|(_, action)| matches!(
            action,
            Action::Bevel(bevel::Kind::Pressed, fill)
                if *fill == crate::palette::SILVER.face
        )));
    }

    #[test]
    fn selected_menu_row_stays_a_flat_selection() {
        let entries = [clipped(selected_rect(Stroke::NONE))];
        let actions = classify(
            entries.iter(),
            &crate::palette::SILVER,
            None,
            egui::Order::Foreground,
        );

        assert!(
            actions
                .iter()
                .any(|(_, action)| matches!(action, Action::Flatten))
        );
    }

    #[test]
    fn rounded_dark_canvas_becomes_a_clipped_sunken_well() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(160.0, 90.0));
        let entries = [
            clipped(Shape::rect_filled(
                rect,
                CornerRadius::same(4),
                Color32::from_black_alpha(140),
            )),
            clipped(Shape::line_segment(
                [rect.left_center(), rect.right_center()],
                Stroke::new(1.0, Color32::WHITE),
            )),
        ];
        let actions = classify(
            entries.iter(),
            &crate::palette::SILVER,
            None,
            egui::Order::Middle,
        );

        assert!(actions.iter().any(|(i, action)| {
            *i == 0 && matches!(action, Action::Bevel(bevel::Kind::Sunken, _))
        }));
        assert!(actions.iter().any(|(i, action)| {
            *i == 1 && matches!(action, Action::InsetClip(inner) if *inner == rect.shrink(2.0))
        }));
    }
}
