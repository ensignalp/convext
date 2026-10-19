use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionTool {
    Blackout,
    Pixelate,
    Crop,
}

#[derive(Debug, Clone)]
pub struct RedactionBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub tool: RedactionTool,
}

struct EditorState {
    tool: RedactionTool,
    boxes: Vec<RedactionBox>,
    drag_start: Option<(f64, f64)>,
    drag_curr: Option<(f64, f64)>,
    img_width: u32,
    img_height: u32,
    pixbuf: Option<gdk4::gdk_pixbuf::Pixbuf>,
}

pub struct ImageEditorModal;

impl ImageEditorModal {
    pub fn show<F>(app: &gtk4::Application, file_path: PathBuf, on_apply: F)
    where
        F: Fn(PathBuf, Vec<RedactionBox>) + 'static,
    {
        let window = adw::Window::builder()
            .application(app)
            .title("Convext — Görsel Sansürleme & Kırpma")
            .default_width(720)
            .default_height(560)
            .build();

        let pixbuf = gdk4::gdk_pixbuf::Pixbuf::from_file(&file_path).ok();
        let (img_width, img_height) = Self::read_dimensions(&file_path);

        let state = Rc::new(RefCell::new(EditorState {
            tool: RedactionTool::Pixelate,
            boxes: Vec::new(),
            drag_start: None,
            drag_curr: None,
            img_width,
            img_height,
            pixbuf,
        }));

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new(
            "Convext Görsel Düzenleyici",
            "Kalıcı Piksel Sansürleme (Pixelate / Blackout) & Kırpma",
        );
        header_bar.set_title_widget(Some(&win_title));

        let btn_pixelate = gtk4::Button::with_label("Mozaik (Pixelate)");
        btn_pixelate.add_css_class("suggested-action");
        let btn_blackout = gtk4::Button::with_label("Karart (Blackout)");
        let btn_crop = gtk4::Button::with_label("Kırp (Crop)");
        let btn_undo = gtk4::Button::with_label("Geri Al");

        header_bar.pack_start(&btn_pixelate);
        header_bar.pack_start(&btn_blackout);
        header_bar.pack_start(&btn_crop);
        header_bar.pack_end(&btn_undo);
        toolbar_view.add_top_bar(&header_bar);

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(12);

        let canvas = gtk4::DrawingArea::builder()
            .content_width(660)
            .content_height(430)
            .vexpand(true)
            .hexpand(true)
            .build();
        canvas.add_css_class("card");

        let st_draw = state.clone();
        canvas.set_draw_func(move |_, cr, w, h| {
            let st = st_draw.borrow();
            cr.set_source_rgba(0.11, 0.11, 0.13, 1.0);
            let _ = cr.paint();

            let scale_x = w as f64 / st.img_width.max(1) as f64;
            let scale_y = h as f64 / st.img_height.max(1) as f64;
            let scale = scale_x.min(scale_y).min(1.0);
            let draw_w = st.img_width as f64 * scale;
            let draw_h = st.img_height as f64 * scale;
            let offset_x = (w as f64 - draw_w) / 2.0;
            let offset_y = (h as f64 - draw_h) / 2.0;

            if let Some(ref pb) = st.pixbuf {
                cr.save().ok();
                cr.translate(offset_x, offset_y);
                cr.scale(scale, scale);
                cr.set_source_pixbuf(pb, 0.0, 0.0);
                let _ = cr.paint();
                cr.restore().ok();
            }

            for b in &st.boxes {
                let rx = offset_x + b.x as f64 * scale;
                let ry = offset_y + b.y as f64 * scale;
                let rw = b.width as f64 * scale;
                let rh = b.height as f64 * scale;
                cr.rectangle(rx, ry, rw, rh);
                match b.tool {
                    RedactionTool::Blackout => cr.set_source_rgba(0.0, 0.0, 0.0, 0.92),
                    RedactionTool::Pixelate => cr.set_source_rgba(0.21, 0.52, 0.89, 0.45),
                    RedactionTool::Crop => cr.set_source_rgba(0.2, 0.85, 0.45, 0.32),
                }
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.38, 0.63, 0.92, 0.95);
                cr.set_line_width(1.8);
                let _ = cr.stroke();
            }

            if let (Some((x0, y0)), Some((x1, y1))) = (st.drag_start, st.drag_curr) {
                let rx = x0.min(x1);
                let ry = y0.min(y1);
                let rw = (x1 - x0).abs();
                let rh = (y1 - y0).abs();
                cr.rectangle(rx, ry, rw, rh);
                cr.set_source_rgba(0.21, 0.52, 0.89, 0.30);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.21, 0.52, 0.89, 0.95);
                cr.set_line_width(2.0);
                let _ = cr.stroke();
            }
        });

        let drag = gtk4::GestureDrag::new();
        let st_b = state.clone();
        let cv_b = canvas.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let mut st = st_b.borrow_mut();
            st.drag_start = Some((x, y));
            st.drag_curr = Some((x, y));
            cv_b.queue_draw();
        });

        let st_u = state.clone();
        let cv_u = canvas.clone();
        drag.connect_drag_update(move |_, dx, dy| {
            let mut st = st_u.borrow_mut();
            if let Some((sx, sy)) = st.drag_start {
                st.drag_curr = Some((sx + dx, sy + dy));
                cv_u.queue_draw();
            }
        });

        let st_e = state.clone();
        let cv_e = canvas.clone();
        drag.connect_drag_end(move |_, dx, dy| {
            let mut st = st_e.borrow_mut();
            let w = cv_e.width() as f64;
            let h = cv_e.height() as f64;
            let scale_x = w / st.img_width.max(1) as f64;
            let scale_y = h / st.img_height.max(1) as f64;
            let scale = scale_x.min(scale_y).min(1.0).max(0.01);
            let draw_w = st.img_width as f64 * scale;
            let draw_h = st.img_height as f64 * scale;
            let offset_x = (w - draw_w) / 2.0;
            let offset_y = (h - draw_h) / 2.0;

            if let Some((sx, sy)) = st.drag_start {
                let ex = sx + dx;
                let ey = sy + dy;
                let rx = ((sx.min(ex) - offset_x) / scale).max(0.0) as u32;
                let ry = ((sy.min(ey) - offset_y) / scale).max(0.0) as u32;
                let rw = ((ex - sx).abs() / scale) as u32;
                let rh = ((ey - sy).abs() / scale) as u32;
                if rw > 4 && rh > 4 {
                    let tool = st.tool;
                    st.boxes.push(RedactionBox {
                        x: rx,
                        y: ry,
                        width: rw,
                        height: rh,
                        tool,
                    });
                }
            }
            st.drag_start = None;
            st.drag_curr = None;
            cv_e.queue_draw();
        });
        canvas.add_controller(drag);
        root.append(&canvas);

        let st_pix = state.clone();
        btn_pixelate.connect_clicked(move |_| {
            st_pix.borrow_mut().tool = RedactionTool::Pixelate;
        });
        let st_blk = state.clone();
        btn_blackout.connect_clicked(move |_| {
            st_blk.borrow_mut().tool = RedactionTool::Blackout;
        });
        let st_crp = state.clone();
        btn_crop.connect_clicked(move |_| {
            st_crp.borrow_mut().tool = RedactionTool::Crop;
        });
        let st_undo = state.clone();
        let cv_undo = canvas.clone();
        btn_undo.connect_clicked(move |_| {
            st_undo.borrow_mut().boxes.pop();
            cv_undo.queue_draw();
        });

        let footer = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        footer.set_halign(gtk4::Align::End);
        let cancel_btn = gtk4::Button::with_label("İptal");
        cancel_btn.add_css_class("pill");
        let apply_btn = gtk4::Button::with_label("Uygula ve Kaydet");
        apply_btn.add_css_class("suggested-action");
        apply_btn.add_css_class("pill");

        let win_cancel = window.clone();
        cancel_btn.connect_clicked(move |_| {
            win_cancel.close();
        });

        let win_apply = window.clone();
        let st_apply = state.clone();
        apply_btn.connect_clicked(move |_| {
            let boxes = st_apply.borrow().boxes.clone();
            on_apply(file_path.clone(), boxes);
            win_apply.close();
        });

        footer.append(&cancel_btn);
        footer.append(&apply_btn);
        root.append(&footer);

        toolbar_view.set_content(Some(&root));
        window.set_content(Some(&toolbar_view));
        window.present();
    }

    fn read_dimensions(path: &Path) -> (u32, u32) {
        image::image_dimensions(path).unwrap_or((800, 600))
    }
}
