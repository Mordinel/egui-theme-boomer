//! `Window95` — a floating window with authentic Win95 chrome, ported from
//! BevelDesk's `theme95/window.cpp`: 4px beveled sizing border, 18px caption
//! with a horizontal gradient (navy→blue active, grays inactive), a system
//! menu behind the caption icon, and [minimize][maximize/restore][close]
//! caption buttons.
//!
//! Geometry is owned, Win95-style: the window is a fixed canvas that never
//! resizes itself to content (overflow clips); only the user resizes it, by
//! dragging the 4px border or corners, and only moves it by its caption.

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{
    Align2, Context, CursorIcon, Id, InnerResponse, Key, Order, Painter, Pos2, Rect, Sense, Ui,
    Vec2, pos2, vec2,
};

use crate::bevel;
use crate::icons::Icon;
use crate::palette::{CAPBTN_H, CAPBTN_W, Palette, TITLEBAR_H, WIN_BORDER};
use crate::shell::BoldText;

/// Per-window state you own, connecting a [`Window95`] to the
/// [taskbar](crate::shell::Taskbar).
#[derive(Debug, Clone, Default)]
pub struct WindowState {
    /// Window exists. Set false by the close button.
    pub open: bool,
    /// Hidden, restorable from the taskbar.
    pub minimized: bool,
    /// Fills the whole constraint area.
    pub maximized: bool,
    /// Current top-left; set on first show, then only by dragging the caption.
    pos: Option<Pos2>,
    /// Current size; set on first show, then only by border drags.
    size: Option<Vec2>,
    /// Geometry to restore when un-maximizing.
    restore_rect: Option<Rect>,
    request_focus: bool,
    active: bool,
    sysmenu_open: bool,
    sysmenu_opened_now: bool,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            open: true,
            ..Self::default()
        }
    }

    /// A window that starts closed (open it later, e.g. from a menu).
    pub fn closed() -> Self {
        Self::default()
    }

    /// Re-open / un-minimize and bring to front.
    pub fn restore(&mut self) {
        self.open = true;
        self.minimized = false;
        self.request_focus = true;
    }

    pub fn minimize(&mut self) {
        self.minimized = true;
    }

    /// Bring to front next frame.
    pub fn focus(&mut self) {
        self.request_focus = true;
    }

    /// Was this the frontmost window last time it was shown?
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// A Win95 window. Show your content inside; chrome is drawn for you.
///
/// Windows move by their caption, resize by their beveled border (never by
/// their content), minimize to a [`crate::shell::Taskbar`], maximize on
/// caption double-click, and carry the full system menu on the caption icon —
/// double-clicking the icon closes the window, like the original.
#[must_use = "call .show() to display the window"]
pub struct Window95 {
    title: String,
    id: Id,
    icon: Icon,
    dialog: bool,
    resizable: bool,
    minimizable: bool,
    maximizable: bool,
    default_size: Vec2,
    default_pos: Option<Pos2>,
    min_size: Vec2,
    constrain_rect: Option<Rect>,
}

