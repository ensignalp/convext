use crate::daemon::surface_bridge::SurfaceBridge;
use gtk4::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct TargetSizeSpec {
    pub target_bytes: u64,
}

pub struct TargetSizeModal;

impl TargetSizeModal {
    pub fn show<F>(app: &gtk4::Application, files: Vec<PathBuf>, is_video: bool, on_confirm: F)
    where
        F: Fn(Vec<PathBuf>, TargetSizeSpec, bool) + 'static,
    {
        let window = gtk4::Window::builder()
            .application(app)
            .title("Tangerine Exact Target Size")
            .build();
        SurfaceBridge::configure_floating_panel(&window, "Target File Size", 360, 220);
        window.add_css_class("citrus-modal-window");

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        root.set_margin_top(16);
        root.set_margin_bottom(16);
        root.set_margin_start(16);
        root.set_margin_end(16);

        let title = gtk4::Label::new(Some(if is_video {
            "Target Video Size (Bitrate Solver)"
        } else {
            "Target Image Size (Binary Search)"
        }));
        title.add_css_class("citrus-header");
        title.set_halign(gtk4::Align::Start);
        root.append(&title);

        let input_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let entry = gtk4::Entry::new();
        entry.add_css_class("citrus-input");
        entry.set_text(if is_video { "8" } else { "500" });
        entry.set_hexpand(true);

        let unit_combo = gtk4::DropDown::from_strings(&["KB", "MB"]);
        unit_combo.set_selected(if is_video { 1 } else { 0 });

        input_row.append(&entry);
        input_row.append(&unit_combo);
        root.append(&input_row);

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let cancel_btn = gtk4::Button::with_label("Cancel");
        cancel_btn.add_css_class("citrus-btn-secondary");
        let run_btn = gtk4::Button::with_label("Compress to Target");
        run_btn.add_css_class("citrus-btn-primary");
        run_btn.set_hexpand(true);

        let win_c = window.clone();
        cancel_btn.connect_clicked(move |_| {
            win_c.close();
        });

        let win_r = window.clone();
        run_btn.connect_clicked(move |_| {
            let val: f64 = entry.text().trim().parse().unwrap_or(500.0);
            let mult: u64 = if unit_combo.selected() == 1 {
                1024 * 1024
            } else {
                1024
            };
            let bytes = ((val * mult as f64) as u64).max(4096);
            on_confirm(
                files.clone(),
                TargetSizeSpec {
                    target_bytes: bytes,
                },
                is_video,
            );
            win_r.close();
        });

        btn_row.append(&cancel_btn);
        btn_row.append(&run_btn);
        root.append(&btn_row);

        window.set_child(Some(&root));
        window.present();
    }
}
