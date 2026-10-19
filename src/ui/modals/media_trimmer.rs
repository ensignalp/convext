use crate::engine::video_proc::VideoProcessor;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
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
        let window = adw::Window::builder()
            .application(app)
            .title("Convext — Medya Kırpıcı & Hız")
            .default_width(460)
            .default_height(380)
            .build();

        let default_dur = files
            .first()
            .and_then(|p| VideoProcessor::probe_duration_secs(p).ok())
            .unwrap_or(30.0);

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new(
            "Medya Kırpıcı & Tempo",
            "Video / Ses Başlangıç, Bitiş ve Oynatma Hızı",
        );
        header_bar.set_title_widget(Some(&win_title));
        toolbar_view.add_top_bar(&header_bar);

        let page = adw::PreferencesPage::new();
        let group = adw::PreferencesGroup::builder()
            .title("Zaman Çizelgesi & Hız Ayarları")
            .description("Kırpılacak aralığı (saniye) ve oynatma hız çarpanını seçin")
            .build();

        let start_scale =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, default_dur, 0.1);
        start_scale.set_draw_value(true);
        start_scale.set_value(0.0);
        start_scale.set_hexpand(true);
        start_scale.set_width_request(180);

        let start_row = adw::ActionRow::builder()
            .title("Başlangıç Zamanı (sn)")
            .subtitle("Kırpmanın başlayacağı saniye")
            .build();
        start_row.add_suffix(&start_scale);

        let end_scale =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.2, default_dur, 0.1);
        end_scale.set_draw_value(true);
        end_scale.set_value(default_dur);
        end_scale.set_hexpand(true);
        end_scale.set_width_request(180);

        let end_row = adw::ActionRow::builder()
            .title("Bitiş Zamanı (sn)")
            .subtitle("Kırpmanın biteceği saniye")
            .build();
        end_row.add_suffix(&end_scale);

        let speed_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.5, 2.0, 0.05);
        speed_scale.set_draw_value(true);
        speed_scale.set_value(1.0);
        speed_scale.set_hexpand(true);
        speed_scale.set_width_request(180);

        let speed_row = adw::ActionRow::builder()
            .title("Oynatma Hızı (0.5x – 2.0x)")
            .subtitle("Ses tonunu bozmadan hızlandır veya yavaşlat")
            .build();
        speed_row.add_suffix(&speed_scale);

        group.add(&start_row);
        group.add(&end_row);
        group.add(&speed_row);
        page.add(&group);

        let action_group = adw::PreferencesGroup::new();
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        btn_row.set_halign(gtk4::Align::End);

        let cancel_btn = gtk4::Button::with_label("İptal");
        cancel_btn.add_css_class("pill");

        let apply_btn = gtk4::Button::with_label("Medyayı İşle");
        apply_btn.add_css_class("suggested-action");
        apply_btn.add_css_class("pill");

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
        action_group.add(&btn_row);
        page.add(&action_group);

        toolbar_view.set_content(Some(&page));
        window.set_content(Some(&toolbar_view));
        window.present();
    }
}