impl Window95 {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            id: Id::new(("boomer-window95", &title)),
            title,
            icon: Icon::MyComputer,
            dialog: false,
            resizable: true,
            minimizable: true,
            maximizable: true,
            default_size: vec2(420.0, 300.0),
            default_pos: None,
            min_size: vec2(160.0, 96.0),
            constrain_rect: None,
        }
    }

    /// Explicit id, needed when two windows share a title.
    pub fn id(mut self, id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Id::new(id);
        self
    }

    /// Caption mini-icon (default: [`Icon::MyComputer`]).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    /// Fixed-size centered dialog: no caption icon, no minimize/maximize,
    /// not resizable — only a close button. Like BevelDesk's `CenteredDialog`.
    pub fn dialog(mut self, size: Vec2) -> Self {
        self.dialog = true;
        self.resizable = false;
        self.minimizable = false;
        self.maximizable = false;
        self.default_size = size;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn minimizable(mut self, minimizable: bool) -> Self {
        self.minimizable = minimizable;
        self
    }

    pub fn maximizable(mut self, maximizable: bool) -> Self {
        self.maximizable = maximizable;
        self
    }

    pub fn default_size(mut self, size: impl Into<Vec2>) -> Self {
        self.default_size = size.into();
        self
    }

    pub fn default_pos(mut self, pos: impl Into<Pos2>) -> Self {
        self.default_pos = Some(pos.into());
        self
    }

    /// Smallest size border-resizing allows (default 112×64).
    pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
        self.min_size = min_size.into();
        self
    }

    /// Keep the window inside this rect; also the rect maximizing fills.
    /// Typically [`crate::shell::DesktopResponse::rect`]. Defaults to the
    /// whole content area.
    pub fn constrain_to(mut self, rect: Rect) -> Self {
        self.constrain_rect = Some(rect);
        self
    }

    /// Show the window (unless closed or minimized).
    pub fn show<R>(
        self,
        ctx: &Context,
        state: &mut WindowState,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        if !state.open || state.minimized {
            state.active = false;
            state.sysmenu_open = false;
            return None;
        }

        let pal = Palette::current(ctx);
        let constrain = self.constrain_rect.unwrap_or_else(|| ctx.content_rect());
        let min_size = self.min_size.max(vec2(90.0, WIN_BORDER * 2.0 + TITLEBAR_H + 9.0));

        // ---- resolve geometry (ours, not egui's) ----
        let size = *state.size.get_or_insert(self.default_size);
        let pos = *state.pos.get_or_insert_with(|| {
            self.default_pos
                .unwrap_or_else(|| constrain.center() - size / 2.0)
        });
        let mut rect = if state.maximized {
            constrain
        } else {
            clamp_to(Rect::from_min_size(pos, size.max(min_size)), constrain)
        };
        rect = round_rect(rect);

        let area = egui::Area::new(self.id)
            .order(Order::Middle)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                // Reserve the window's footprint in the area.
                let (_, _) = ui.allocate_exact_size(rect.size(), Sense::hover());

                // ---- interactions first (they use last frame's rect and
                // adjust this frame's), then painting at the final rect ----
                if !state.maximized {
                    if self.resizable && !self.dialog {
                        rect = self.border_resize(ui, rect, constrain, min_size);
                    }
                    rect = round_rect(clamp_to(rect, constrain));
                }
                let moved = self.caption_interact(ui, rect, state);
                if moved != Vec2::ZERO && !state.maximized {
                    rect = round_rect(clamp_to(rect.translate(moved), constrain));
                }
                state.pos = Some(rect.min);
                if !state.maximized {
                    state.size = Some(rect.size());
                }

                // ---- chrome ----
                let painter = ui.ctx().layer_painter(ui.layer_id());
                // Hard offset shadow, painted first so the window covers it.
                painter.rect_filled(
                    rect.translate(vec2(2.0, 2.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_black_alpha(96),
                );
                bevel::window_frame(&painter, rect, pal, true);
                self.caption_paint(ui, &painter, rect, state, pal);

                // ---- content: a fixed, clipped canvas ----
                let client = client_rect(rect);
                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(client)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                content_ui.set_clip_rect(client.intersect(content_ui.clip_rect()));
                add_contents(&mut content_ui)
            });

        let layer = area.response.layer_id;
        if state.request_focus {
            ctx.move_to_top(layer);
            state.request_focus = false;
        }
        // Win95 focuses on mouse-down anywhere in the window — but only when
        // the press actually lands on this window (not on something above it).
        let press_pos = ctx.input(|i| {
            i.pointer
                .primary_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        });
        if let Some(pos) = press_pos
            && rect.contains(pos)
            && ctx.layer_id_at(pos) == Some(layer)
        {
            ctx.move_to_top(layer);
        }
        state.active = ctx.top_layer_id() == Some(layer) || state.sysmenu_open;

        Some(InnerResponse::new(area.inner, area.response))
    }

    /// The caption row: gradient, icon, title, buttons, system menu, and the
    /// move-drag / double-click-maximize zone. Painting happens in
    /// [`Self::caption_paint`]; this handles interactions and returns the
    /// move delta.
    fn caption_interact(&self, ui: &mut Ui, rect: Rect, state: &mut WindowState) -> Vec2 {
        let cap = caption_rect(rect);
        let n_buttons = if self.dialog { 1.0 } else { 3.0 };
        let btn_row_x = cap.max.x - 2.0 - CAPBTN_W * n_buttons - 2.0;

        // System-menu box: click opens the menu, double-click closes.
        state.sysmenu_opened_now = false;
        if !self.dialog {
            let box_rect = Rect::from_min_size(cap.min, vec2(18.0, TITLEBAR_H));
            let resp = ui.interact(box_rect, self.id.with("sysmenu-box"), Sense::click());
            if resp.double_clicked() {
                state.open = false;
                state.sysmenu_open = false;
            } else if resp.clicked() {
                state.sysmenu_open = !state.sysmenu_open;
                state.sysmenu_opened_now = state.sysmenu_open;
            }
        }

        // Drag zone: move the window; double-click toggles maximize.
        let drag_x = if self.dialog { cap.min.x } else { cap.min.x + 18.0 };
        let mut moved = Vec2::ZERO;
        if btn_row_x - drag_x > 8.0 {
            let zone = Rect::from_min_max(pos2(drag_x, cap.min.y), pos2(btn_row_x, cap.max.y));
            let resp = ui.interact(zone, self.id.with("caption"), Sense::click_and_drag());
            if resp.double_clicked() && self.maximizable && !self.dialog {
                toggle_maximize(state, rect);
            } else if resp.dragged() && !state.maximized {
                moved = resp.drag_delta();
            }
        }
        moved
    }

    fn caption_paint(
        &self,
        ui: &mut Ui,
        painter: &Painter,
        rect: Rect,
        state: &mut WindowState,
        pal: &'static Palette,
    ) {
        let cap = caption_rect(rect);
        let n_buttons = if self.dialog { 1.0 } else { 3.0 };
        let btn_row_x = cap.max.x - 2.0 - CAPBTN_W * n_buttons - 2.0;

        // Caption reads active while its system menu is open, like Win95.
        let active = state.active || state.sysmenu_open;
        let (g0, g1) = if active {
            (pal.title_act, pal.title_act2)
        } else {
            (pal.title_inact, pal.title_inact2)
        };
        // Horizontal gradient, dark at the left.
        let mut mesh = Mesh::default();
        mesh.vertices.extend([
            Vertex { pos: cap.left_top(), uv: WHITE_UV, color: g0 },
            Vertex { pos: cap.right_top(), uv: WHITE_UV, color: g1 },
            Vertex { pos: cap.right_bottom(), uv: WHITE_UV, color: g1 },
            Vertex { pos: cap.left_bottom(), uv: WHITE_UV, color: g0 },
        ]);
        mesh.indices.extend([0, 1, 2, 0, 2, 3]);
        painter.add(mesh);

        // Caption icon + bold title, clipped before the buttons.
        let mut text_x = cap.min.x + 4.0;
        if !self.dialog {
            self.icon.paint_small(painter, cap.min + vec2(2.0, 2.0), pal);
            text_x = cap.min.x + 20.0;
        }
        let title = BoldText::layout(ui.ctx(), painter, &self.title, pal.title_text);
        let clip = Rect::from_min_max(pos2(text_x, cap.min.y), pos2(btn_row_x - 2.0, cap.max.y));
        title.paint(
            &painter.with_clip_rect(clip),
            pos2(text_x, cap.min.y + (TITLEBAR_H - title.size().y) / 2.0),
        );

        // Caption buttons, inset 2px from top/right: [min][max] gap [close].
        let by = cap.min.y + 2.0;
        let close_p = pos2(cap.max.x - 2.0 - CAPBTN_W, by);
        if !self.dialog {
            let max_p = pos2(close_p.x - 2.0 - CAPBTN_W, by);
            let min_p = pos2(if self.maximizable { max_p.x - CAPBTN_W } else { max_p.x }, by);
            if self.minimizable
                && caption_button(ui, painter, min_p, Glyph::Min, self.id.with("cap-min"), pal)
            {
                state.minimized = true;
                state.sysmenu_open = false;
            }
            if self.maximizable {
                let glyph = if state.maximized { Glyph::Restore } else { Glyph::Max };
                if caption_button(ui, painter, max_p, glyph, self.id.with("cap-max"), pal) {
                    toggle_maximize(state, rect);
                }
            }
        }
        if caption_button(ui, painter, close_p, Glyph::Close, self.id.with("cap-close"), pal) {
            state.open = false;
            state.sysmenu_open = false;
        }

        if state.sysmenu_open {
            self.system_menu(ui.ctx(), state, cap, rect, pal);
        }
    }

    /// Win95 border resizing: 4px edges + corner zones, clamped to the
    /// minimum size and the constraint rect. Dragging one edge never moves
    /// the opposite edge.
    fn border_resize(&self, ui: &mut Ui, rect: Rect, constrain: Rect, min_size: Vec2) -> Rect {
        const B: f32 = WIN_BORDER;
        const CORNER: f32 = 14.0;
        let mut min = rect.min;
        let mut max = rect.max;

        struct Zone {
            key: &'static str,
            rect: Rect,
            cursor: CursorIcon,
            edges: (bool, bool, bool, bool), // left, top, right, bottom
        }
        let zones = [
            Zone {
                key: "rz-left",
                rect: Rect::from_min_max(pos2(rect.min.x, rect.min.y + CORNER), pos2(rect.min.x + B, rect.max.y - CORNER)),
                cursor: CursorIcon::ResizeHorizontal,
                edges: (true, false, false, false),
            },
            Zone {
                key: "rz-right",
                rect: Rect::from_min_max(pos2(rect.max.x - B, rect.min.y + CORNER), pos2(rect.max.x, rect.max.y - CORNER)),
                cursor: CursorIcon::ResizeHorizontal,
                edges: (false, false, true, false),
            },
            Zone {
                key: "rz-top",
                rect: Rect::from_min_max(pos2(rect.min.x + CORNER, rect.min.y), pos2(rect.max.x - CORNER, rect.min.y + B)),
                cursor: CursorIcon::ResizeVertical,
                edges: (false, true, false, false),
            },
            Zone {
                key: "rz-bottom",
                rect: Rect::from_min_max(pos2(rect.min.x + CORNER, rect.max.y - B), pos2(rect.max.x - CORNER, rect.max.y)),
                cursor: CursorIcon::ResizeVertical,
                edges: (false, false, false, true),
            },
            // Corner zones: only the two border arms of each corner (an "L"),
            // like Win95 — never the caption or client area, so the close
            // button next to the top-right corner stays clickable.
            Zone {
                key: "rz-tl-h",
                rect: Rect::from_min_size(rect.min, vec2(CORNER, B)),
                cursor: CursorIcon::ResizeNwSe,
                edges: (true, true, false, false),
            },
            Zone {
                key: "rz-tl-v",
                rect: Rect::from_min_size(rect.min, vec2(B, CORNER)),
                cursor: CursorIcon::ResizeNwSe,
                edges: (true, true, false, false),
            },
            Zone {
                key: "rz-tr-h",
                rect: Rect::from_min_size(pos2(rect.max.x - CORNER, rect.min.y), vec2(CORNER, B)),
                cursor: CursorIcon::ResizeNeSw,
                edges: (false, true, true, false),
            },
            Zone {
                key: "rz-tr-v",
                rect: Rect::from_min_size(pos2(rect.max.x - B, rect.min.y), vec2(B, CORNER)),
                cursor: CursorIcon::ResizeNeSw,
                edges: (false, true, true, false),
            },
            Zone {
                key: "rz-bl-h",
                rect: Rect::from_min_size(pos2(rect.min.x, rect.max.y - B), vec2(CORNER, B)),
                cursor: CursorIcon::ResizeNeSw,
                edges: (true, false, false, true),
            },
            Zone {
                key: "rz-bl-v",
                rect: Rect::from_min_size(pos2(rect.min.x, rect.max.y - CORNER), vec2(B, CORNER)),
                cursor: CursorIcon::ResizeNeSw,
                edges: (true, false, false, true),
            },
            Zone {
                key: "rz-br-h",
                rect: Rect::from_min_size(pos2(rect.max.x - CORNER, rect.max.y - B), vec2(CORNER, B)),
                cursor: CursorIcon::ResizeNwSe,
                edges: (false, false, true, true),
            },
            Zone {
                key: "rz-br-v",
                rect: Rect::from_min_size(pos2(rect.max.x - B, rect.max.y - CORNER), vec2(B, CORNER)),
                cursor: CursorIcon::ResizeNwSe,
                edges: (false, false, true, true),
            },
        ];

        for zone in zones {
            let resp = ui.interact(zone.rect, self.id.with(zone.key), Sense::drag());
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(zone.cursor);
            }
            if resp.dragged() {
                let d = resp.drag_delta();
                let (l, t, r, b) = zone.edges;
                if l {
                    min.x = (min.x + d.x).clamp(constrain.min.x, max.x - min_size.x);
                }
                if r {
                    max.x = (max.x + d.x).clamp(min.x + min_size.x, constrain.max.x);
                }
                if t {
                    min.y = (min.y + d.y).clamp(constrain.min.y, max.y - min_size.y);
                }
                if b {
                    max.y = (max.y + d.y).clamp(min.y + min_size.y, constrain.max.y);
                }
            }
        }
        Rect::from_min_max(min, max)
    }

    /// The Restore/Move/Size/Minimize/Maximize/Close popup under the caption icon.
    fn system_menu(
        &self,
        ctx: &Context,
        state: &mut WindowState,
        cap: Rect,
        outer: Rect,
        pal: &'static Palette,
    ) {
        struct Item {
            label: &'static str,
            shortcut: Option<&'static str>,
            enabled: bool,
            action: u8,
            sep_before: bool,
        }
        let items = [
            Item { label: "Restore", shortcut: None, enabled: state.maximized, action: b'r', sep_before: false },
            Item { label: "Move", shortcut: None, enabled: false, action: 0, sep_before: false },
            Item { label: "Size", shortcut: None, enabled: false, action: 0, sep_before: false },
            Item { label: "Minimize", shortcut: None, enabled: self.minimizable, action: b'n', sep_before: false },
            Item { label: "Maximize", shortcut: None, enabled: self.maximizable && !state.maximized, action: b'x', sep_before: false },
            Item { label: "Close", shortcut: Some("Alt+F4"), enabled: true, action: b'c', sep_before: true },
        ];
        const MW: f32 = 150.0;
        const IH: f32 = 18.0;
        let n_seps = items.iter().filter(|i| i.sep_before).count();
        let mh = 6.0 + items.len() as f32 * IH + n_seps as f32 * 5.0;

        let area = egui::Area::new(self.id.with("sysmenu"))
            .order(Order::Foreground)
            .fixed_pos(pos2(cap.min.x, cap.max.y))
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(MW, mh), Sense::hover());
                let painter = ui.painter();
                bevel::raised(painter, rect, pal, true);

                let mut y = rect.min.y + 3.0;
                for item in &items {
                    if item.sep_before {
                        bevel::thin_sunken(
                            painter,
                            Rect::from_min_max(
                                pos2(rect.min.x + 4.0, y + 1.0),
                                pos2(rect.max.x - 4.0, y + 3.0),
                            ),
                            pal,
                        );
                        y += 5.0;
                    }
                    let row =
                        Rect::from_min_max(pos2(rect.min.x + 3.0, y), pos2(rect.max.x - 3.0, y + IH));
                    let resp = ui.interact(row, self.id.with(("sysmenu-item", item.label)), Sense::click());
                    let hovered = item.enabled && resp.hovered();
                    if hovered {
                        painter.rect_filled(row, egui::CornerRadius::ZERO, pal.sel);
                    }
                    let font = egui::FontId::new(12.0, egui::FontFamily::Proportional);
                    let ty = pos2(row.min.x + 8.0, y + IH / 2.0);
                    if item.enabled {
                        let color = if hovered { pal.sel_text } else { pal.text };
                        painter.text(ty, Align2::LEFT_CENTER, item.label, font.clone(), color);
                    } else {
                        // Win95 disabled text: gray with a white emboss.
                        painter.text(
                            ty + vec2(1.0, 1.0),
                            Align2::LEFT_CENTER,
                            item.label,
                            font.clone(),
                            pal.hilight,
                        );
                        painter.text(ty, Align2::LEFT_CENTER, item.label, font.clone(), pal.graytext);
                    }
                    if let Some(shortcut) = item.shortcut {
                        let color = if !item.enabled {
                            pal.graytext
                        } else if hovered {
                            pal.sel_text
                        } else {
                            pal.text
                        };
                        painter.text(
                            pos2(row.max.x - 8.0, y + IH / 2.0),
                            Align2::RIGHT_CENTER,
                            shortcut,
                            font,
                            color,
                        );
                    }
                    if resp.clicked() && item.enabled {
                        match item.action {
                            b'r' | b'x' => toggle_maximize(state, outer),
                            b'n' => state.minimized = true,
                            b'c' => state.open = false,
                            _ => {}
                        }
                        state.sysmenu_open = false;
                    }
                    y += IH;
                }
            });

        if !state.sysmenu_opened_now
            && (area.response.clicked_elsewhere() || ctx.input(|i| i.key_pressed(Key::Escape)))
        {
            state.sysmenu_open = false;
        }
    }
}

