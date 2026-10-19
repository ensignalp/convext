use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct FloatShelfWindow {
    pub window: adw::Window,
    pub files: Rc<RefCell<Vec<PathBuf>>>,
    list_box: gtk4::ListBox,
    counter_label: gtk4::Label,
}

impl FloatShelfWindow {
    pub fn new<F>(app: &gtk4::Application, on_open_studio: F) -> Self
    where
        F: Fn(Vec<PathBuf>) + 'static,
    {
        let window = adw::Window::builder()
            .application(app)
            .title("Convext — Yüzen Dosya Rafı")
            .default_width(360)
            .default_height(420)
            .build();

        let files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new("Convext Float Rafı", "Sürükle, Topla ve Toplu İşle");
        header_bar.set_title_widget(Some(&win_title));

        let counter_label = gtk4::Label::new(Some("0 dosya"));
        counter_label.add_css_class("caption");
        header_bar.pack_start(&counter_label);
        toolbar_view.add_top_bar(&header_bar);

        let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        root_box.set_margin_top(12);
        root_box.set_margin_bottom(12);
        root_box.set_margin_start(12);
        root_box.set_margin_end(12);

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&list_box));
        root_box.append(&scrolled);

        let drag_out_btn = gtk4::Button::with_label("Tüm Dosyaları Sürükle ve Çıkart");
        drag_out_btn.add_css_class("pill");

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
        let studio_btn = gtk4::Button::with_label("Convext Menüde Aç");
        studio_btn.add_css_class("suggested-action");
        studio_btn.add_css_class("pill");
        studio_btn.set_hexpand(true);

        let clear_btn = gtk4::Button::with_label("Temizle");
        clear_btn.add_css_class("destructive-action");
        clear_btn.add_css_class("pill");

        action_row.append(&studio_btn);
        action_row.append(&clear_btn);
        root_box.append(&action_row);

        toolbar_view.set_content(Some(&root_box));
        window.set_content(Some(&toolbar_view));

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

        let files_studio = files.clone();
        let on_studio_rc = Rc::new(on_open_studio);
        studio_btn.connect_clicked(move |_| {
            let current = files_studio.borrow().clone();
            if !current.is_empty() {
                on_studio_rc(current);
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
            .set_text(&format!("{} dosya", current.len()));

        for path in current {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let row = adw::ActionRow::builder()
                .title(fname)
                .subtitle(path.to_string_lossy().as_ref())
                .build();

            let remove_btn = gtk4::Button::builder()
                .label("×")
                .valign(gtk4::Align::Center)
                .build();
            remove_btn.add_css_class("flat");
            let shelf_rm = self.clone();
            let path_rm = path.clone();
            remove_btn.connect_clicked(move |_| {
                shelf_rm.files.borrow_mut().retain(|p| p != &path_rm);
                shelf_rm.rebuild_list();
            });

            row.add_suffix(&remove_btn);
            self.list_box.append(&row);
        }
    }
}
