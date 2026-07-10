//! The full retro desktop: taskbar with Start menu, desktop icons, and Win95
//! windows — the opt-in components from `egui_theme_boomer::shell`.
//!
//! Run with: `cargo run --example desktop`

#[path = "common/mod.rs"]
mod common;

use eframe::egui;
use egui_theme_boomer::icons::Icon;
use egui_theme_boomer::shell::{
    Desktop, DesktopIcon, DesktopState, TaskEntry, Taskbar, TaskbarState, Window95, WindowState,
    start_menu_item, start_menu_separator,
};
use egui_theme_boomer::Scheme;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-theme-boomer desktop",
        options,
        Box::new(|cc| {
            common::theme_from_env(&cc.egui_ctx);
            let mut app = App::default();
            if app.shot.active() {
                // Open more windows so the screenshot shows inactive
                // captions, the menu bar, and the tab control.
                app.notepad.restore();
                app.display_props.restore();
            }
            Ok(Box::new(app))
        }),
    )
}

struct App {
    taskbar: TaskbarState,
    desktop: DesktopState,
    my_computer: WindowState,
    notepad: WindowState,
    display_props: WindowState,
    about: WindowState,
    notepad_text: String,
    display_tab: usize,
    wallpaper: usize,
    shot: common::Screenshotter,
}

