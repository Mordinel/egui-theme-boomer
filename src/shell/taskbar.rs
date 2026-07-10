//! The Win95 taskbar and Start menu, ported from BevelDesk's
//! `shell/taskbar.cpp`: a 28px bottom panel with the beveled Start button,
//! per-window task buttons (pressed-in when active), a sunken clock well, and
//! the Start menu popup with its vertical navy brand band.

use egui::epaint::TextShape;
use egui::{
    Align2, CornerRadius, FontId, Id, Order, Rect, Sense, Ui, pos2, vec2,
};

use crate::bevel;
use crate::icons::{self, Icon};
use crate::palette::{NAVY, Palette, TASKBAR_H};
use crate::shell::{BoldText, WindowState};

/// State the taskbar keeps between frames (Start menu open/closed).
#[derive(Debug, Clone, Default)]
pub struct TaskbarState {
    pub start_open: bool,
    opened_now: bool,
}

/// One task button: a window title, its icon, and the window's state.
pub struct TaskEntry<'a> {
    pub title: &'a str,
    pub icon: Icon,
    pub state: &'a mut WindowState,
}

impl<'a> TaskEntry<'a> {
    pub fn new(title: &'a str, icon: Icon, state: &'a mut WindowState) -> Self {
        Self { title, icon, state }
    }
}

/// The taskbar. Show it before the [`crate::shell::Desktop`] so the desktop
/// fills the remaining space.
#[must_use = "call .show() to display the taskbar"]
pub struct Taskbar<'a> {
    state: &'a mut TaskbarState,
    brand: String,
    start_label: String,
    clock: Option<String>,
}

impl<'a> Taskbar<'a> {
    pub fn new(state: &'a mut TaskbarState) -> Self {
        Self {
            state,
            brand: "Boomer95".to_owned(),
            start_label: "Start".to_owned(),
            clock: None,
        }
    }

    /// Vertical text on the Start menu's navy band (default "Boomer95").
    pub fn brand(mut self, brand: impl Into<String>) -> Self {
        self.brand = brand.into();
        self
    }

    /// Start button label (default "Start").
    pub fn start_label(mut self, label: impl Into<String>) -> Self {
        self.start_label = label.into();
        self
    }

    /// Text for the sunken clock well, e.g. `"3:07 PM"`. Omit to hide the well.
    pub fn clock(mut self, text: impl Into<String>) -> Self {
        self.clock = Some(text.into());
        self
    }

    /// Show the taskbar as a bottom panel. `entries` become task buttons
    /// (closed windows are skipped); `start_menu` fills the Start menu — add
    /// items with [`start_menu_item`] and [`start_menu_separator`].
    pub fn show(
        self,
        ui: &mut Ui,
        entries: &mut [TaskEntry<'_>],
        start_menu: impl FnOnce(&mut Ui),
    ) {
        let pal = Palette::current(ui.ctx());
        let state = self.state;

        let panel = egui::Panel::bottom(Id::new("boomer-taskbar"))
            .exact_size(TASKBAR_H)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(pal.face))
            .show(ui, |ui| {
                let bar = ui.max_rect();
                let painter = ui.painter().clone();
                // 1px light top edge.
                painter.rect_filled(
                    Rect::from_min_max(bar.min, pos2(bar.max.x, bar.min.y + 1.0)),
                    CornerRadius::ZERO,
                    pal.hilight,
                );

                let btn_y = bar.min.y + 4.0;
                let btn_h = 22.0;

                // ---- Start button ----
                let sb = Rect::from_min_size(pos2(bar.min.x + 2.0, btn_y), vec2(64.0, btn_h));
                let resp = ui.interact(sb, Id::new("boomer-start-button"), Sense::click());
                let held = resp.is_pointer_button_down_on() && resp.hovered();
                let down = held || state.start_open;
                if down {
                    bevel::pressed(&painter, sb, pal, true);
                } else {
                    let fill = if resp.hovered() { pal.light } else { pal.face };
                    bevel::paint(bevel::Kind::Raised, &painter, sb, pal, Some(fill));
                }
                let o = if down { 1.0 } else { 0.0 };
                icons::logo_13(&painter, sb.min + vec2(4.0 + o, 4.0 + o), pal);
                let label = BoldText::layout(ui.ctx(), &painter, &self.start_label, pal.text);
                label.paint(
                    &painter,
                    pos2(sb.min.x + 23.0 + o, sb.min.y + (btn_h - label.size().y) / 2.0 + o),
                );
                state.opened_now = false;
                if resp.clicked() {
                    state.start_open = !state.start_open;
                    state.opened_now = state.start_open;
                }

                // ---- clock well (far right) ----
                let mut right_limit = bar.max.x - 4.0;
                if let Some(clock) = &self.clock {
                    let clock_w = 72.0;
                    let well = Rect::from_min_max(
                        pos2(bar.max.x - 4.0 - clock_w, btn_y),
                        pos2(bar.max.x - 4.0, btn_y + btn_h),
                    );
                    bevel::thin_sunken(&painter, well, pal);
                    painter.text(
                        well.center(),
                        Align2::CENTER_CENTER,
                        clock,
                        FontId::proportional(12.0),
                        pal.text,
                    );
                    right_limit = well.min.x;
                }

                // ---- task buttons ----
                let mut tx = sb.max.x + 8.0;
                let open: Vec<usize> = entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.state.open)
                    .map(|(i, _)| i)
                    .collect();
                let avail = right_limit - 8.0 - tx;
                if !open.is_empty() && avail > 40.0 {
                    let bw = (avail / open.len() as f32 - 3.0).clamp(40.0, 160.0);
                    for i in open {
                        let entry = &mut entries[i];
                        let rect = Rect::from_min_size(pos2(tx, btn_y), vec2(bw, btn_h));
                        let resp =
                            ui.interact(rect, Id::new(("boomer-task-button", i)), Sense::click());
                        let held = resp.is_pointer_button_down_on() && resp.hovered();
                        let active = entry.state.is_active() && !entry.state.minimized;
                        if active || held {
                            bevel::pressed(&painter, rect, pal, true);
                            if active && !held {
                                painter.rect_filled(rect.shrink(2.0), CornerRadius::ZERO, pal.light);
                            }
                        } else {
                            let fill = if resp.hovered() { pal.light } else { pal.face };
                            bevel::paint(bevel::Kind::Raised, &painter, rect, pal, Some(fill));
                        }
                        let o = if active || held { 1.0 } else { 0.0 };
                        entry.icon.paint_small(&painter, rect.min + vec2(3.0 + o, 4.0 + o), pal);
                        let title = BoldText::layout(ui.ctx(), &painter, entry.title, pal.text);
                        let clip = Rect::from_min_max(
                            pos2(rect.min.x + 20.0, rect.min.y),
                            pos2(rect.max.x - 4.0, rect.max.y),
                        );
                        title.paint(
                            &painter.with_clip_rect(clip),
                            pos2(
                                rect.min.x + 20.0 + o,
                                rect.min.y + (btn_h - title.size().y) / 2.0 + o,
                            ),
                        );
                        if resp.clicked() {
                            if entry.state.minimized {
                                entry.state.restore();
                            } else if active {
                                entry.state.minimize();
                            } else {
                                entry.state.focus();
                            }
                        }
                        tx += bw + 3.0;
                    }
                }

                bar
            });

