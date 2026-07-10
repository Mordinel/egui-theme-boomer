//! Opt-in retro desktop components: Win95 windows, the taskbar with Start
//! menu, and desktop icons.
//!
//! Unlike [`crate::install`] — which only restyles the widgets you already
//! have — these are things you explicitly import and use:
//!
//! ```no_run
//! use egui_theme_boomer::shell::{Desktop, DesktopIcon, Taskbar, TaskEntry, Window95};
//! # use egui_theme_boomer::icons::Icon;
//! # fn ui(ui: &mut egui::Ui, taskbar_state: &mut egui_theme_boomer::shell::TaskbarState,
//! #       desk: &mut egui_theme_boomer::shell::DesktopState,
//! #       win: &mut egui_theme_boomer::shell::WindowState) {
//! Taskbar::new(taskbar_state).show(ui, &mut [TaskEntry::new("My Computer", Icon::MyComputer, win)], |_menu| {});
//! let desktop = Desktop::new(desk).show(ui, &[DesktopIcon::new("My Computer", Icon::MyComputer)]);
//! Window95::new("My Computer")
//!     .constrain_to(desktop.rect)
//!     .show(ui.ctx(), win, |ui| { ui.label("hello"); });
//! # }
//! ```

mod desktop;
mod tabs;
mod taskbar;
mod window;

pub use desktop::{Desktop, DesktopIcon, DesktopResponse, DesktopState};
pub use tabs::TabControl;
pub use taskbar::{TaskEntry, Taskbar, TaskbarState, start_menu_item, start_menu_separator};
pub use window::{Window95, WindowState};

use std::sync::Arc;

use egui::{Color32, Context, FontFamily, FontId, Galley, Painter, Pos2, vec2};

/// Bold 12px chrome text (captions, Start button, task buttons).
///
/// Uses the real bold system font when [`crate::fonts::install_fonts`] found
/// one; otherwise fakes it with a 1px double-strike, like BevelDesk's
/// `AddTextBold` fallback.
pub(crate) struct BoldText {
    galley: Arc<Galley>,
    double_strike: bool,
}

impl BoldText {
    pub fn layout(ctx: &Context, painter: &Painter, text: &str, color: Color32) -> Self {
        let real = crate::fonts::has_real_bold(ctx);
        let family = if real {
            crate::fonts::bold_family()
        } else {
            FontFamily::Proportional
        };
        let galley = painter.layout_no_wrap(text.to_owned(), FontId::new(12.0, family), color);
        Self {
            galley,
            double_strike: !real,
        }
    }

    pub fn size(&self) -> egui::Vec2 {
        self.galley.size()
    }

    pub fn paint(&self, painter: &Painter, pos: Pos2) {
        painter.galley(pos, self.galley.clone(), Color32::PLACEHOLDER);
        if self.double_strike {
            painter.galley(pos + vec2(1.0, 0.0), self.galley.clone(), Color32::PLACEHOLDER);
        }
    }
}
