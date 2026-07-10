//! The desktop background with selectable, double-clickable icons, ported
//! from BevelDesk's `shell/desktop.cpp`: teal in Silver, charcoal in NeXT
//! Night, icons in a left-edge column with navy label highlight on select.

use egui::{CornerRadius, FontId, Id, Rect, Sense, Ui, pos2, vec2};

use crate::icons::Icon;
use crate::palette::Palette;

/// Desktop selection state you own.
#[derive(Debug, Clone, Default)]
pub struct DesktopState {
    /// Index of the currently selected icon, if any.
    pub selected: Option<usize>,
}

/// A desktop icon: a label under one of the built-in pixel icons.
#[derive(Debug, Clone)]
pub struct DesktopIcon {
    pub label: String,
    pub icon: Icon,
}

impl DesktopIcon {
    pub fn new(label: impl Into<String>, icon: Icon) -> Self {
        Self {
            label: label.into(),
            icon,
        }
    }
}

/// What happened on the desktop this frame.
pub struct DesktopResponse {
    /// The desktop area — pass to [`crate::shell::Window95::constrain_to`]
    /// so windows stay off the taskbar and maximize into this rect.
    pub rect: Rect,
    /// Icon index that was double-clicked ("opened") this frame.
    pub opened: Option<usize>,
}

/// The desktop. Show it *after* the [`crate::shell::Taskbar`] — it fills all
/// remaining space with the scheme's desktop color.
#[must_use = "call .show() to display the desktop"]
pub struct Desktop<'a> {
    state: &'a mut DesktopState,
}

impl<'a> Desktop<'a> {
    pub fn new(state: &'a mut DesktopState) -> Self {
        Self { state }
    }

    pub fn show(self, ui: &mut Ui, icons: &[DesktopIcon]) -> DesktopResponse {
        let pal = Palette::current(ui.ctx());
        let mut opened = None;

        let inner = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(pal.desktop))
            .show(ui, |ui| {
                let rect = ui.max_rect();

                // Background click clears the selection (icons interact on
                // top of this and win the hit test).
                let bg = ui.interact(rect, Id::new("boomer-desktop-bg"), Sense::click());
                if bg.clicked() {
                    self.state.selected = None;
                }

                for (i, icon) in icons.iter().enumerate() {
                    let cell = Rect::from_min_size(
                        pos2(rect.min.x + 14.0, rect.min.y + 16.0 + i as f32 * 78.0),
                        vec2(72.0, 56.0),
                    );
                    let resp = ui.interact(cell, Id::new(("boomer-desktop-icon", i)), Sense::click());
                    if resp.clicked() {
                        self.state.selected = Some(i);
                    }
                    if resp.double_clicked() {
                        opened = Some(i);
                    }

                    let painter = ui.painter();
                    icon.icon.paint_large(
                        painter,
                        pos2(cell.min.x + (cell.width() - 32.0) / 2.0, cell.min.y),
                        pal,
                    );
                    let galley = painter.layout_no_wrap(
                        icon.label.clone(),
                        FontId::proportional(12.0),
                        pal.title_text,
                    );
                    let size = galley.size();
                    let tp = pos2(cell.min.x + (cell.width() - size.x) / 2.0, cell.min.y + 37.0);
                    if self.state.selected == Some(i) {
                        painter.rect_filled(
                            Rect::from_min_max(
                                tp - vec2(2.0, 1.0),
                                tp + vec2(size.x + 2.0, size.y + 1.0),
                            ),
                            CornerRadius::ZERO,
                            pal.sel,
                        );
                    }
                    painter.galley(tp, galley, pal.title_text);
                }

                rect
            });

        DesktopResponse {
            rect: inner.inner,
            opened,
        }
    }
}