        // ---- Start menu popup ----
        if state.start_open {
            let bar = panel.inner;
            show_start_menu(ui, state, bar, &self.brand, pal, start_menu);
        }
    }
}

fn show_start_menu(
    ui: &Ui,
    state: &mut TaskbarState,
    bar: Rect,
    brand: &str,
    pal: &'static Palette,
    add_items: impl FnOnce(&mut Ui),
) {
    const WIDTH: f32 = 176.0;
    const SIDE_W: f32 = 22.0;
    let ctx = ui.ctx().clone();

    let area = egui::Area::new(Id::new("boomer-start-menu"))
        .order(Order::Foreground)
        .fixed_pos(pos2(bar.min.x + 2.0, bar.min.y))
        .pivot(Align2::LEFT_BOTTOM)
        .show(&ctx, |ui| {
            // Face fill paints under the items; bevel + band go on top after.
            let frame = egui::Frame::new().fill(pal.face).inner_margin(egui::Margin {
                left: (3.0 + SIDE_W) as i8,
                right: 3,
                top: 3,
                bottom: 3,
            });
            frame.show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_width(WIDTH - 6.0 - SIDE_W);
                add_items(ui);
            });

            // Chrome over the final rect: bevel edges + navy brand band
            // (both live in the frame margins, clear of the items).
            let rect = ui.min_rect();
            let painter = ui.painter();
            bevel::raised(painter, rect, pal, false);
            let band = Rect::from_min_max(
                rect.min + vec2(3.0, 3.0),
                pos2(rect.min.x + 3.0 + SIDE_W, rect.max.y - 3.0),
            );
            painter.rect_filled(band, CornerRadius::ZERO, NAVY);
            // Vertical brand text reading bottom-to-top.
            let galley = painter.layout_no_wrap(
                brand.to_owned(),
                FontId::proportional(12.0),
                pal.title_text,
            );
            let size = galley.size();
            painter.add(TextShape {
                angle: -std::f32::consts::FRAC_PI_2,
                ..TextShape::new(
                    pos2(band.min.x + (SIDE_W - size.y) / 2.0, band.max.y - 6.0),
                    galley,
                    pal.title_text,
                )
            });
        });

    let clicked_any = ctx.input(|i| i.pointer.any_click());
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if !state.opened_now && (escape || (clicked_any && area.response.clicked_elsewhere())) {
        state.start_open = false;
    }
    // Any click on a menu item also closes the menu, like Win95.
    if !state.opened_now && clicked_any && area.response.contains_pointer() {
        state.start_open = false;
    }
}

/// A Start-menu row: 30px tall, navy highlight, white text on hover.
pub fn start_menu_item(ui: &mut Ui, label: &str) -> egui::Response {
    let pal = Palette::current(ui.ctx());
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let hovered = resp.hovered();
    let painter = ui.painter();
    if hovered {
        painter.rect_filled(rect, CornerRadius::ZERO, pal.sel);
    }
    painter.text(
        pos2(rect.min.x + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        if hovered { pal.sel_text } else { pal.text },
    );
    resp
}

/// A sunken separator groove between Start-menu items.
pub fn start_menu_separator(ui: &mut Ui) {
    let pal = Palette::current(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 7.0), Sense::hover());
    bevel::thin_sunken(
        ui.painter(),
        Rect::from_min_max(
            pos2(rect.min.x + 4.0, rect.center().y - 1.0),
            pos2(rect.max.x - 4.0, rect.center().y + 1.0),
        ),
        pal,
    );
}