impl Default for App {
    fn default() -> Self {
        Self {
            taskbar: TaskbarState::default(),
            desktop: DesktopState::default(),
            my_computer: WindowState::new(),
            notepad: WindowState::closed(),
            display_props: WindowState::closed(),
            about: WindowState::closed(),
            notepad_text: "Dear diary,\n\nToday I installed a new theme.\n".to_owned(),
            display_tab: 0,
            wallpaper: 0,
            shot: common::Screenshotter::default(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.shot.tick(&ctx);
        // Keep the taskbar clock ticking.
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // ---- taskbar (bottom panel, so show it first) ----
        let clock = chrono::Local::now().format("%-I:%M %p").to_string();
        let mut entries = [
            TaskEntry::new("My Computer", Icon::MyComputer, &mut self.my_computer),
            TaskEntry::new("Notepad", Icon::Document, &mut self.notepad),
            TaskEntry::new("Display Properties", Icon::MyComputer, &mut self.display_props),
        ];
        let mut open_notepad = false;
        let mut open_about = false;
        let mut open_display = false;
        Taskbar::new(&mut self.taskbar)
            .clock(clock)
            .show(ui, &mut entries, |menu| {
                if start_menu_item(menu, "Programs").clicked() {
                    open_notepad = true;
                }
                if start_menu_item(menu, "Documents").clicked() {
                    open_notepad = true;
                }
                if start_menu_item(menu, "Settings").clicked() {
                    open_display = true;
                }
                if start_menu_item(menu, "Help").clicked() {
                    open_about = true;
                }
                start_menu_separator(menu);
                if start_menu_item(menu, "Shut Down...").clicked() {
                    menu.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

        // ---- desktop with icons ----
        let icons = [
            DesktopIcon::new("My Computer", Icon::MyComputer),
            DesktopIcon::new("Notepad", Icon::Document),
            DesktopIcon::new("Recycle Bin", Icon::RecycleBin),
        ];
        let desktop = Desktop::new(&mut self.desktop).show(ui, &icons);
        match desktop.opened {
            Some(0) => self.my_computer.restore(),
            Some(1) => open_notepad = true,
            _ => {}
        }
        if open_notepad {
            self.notepad.restore();
        }
        if open_about {
            self.about.restore();
        }
        if open_display {
            self.display_props.restore();
        }

        // ---- windows ----
        let desk = desktop.rect;
        Window95::new("My Computer")
            .icon(Icon::MyComputer)
            .default_size(egui::vec2(430.0, 280.0))
            .default_pos(desk.min + egui::vec2(120.0, 40.0))
            .min_size(egui::vec2(300.0, 180.0))
            .constrain_to(desk)
            .show(&ctx, &mut self.my_computer, |ui| {
                let pal = egui_theme_boomer::Palette::current(ui.ctx());
                // Win95 status bar: a sunken well pinned to the bottom.
                egui::Panel::bottom("mycomputer-status")
                    .exact_size(22.0)
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(egui::Frame::new())
                    .show(ui, |ui| {
                        let well = ui.max_rect().shrink2(egui::vec2(1.0, 2.0));
                        egui_theme_boomer::bevel::thin_sunken(ui.painter(), well, pal);
                        ui.painter().text(
                            egui::pos2(well.min.x + 6.0, well.center().y),
                            egui::Align2::LEFT_CENTER,
                            "4 object(s)",
                            egui::FontId::proportional(12.0),
                            pal.text,
                        );
                    });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    for (icon, label) in [
                        (Icon::Folder, "Windows"),
                        (Icon::Folder, "Program Files"),
                        (Icon::Document, "README.TXT"),
                        (Icon::DosPrompt, "MS-DOS"),
                    ] {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(94.0, 56.0), egui::Sense::hover());
                        icon.paint_large(
                            ui.painter(),
                            egui::pos2(rect.center().x - 16.0, rect.min.y + 2.0),
                            pal,
                        );
                        ui.painter().text(
                            egui::pos2(rect.center().x, rect.min.y + 40.0),
                            egui::Align2::CENTER_TOP,
                            label,
                            egui::FontId::proportional(12.0),
                            pal.text,
                        );
                    }
                });
            });

        let mut close_notepad = false;
        let mut clear_notepad = false;
        Window95::new("Notepad")
            .icon(Icon::Document)
            .default_size(egui::vec2(360.0, 260.0))
            .default_pos(desk.min + egui::vec2(500.0, 60.0))
            .constrain_to(desk)
            .show(&ctx, &mut self.notepad, |ui| {
                // A stock egui menu bar — skinned into the Win95 one.
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            clear_notepad = true;
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            close_notepad = true;
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        let _ = ui.button("Cut");
                        let _ = ui.button("Copy");
                        let _ = ui.button("Paste");
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About Notepad").clicked() {
                            open_about = true;
                        }
                    });
                });
                // Notepad structure: one sunken white field filling the whole
                // client, with a frameless text edit scrolling inside it.
                let pal = egui_theme_boomer::Palette::current(ui.ctx());
                let field = ui.available_rect_before_wrap();
                egui_theme_boomer::bevel::sunken_field(ui.painter(), field, pal, Some(pal.winbg));
                let mut inner = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(field.shrink(3.0))
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(&mut inner, |ui| {
                        let width = ui.available_width();
                        ui.add(
                            egui::TextEdit::multiline(&mut self.notepad_text)
                                .frame(egui::Frame::new())
                                .desired_width(width),
                        );
                    });
            });
        if clear_notepad {
            self.notepad_text.clear();
        }
        if close_notepad {
            self.notepad.open = false;
        }
        if open_about {
            self.about.restore();
        }

        // ---- Display Properties: Win95 property-sheet tabs ----
        Window95::new("Display Properties")
            .icon(Icon::MyComputer)
            .dialog(egui::vec2(380.0, 320.0))
            .default_pos(desk.min + egui::vec2(70.0, 290.0))
            .constrain_to(desk)
            .show(&ctx, &mut self.display_props, |ui| {
                ui.add_space(6.0);
                let tabs = ["Background", "Appearance", "Settings"];
                egui_theme_boomer::shell::TabControl::new("display-tabs", &tabs).show(
                    ui,
                    &mut self.display_tab,
                    |ui, tab| match tab {
                        0 => {
                            ui.label("Wallpaper:");
                            egui::ComboBox::from_id_salt("wallpaper")
                                .selected_text(["(None)", "Clouds", "Setup"][self.wallpaper])
                                .show_ui(ui, |ui| {
                                    for (i, name) in
                                        ["(None)", "Clouds", "Setup"].iter().enumerate()
                                    {
                                        ui.selectable_value(&mut self.wallpaper, i, *name);
                                    }
                                });
                        }
                        1 => {
                            ui.label("Color scheme:");
                            if ui.button("Windows Standard (Silver)").clicked() {
                                egui_theme_boomer::apply(ui.ctx(), Scheme::Silver);
                            }
                            if ui.button("NeXT Night").clicked() {
                                egui_theme_boomer::apply(ui.ctx(), Scheme::NextNight);
                            }
                            if ui.button("Follow system").clicked() {
                                egui_theme_boomer::install(ui.ctx());
                            }
                        }
                        _ => {
                            ui.label("Desktop area:");
                            ui.label("More colors make things prettier,");
                            ui.label("but slower. 256 colors is a good choice.");
                        }
                    },
                );
            });

        let mut close_about = false;
        Window95::new("About")
            .dialog(egui::vec2(320.0, 140.0))
            .constrain_to(desk)
            .show(&ctx, &mut self.about, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.heading("egui-theme-boomer");
                    ui.label("Silver and NeXT Night, ported from BevelDesk.");
                    ui.hyperlink("https://github.com/marchildmann/BevelDesk");
                    ui.add_space(8.0);
                    if ui.button("OK").clicked() {
                        close_about = true;
                    }
                });
            });
        if close_about {
            self.about.open = false;
        }
    }
}
