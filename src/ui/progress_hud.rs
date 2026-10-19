use crate::daemon::surface_bridge::SurfaceBridge;
use crate::engine::EngineProgress;
use gtk4::prelude::*;

#[derive(Clone)]
pub struct ProgressHud {
    pub window: gtk4::Window,
    title_label: gtk4::Label,
    status_label: gtk4::Label,
    progress_bar: gtk4::ProgressBar,
}

impl ProgressHud {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::Window::builder()
            .application(app)
            .title("Convext Batch Progress")
            .build();
        SurfaceBridge::configure_floating_panel(&window, "Convext Progress", 340, 115);

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        vbox.set_margin_top(14);
        vbox.set_margin_bottom(14);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);

        let title_label = gtk4::Label::new(Some("Processing..."));
        title_label.set_halign(gtk4::Align::Start);

        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.set_show_text(true);

        let status_label = gtk4::Label::new(Some("Initializing background worker"));
        status_label.set_halign(gtk4::Align::Start);
        status_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        vbox.append(&title_label);
        vbox.append(&progress_bar);
        vbox.append(&status_label);

        window.set_child(Some(&vbox));

        Self {
            window,
            title_label,
            status_label,
            progress_bar,
        }
    }

    pub fn apply_event(&self, event: &EngineProgress) {
        match event {
            EngineProgress::Started {
                title, total_items, ..
            } => {
                self.title_label.set_text(title);
                self.progress_bar.set_fraction(0.05);
                self.status_label
                    .set_text(&format!("0 / {} items processed", total_items));
                self.window.set_visible(true);
                self.window.present();
            }
            EngineProgress::Step {
                completed_items,
                total_items,
                status_text,
                ..
            } => {
                let frac = if *total_items > 0 {
                    (*completed_items as f64 / *total_items as f64).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                self.progress_bar.set_fraction(frac);
                self.status_label.set_text(status_text);
            }
            EngineProgress::Finished { summary, .. } => {
                self.progress_bar.set_fraction(1.0);
                self.status_label.set_text(summary);
                let win = self.window.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(2200), move || {
                    win.set_visible(false);
                });
            }
            EngineProgress::Failed { error, .. } => {
                self.title_label.set_text("Operation Failed");
                self.status_label.set_text(error);
                let win = self.window.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(4000), move || {
                    win.set_visible(false);
                });
            }
        }
    }
}
