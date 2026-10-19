use crate::daemon::surface_bridge::SurfaceBridge;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct FloatShelfWindow {
    pub window: gtk4::Window,
    pub files: Rc<RefCell<Vec<PathBuf>>>,
    list_box: gtk4::ListBox,
    counter_label: gtk4::Label,
}

impl FloatShelfWindow {
    pub fn new<F>(app: &gtk4::Application, on_send_to_wheel: F) -> Self
    where
        F: Fn(Vec<PathBuf>) + 'static,
    {
        let window = gtk4::Window::builder()
            .application(app)
            .title("Tangerine Floating Shelf")
            .build();
        SurfaceBridge::configure_floating_panel(&window, "Tangerine Shelf", 320, 380);
        window.add_css_class("citrus-shelf-window");

        let files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));

        let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        root_box.set_margin_top(14);
        root_box.set_margin_bottom(14);
        root_box.set_margin_start(14);
        root_box.set_margin_end(14);

        let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let title_lbl = gtk4::Label::new(Some("Floating Shelf"));
        title_lbl.add_css_class("citrus-header");
        title_lbl.set_hexpand(true);
        title_lbl.set_halign(gtk4::Align::Start);

        let counter_label = gtk4::Label::new(Some("0 files"));
        counter_label.add_css_class("citrus-badge");

        let close_btn = gtk4::Button::with_label("Hide");
        close_btn.add_css_class("citrus-btn-secondary");
        let win_hide = window.clone();
        close_btn.connect_clicked(move |_| {
            win_hide.set_visible(false);
        });

        header_box.append(&title_lbl);
        header_box.append(&counter_label);
        header_box.append(&close_btn);
        root_box.append(&header_box);

        let hint_lbl = gtk4::Label::new(Some(
            "Drop files from any folder to collect them, then drag out or process in batch.",
        ));
        hint_lbl.add_css_class("citrus-Subheader");
        hint_lbl.set_wrap(true);
        hint_lbl.set_halign(gtk4::Align::Start);
        root_box.append(&hint_lbl);

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        list_box.add_css_class("citrus-card");
        scrolled.set_child(Some(&list_box));
        root_box.append(&scrolled);

        let drag_out_btn = gtk4::Button::with_label("Drag All Files Out");
        drag_out_btn.add_css_class("citrus-btn-secondary");

        let drag_source = gtk4::DragSource::new();
        drag_source.set_actions(gdk4::DragAction::COPY);
        let files_drag = files.clone();
        drag_source.connect_prepare(move |_, _, _| {
            let current = files_drag.borrow();
            if current.is_empty() {
                return None;
            }
            let gfiles: Vec<gio::File> = current
                .iter()
                .map(|p| gio::File::for_path(p))
                .collect();
            let flist = gdk4::FileList::from_array(&gfiles);
            Some(gdk4::ContentProvider::for_value(&flist.to_value()))
        });
        drag_out_btn.add_controller(drag_source);
        root_box.append(&drag_out_btn);

        let action_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let wheel_btn = gtk4::Button::with_label("Open in Radial Wheel");
        wheel_btn.add_css_class("citrus-btn-primary");
        wheel_btn.set_hexpand(true);

        let clear_btn = gtk4::Button::with_label("Clear");
        clear_btn.add_css_class("citrus-btn-danger");

        action_row.append(&wheel_btn);
        action_row.append(&clear_btn);
        root_box.append(&action_row);

        window.set_child(Some(&root_box));

        let shelf = Self {
            window: window.clone(),
            files: files.clone(),
            list_box: list_box.clone(),
            counter_label: counter_label.clone(),
        };

        let shelf_drop = shelf.clone();
        let drop_target =
            gtk4::DropTarget::new(gdk4::FileList::static_type(), gdk4::DragAction::COPY);
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(file_list) = value.get::<gdk4::FileList>() {
                let dropped: Vec<PathBuf> = file_list
                    .files()
                    .into_iter()
                    .filter_map(|f| f.path())
                    .collect();
                shelf_drop.add_files(dropped);
                return true;
            }
            false
        });
        window.add_controller(drop_target);

        let shelf_clear = shelf.clone();
        clear_btn.connect_clicked(move |_| {
            shelf_clear.clear();
        });

        let files_wheel = files.clone();
        let on_wheel_rc = Rc::new(on_send_to_wheel);
        wheel_btn.connect_clicked(move |_| {
            let current = files_wheel.borrow().clone();
            if !current.is_empty() {
                on_wheel_rc(current);
            }
        });

        shelf
    }

    pub fn add_files(&self, new_files: Vec<PathBuf>) {
        {
            let mut vec = self.files.borrow_mut();
            for p in new_files {
                if !vec.contains(&p) && p.exists() {
                    vec.push(p);
                }
            }
        }
        self.rebuild_list();
    }

    pub fn clear(&self) {
        self.files.borrow_mut().clear();
        self.rebuild_list();
    }

    fn rebuild_list(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        let current = self.files.borrow().clone();
        self.counter_label
            .set_text(&format!("{} files", current.len()));

        for path in current {
            let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            row.set_margin_top(4);
            row.set_margin_bottom(4);
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let lbl = gtk4::Label::new(Some(fname));
            lbl.set_hexpand(true);
            lbl.set_halign(gtk4::Align::Start);
            lbl.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);

            let remove_btn = gtk4::Button::with_label("×");
            remove_btn.add_css_class("citrus-btn-secondary");
            let shelf_rm = self.clone();
            let path_rm = path.clone();
            remove_btn.connect_clicked(move |_| {
                shelf_rm.files.borrow_mut().retain(|p| p != &path_rm);
                shelf_rm.rebuild_list();
            });

            row.append(&lbl);
            row.append(&remove_btn);
            self.list_box.append(&row);
        }
    }
}