fn caption_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        rect.min + vec2(WIN_BORDER, WIN_BORDER),
        pos2(rect.max.x - WIN_BORDER, rect.min.y + WIN_BORDER + TITLEBAR_H),
    )
}

fn client_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        rect.min + vec2(WIN_BORDER, WIN_BORDER + TITLEBAR_H + 1.0),
        rect.max - vec2(WIN_BORDER, WIN_BORDER),
    )
}

fn clamp_to(rect: Rect, bounds: Rect) -> Rect {
    let size = rect.size().min(bounds.size());
    let mut min = rect.min;
    min.x = min.x.clamp(bounds.min.x, bounds.max.x - size.x);
    min.y = min.y.clamp(bounds.min.y, bounds.max.y - size.y);
    Rect::from_min_size(min, size)
}

fn round_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        pos2(rect.min.x.round(), rect.min.y.round()),
        pos2(rect.max.x.round(), rect.max.y.round()),
    )
}

fn toggle_maximize(state: &mut WindowState, current: Rect) {
    if state.maximized {
        state.maximized = false;
        if let Some(rect) = state.restore_rect {
            state.pos = Some(rect.min);
            state.size = Some(rect.size());
        }
    } else {
        state.restore_rect = Some(current);
        state.maximized = true;
    }
}

