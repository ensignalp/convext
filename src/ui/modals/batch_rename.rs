use crate::daemon::surface_bridge::SurfaceBridge;
use crate::error::CitrusResult;
use gtk4::prelude::*;
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
        let window = gtk4::Window::builder()
            .application(app)
            .title("Tangerine Batch Renamer")
            .build();
        SurfaceBridge::configure_floating_panel(&window, "Batch Rename", 400, 240);
        window.add_css_class("citrus-modal-window");

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        root.set_margin_top(16);
        root.set_margin_bottom(16);
        root.set_margin_start(16);
        root.set_margin_end(16);

        let title = gtk4::Label::new(Some("Pattern Batch Renamer"));
        title.add_css_class("citrus-header");
        title.set_halign(gtk4::Align::Start);
        root.append(&title);

        let hint = gtk4::Label::new(Some("Tokens: {name}, {index}, {date}"));
        hint.add_css_class("citrus-Subheader");
        hint.set_halign(gtk4::Align::Start);
        root.append(&hint);

        let pattern_entry = gtk4::Entry::new();
        pattern_entry.add_css_class("citrus-input");
        pattern_entry.set_text("{name}_{index}_{date}");
        root.append(&pattern_entry);

        let preview_lbl = gtk4::Label::new(Some("Preview: file_01_2026-10-19.ext"));
        preview_lbl.add_css_class("citrus-Subheader");
        preview_lbl.set_halign(gtk4::Align::Start);
        root.append(&preview_lbl);

        let first_sample = files.first().cloned();
        let prev_clone = preview_lbl.clone();
        pattern_entry.connect_changed(move |e| {
            let pat = e.text().to_string();
            if let Some(ref sample) = first_sample {
                let rendered = Self::format_filename(sample, &pat, 1);
                prev_clone.set_text(&format!("Preview: {}", rendered));
            }
        });

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let cancel_btn = gtk4::Button::with_label("Cancel");
        cancel_btn.add_css_class("citrus-btn-secondary");
        let apply_btn = gtk4::Button::with_label("Rename All");
        apply_btn.add_css_class("citrus-btn-primary");
        apply_btn.set_hexpand(true);

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
        root.append(&btn_row);

        window.set_child(Some(&root));
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
    ) -> CitrusResult<Vec<PathBuf>> {
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
