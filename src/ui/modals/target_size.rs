use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
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
        let window = adw::Window::builder()
            .application(app)
            .title("Convext — Hedef Dosya Boyutu")
            .default_width(420)
            .default_height(280)
            .build();

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let subtitle = if is_video {
            "Bitrate Solver (Video)"
        } else {
            "Binary Search (Görsel)"
        };
        let win_title = adw::WindowTitle::new("Hedef Boyuta Sıkıştır", subtitle);
        header_bar.set_title_widget(Some(&win_title));
        toolbar_view.add_top_bar(&header_bar);

        let page = adw::PreferencesPage::new();
        let group = adw::PreferencesGroup::builder()
            .title("Hedef Boyut Ayarları")
            .description("Dosyanın sıkıştırılacağı maksimum boyutu ve birimi seçin")
            .build();

        let entry_row = adw::EntryRow::builder()
            .title("Hedef Miktar")
            .text(if is_video { "8" } else { "500" })
            .build();

        let unit_combo = adw::ComboRow::builder()
            .title("Birim (Unit)")
            .model(&gtk4::StringList::new(&["KB (Kilobayt)", "MB (Megabayt)"]))
            .selected(if is_video { 1 } else { 0 })
            .build();

        group.add(&entry_row);
        group.add(&unit_combo);
        page.add(&group);

        let action_group = adw::PreferencesGroup::new();
        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        btn_box.set_halign(gtk4::Align::End);

        let cancel_btn = gtk4::Button::with_label("İptal");
        cancel_btn.add_css_class("pill");

        let run_btn = gtk4::Button::with_label("Hedef Boyuta Sıkıştır");
        run_btn.add_css_class("suggested-action");
        run_btn.add_css_class("pill");

        let win_c = window.clone();
        cancel_btn.connect_clicked(move |_| {
            win_c.close();
        });

        let win_r = window.clone();
        run_btn.connect_clicked(move |_| {
            let val: f64 = entry_row.text().trim().parse().unwrap_or(500.0);
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

        btn_box.append(&cancel_btn);
        btn_box.append(&run_btn);
        action_group.add(&btn_box);
        page.add(&action_group);

        toolbar_view.set_content(Some(&page));
        window.set_content(Some(&toolbar_view));
        window.present();
    }
}