#[derive(Clone, Copy)]
enum Glyph {
    Close,
    Min,
    Max,
    Restore,
}

fn caption_button(
    ui: &Ui,
    painter: &Painter,
    pos: Pos2,
    glyph: Glyph,
    id: Id,
    pal: &Palette,
) -> bool {
    let rect = Rect::from_min_size(pos, vec2(CAPBTN_W, CAPBTN_H));
    let resp = ui.interact(rect, id, Sense::click());
    let held = resp.is_pointer_button_down_on() && resp.hovered();
    if held {
        bevel::pressed(painter, rect, pal, true);
    } else {
        let fill = if resp.hovered() { pal.light } else { pal.face };
        bevel::paint(bevel::Kind::Raised, painter, rect, pal, Some(fill));
    }
    let off = if held { 1.0 } else { 0.0 };
    caption_glyph(painter, pos + vec2(off, off), glyph, pal);
    resp.clicked()
}

fn caption_glyph(painter: &Painter, p: Pos2, glyph: Glyph, pal: &Palette) {
    use crate::icons::{fill_px, line_px, outline_px};
    use crate::palette::BLACK;
    match glyph {
        Glyph::Close => {
            // 2px-thick X
            line_px(painter, p, 4.0, 3.0, 10.0, 9.0, BLACK);
            line_px(painter, p, 5.0, 3.0, 11.0, 9.0, BLACK);
            line_px(painter, p, 10.0, 3.0, 4.0, 9.0, BLACK);
            line_px(painter, p, 11.0, 3.0, 5.0, 9.0, BLACK);
        }
        Glyph::Min => {
            fill_px(painter, p, 4.0, 9.0, 10.0, 11.0, BLACK);
        }
        Glyph::Max => {
            outline_px(painter, p, 3.0, 2.0, 12.0, 11.0, BLACK);
            fill_px(painter, p, 3.0, 2.0, 12.0, 4.0, BLACK);
        }
        Glyph::Restore => {
            outline_px(painter, p, 5.0, 2.0, 12.0, 8.0, BLACK);
            fill_px(painter, p, 5.0, 2.0, 12.0, 4.0, BLACK);
            fill_px(painter, p, 3.0, 5.0, 10.0, 7.0, pal.face);
            outline_px(painter, p, 3.0, 5.0, 10.0, 11.0, BLACK);
            fill_px(painter, p, 3.0, 5.0, 10.0, 7.0, BLACK);
        }
    }
}
