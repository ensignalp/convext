use crate::error::ConvextResult;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BatchRenameSpec {
    pub pattern: String,
    pub start_index: usize,
}

pub struct BatchRenameModal;

impl BatchRenameModal {
    pub fn show<F>(app: &gtk4::Application, files: Vec<PathBuf>, on_confirm: F)
    where
        F: Fn(Vec<PathBuf>, BatchRenameSpec) + 'static,
    {
        let window = adw::Window::builder()
            .application(app)
            .title("Convext — Toplu Yeniden Adlandır")
            .default_width(440)
            .default_height(310)
            .build();

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new(
            "Toplu Yeniden Adlandır",
            "Şablon: {name}, {index}, {date}",
        );
        header_bar.set_title_widget(Some(&win_title));
        toolbar_view.add_top_bar(&header_bar);

        let page = adw::PreferencesPage::new();
        let group = adw::PreferencesGroup::builder()
            .title("Adlandırma Şablonu")
            .description("Kullanılabilir değişkenler: {name}, {index}, {date}")
            .build();

        let pattern_entry = adw::EntryRow::builder()
            .title("Şablon (Pattern)")
            .text("{name}_{index}_{date}")
            .build();

        let preview_row = adw::ActionRow::builder()
            .title("Önizleme (Preview)")
            .subtitle("file_01_2026-10-19.ext")
            .build();

        let first_sample = files.first().cloned();
        let prev_clone = preview_row.clone();
        pattern_entry.connect_changed(move |e| {
            let pat = e.text().to_string();
            if let Some(ref sample) = first_sample {
                let rendered = Self::format_filename(sample, &pat, 1);
                prev_clone.set_subtitle(&rendered);
            }
        });

        group.add(&pattern_entry);
        group.add(&preview_row);
        page.add(&group);

        let action_group = adw::PreferencesGroup::new();
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        btn_row.set_halign(gtk4::Align::End);

        let cancel_btn = gtk4::Button::with_label("İptal");
        cancel_btn.add_css_class("pill");

        let apply_btn = gtk4::Button::with_label("Tümünü Yeniden Adlandır");
        apply_btn.add_css_class("suggested-action");
        apply_btn.add_css_class("pill");

        let win_c = window.clone();
        cancel_btn.connect_clicked(move |_| {
            win_c.close();
        });

        let win_a = window.clone();
        apply_btn.connect_clicked(move |_| {
            let pat = pattern_entry.text().to_string();
            on_confirm(
                files.clone(),
                BatchRenameSpec {
                    pattern: pat,
                    start_index: 1,
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

    pub fn format_filename(original: &Path, pattern: &str, index: usize) -> String {
        let stem = original
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = original.extension().and_then(|e| e.to_str()).unwrap_or("");
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let idx_str = format!("{:02}", index);

        let base = pattern
            .replace("{name}", stem)
            .replace("{index}", &idx_str)
            .replace("{date}", &today);

        if ext.is_empty() {
            base
        } else {
            format!("{}.{}", base, ext)
        }
    }

    pub fn execute_batch_rename(
        files: &[PathBuf],
        spec: &BatchRenameSpec,
    ) -> ConvextResult<Vec<PathBuf>> {
        let mut out = Vec::new();
        for (i, path) in files.iter().enumerate() {
            let new_name = Self::format_filename(path, &spec.pattern, spec.start_index + i);
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let dest = parent.join(new_name);
            std::fs::rename(path, &dest)?;
            out.push(dest);
        }
        Ok(out)
    }
}
