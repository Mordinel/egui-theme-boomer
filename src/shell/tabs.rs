//! `TabControl` — Win95 property-sheet tabs (Display Properties, Task
//! Manager): a strip of raised tabs with chamfered top corners over a raised
//! page. The selected tab is taller and two pixels wider on each side, and
//! its face merges seamlessly into the page below, interrupting the page's
//! top highlight — exactly like the original common control.
//!
//! egui has no stock tab widget, so like the taskbar and `Window95` this is
//! something you explicitly import and use.

use egui::{Align2, CornerRadius, FontId, Id, InnerResponse, Rect, Sense, Ui, pos2, vec2};

use crate::bevel;
use crate::icons::fill_px;
use crate::palette::Palette;

const TAB_H: f32 = 21.0;

/// A Win95 tab control: tab strip + raised page filling the remaining space.
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// # let mut selected = 0;
/// egui_theme_boomer::shell::TabControl::new("props", &["Background", "Screen Saver", "Appearance"])
///     .show(ui, &mut selected, |ui, tab| {
///         ui.label(format!("Contents of tab {tab}"));
///     });
/// # });
/// ```
#[must_use = "call .show() to display the tab control"]
pub struct TabControl<'a> {
    id: Id,
    labels: &'a [&'a str],
}

impl<'a> TabControl<'a> {
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, labels: &'a [&'a str]) -> Self {
        Self {
            id: Id::new(("boomer-tabs", id_salt)),
            labels,
        }
    }

    /// Show the tab strip and the page, which fills the remaining space.
    /// `selected` is clamped to the label count.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        selected: &mut usize,
        add_contents: impl FnOnce(&mut Ui, usize) -> R,
    ) -> InnerResponse<R> {
        let pal = Palette::current(ui.ctx());
        *selected = (*selected).min(self.labels.len().saturating_sub(1));

        let avail = ui.available_rect_before_wrap();
        let strip = Rect::from_min_size(avail.min, vec2(avail.width(), TAB_H));
        let page = Rect::from_min_max(pos2(avail.min.x, strip.max.y), avail.max);
        ui.allocate_rect(avail, Sense::hover());
        let painter = ui.painter().clone();

        // Tab geometry first (interaction), painting after the page so the
        // selected tab can overlap the page's top edge.
        let font = FontId::proportional(12.0);
        let mut tab_rects: Vec<Rect> = Vec::with_capacity(self.labels.len());
        let mut x = strip.min.x + 2.0;
        for label in self.labels {
            let width = painter
                .layout_no_wrap((*label).to_owned(), font.clone(), pal.text)
                .size()
                .x
                .ceil()
                + 13.0;
            tab_rects.push(Rect::from_min_size(
                pos2(x, strip.min.y + 2.0),
                vec2(width, TAB_H - 2.0),
            ));
            x += width;
        }
        let mut hovered_tab: Option<usize> = None;
        for (i, rect) in tab_rects.iter().enumerate() {
            // The selected tab grows 2px up and 2px to each side.
            let hit = if i == *selected { grow_selected(*rect) } else { *rect };
            let resp = ui.interact(hit, self.id.with(("tab", i)), Sense::click());
            if resp.clicked() {
                *selected = i;
            }
            if resp.hovered() {
                hovered_tab = Some(i);
            }
        }

        // ---- paint: page, then unselected tabs, then the selected tab ----
        bevel::paint(bevel::Kind::Raised, &painter, page, pal, Some(pal.face));

        for (i, rect) in tab_rects.iter().enumerate() {
            if i != *selected {
                draw_tab(&painter, *rect, pal, false, hovered_tab == Some(i));
            }
        }
        let sel_rect = grow_selected(tab_rects[*selected]);
        draw_tab(&painter, sel_rect, pal, true, false);

        // ---- content ----
        let client = page.shrink(10.0);
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(client)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        content_ui.set_clip_rect(client.intersect(content_ui.clip_rect()));
        let inner = add_contents(&mut content_ui, *selected);

        // Labels on top of everything (the selected tab's grew rect shifted
        // its label up by one pixel, like Win95).
        for (i, rect) in tab_rects.iter().enumerate() {
            let center = if i == *selected {
                rect.center() - vec2(0.0, 1.0)
            } else {
                rect.center()
            };
            painter.text(center, Align2::CENTER_CENTER, self.labels[i], font.clone(), pal.text);
        }

        InnerResponse::new(inner, ui.interact(avail, self.id, Sense::hover()))
    }
}

fn grow_selected(rect: Rect) -> Rect {
    Rect::from_min_max(
        pos2(rect.min.x - 2.0, rect.min.y - 2.0),
        pos2(rect.max.x + 2.0, rect.max.y + 2.0),
    )
}

/// One tab: face fill, chamfered top corners, highlight on the left/top,
/// dark on the right, open at the bottom. The selected tab's fill runs 2px
/// past the strip bottom, covering the page's top highlight beneath it.
fn draw_tab(painter: &egui::Painter, rect: Rect, pal: &Palette, selected: bool, hovered: bool) {
    let p = rect.min;
    let w = rect.width();
    let h = rect.height();
    let fill = if hovered { pal.light } else { pal.face };
    painter.rect_filled(
        Rect::from_min_max(pos2(rect.min.x + 1.0, rect.min.y + 1.0), rect.max),
        CornerRadius::ZERO,
        fill,
    );
    let (light_outer, dark_outer, dark_inner) = if pal.chiseled {
        (pal.dkshadow, pal.dkshadow, pal.shadow)
    } else {
        (pal.hilight, pal.dkshadow, pal.shadow)
    };
    // Left edge + chamfer + top edge (light from the top-left).
    fill_px(painter, p, 0.0, 2.0, 1.0, h, light_outer);
    fill_px(painter, p, 1.0, 1.0, 2.0, 2.0, light_outer);
    fill_px(painter, p, 2.0, 0.0, w - 2.0, 1.0, light_outer);
    if pal.chiseled {
        // NeXT: inner highlight under the keyline.
        fill_px(painter, p, 1.0, 2.0, 2.0, h, pal.hilight);
        fill_px(painter, p, 2.0, 1.0, w - 2.0, 2.0, pal.hilight);
    }
    // Right chamfer + edge (dark toward the bottom-right).
    fill_px(painter, p, w - 2.0, 1.0, w - 1.0, 2.0, dark_outer);
    fill_px(painter, p, w - 1.0, 2.0, w, h, dark_outer);
    fill_px(painter, p, w - 2.0, 2.0, w - 1.0, h, dark_inner);
    if selected {
        // Erase the page's top bevel underneath, merging tab and page.
        painter.rect_filled(
            Rect::from_min_max(
                pos2(rect.min.x + 1.0, rect.max.y - 1.0),
                pos2(rect.max.x - 2.0, rect.max.y + 2.0),
            ),
            CornerRadius::ZERO,
            pal.face,
        );
    }
}
