use crate::daemon::surface_bridge::SurfaceBridge;
use crate::engine::video_proc::VideoProcessor;
use gtk4::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TrimParameters {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub speed_multiplier: f64,
}

pub struct MediaTrimmerModal;

impl MediaTrimmerModal {
    pub fn show<F>(app: &gtk4::Application, files: Vec<PathBuf>, on_confirm: F)
    where
        F: Fn(Vec<PathBuf>, TrimParameters) + 'static,
    {
        let window = gtk4::Window::builder()
            .application(app)
            .title("Tangerine Media Trimmer & Speed")
            .build();
        SurfaceBridge::configure_floating_panel(&window, "Trim & Speed", 420, 310);
        window.add_css_class("citrus-modal-window");

        let default_dur = files
            .first()
            .and_then(|p| VideoProcessor::probe_duration_secs(p).ok())
            .unwrap_or(30.0);

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        root.set_margin_top(16);
        root.set_margin_bottom(16);
        root.set_margin_start(16);
        root.set_margin_end(16);

        let header = gtk4::Label::new(Some("Audio / Video Trimmer & Tempo"));
        header.add_css_class("citrus-header");
        header.set_halign(gtk4::Align::Start);
        root.append(&header);

        let start_lbl = gtk4::Label::new(Some("Start Timestamp (seconds):"));
        start_lbl.set_halign(gtk4::Align::Start);
        let start_scale =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, default_dur, 0.1);
        start_scale.set_draw_value(true);
        start_scale.set_value(0.0);

        let end_lbl = gtk4::Label::new(Some("End Timestamp (seconds):"));
        end_lbl.set_halign(gtk4::Align::Start);
        let end_scale =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.2, default_dur, 0.1);
        end_scale.set_draw_value(true);
        end_scale.set_value(default_dur);

        let speed_lbl = gtk4::Label::new(Some("Playback Speed Multiplier (0.5x - 2.0x):"));
        speed_lbl.set_halign(gtk4::Align::Start);
        let speed_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 2.0, 0.05);
        speed_scale.set_draw_value(true);
        speed_scale.set_value(1.0);

        root.append(&start_lbl);
        root.append(&start_scale);
        root.append(&end_lbl);
        root.append(&end_scale);
        root.append(&speed_lbl);
        root.append(&speed_scale);

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let cancel_btn = gtk4::Button::with_label("Cancel");
        cancel_btn.add_css_class("citrus-btn-secondary");
        let apply_btn = gtk4::Button::with_label("Process Media");
        apply_btn.add_css_class("citrus-btn-primary");
        apply_btn.set_hexpand(true);

        let win_c = window.clone();
        cancel_btn.connect_clicked(move |_| {
            win_c.close();
        });

        let win_a = window.clone();
        apply_btn.connect_clicked(move |_| {
            let start = start_scale.value();
            let end = end_scale.value().max(start + 0.2);
            let speed = speed_scale.value();
            on_confirm(
                files.clone(),
                TrimParameters {
                    start_seconds: start,
                    end_seconds: end,
                    speed_multiplier: speed,
                },
            );
            win_a.close();
        });

        btn_row.append(&cancel_btn);
        btn_row.append(&apply_btn);
        root.append(&btn_row);

        window.set_child(Some(&root));
        window.present();
    }
}
