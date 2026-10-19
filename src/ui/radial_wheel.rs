use crate::daemon::WheelMode;
use crate::engine::mime_router::{SliceAction, SliceIcon, WheelSlice};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct RadialWheelState {
    pub slices: Vec<WheelSlice>,
    pub active_files: Vec<PathBuf>,
    pub mode: WheelMode,
    pub hovered_index: Option<usize>,
    pub pointer_x: f64,
    pub pointer_y: f64,
    pub outer_radius: f64,
    pub inner_radius: f64,
}

impl Default for RadialWheelState {
    fn default() -> Self {
        Self {
            slices: Vec::new(),
            active_files: Vec::new(),
            mode: WheelMode::ConvertFormats,
            hovered_index: Some(0),
            pointer_x: 210.0,
            pointer_y: 210.0,
            outer_radius: 170.0,
            inner_radius: 66.0,
        }
    }
}

#[derive(Clone)]
pub struct RadialWheelWidget {
    pub container: gtk4::Box,
    pub drawing_area: gtk4::DrawingArea,
    pub state: Rc<RefCell<RadialWheelState>>,
}

impl RadialWheelWidget {
    pub fn new<F>(on_select: F) -> Self
    where
        F: Fn(SliceAction, Vec<PathBuf>) + 'static,
    {
        let state = Rc::new(RefCell::new(RadialWheelState::default()));
        let drawing_area = gtk4::DrawingArea::builder()
            .content_width(420)
            .content_height(450)
            .hexpand(true)
            .vexpand(true)
            .build();

        let state_draw = state.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let st = state_draw.borrow();
            Self::render_wheel(cr, width as f64, height as f64, &st);
        });

        let state_motion = state.clone();
        let area_motion = drawing_area.clone();
        let motion_ctrl = gtk4::EventControllerMotion::new();
        motion_ctrl.connect_motion(move |_, x, y| {
            let mut st = state_motion.borrow_mut();
            st.pointer_x = x;
            st.pointer_y = y;
            let cx = 210.0;
            let cy = 200.0;
            let dx = x - cx;
            let dy = y - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let prev = st.hovered_index;

            if dist >= st.inner_radius - 10.0
                && dist <= st.outer_radius + 18.0
                && !st.slices.is_empty()
            {
                let n = st.slices.len();
                if n == 1 {
                    st.hovered_index = Some(0);
                } else {
                    let step = (2.0 * PI) / (n as f64);
                    let mut rel = dy.atan2(dx) - (-PI / 2.0 - step / 2.0);
                    while rel < 0.0 {
                        rel += 2.0 * PI;
                    }
                    while rel >= 2.0 * PI {
                        rel -= 2.0 * PI;
                    }
                    let idx = ((rel / step).floor() as usize).min(n - 1);
                    st.hovered_index = Some(idx);
                }
            }

            if st.hovered_index != prev {
                area_motion.queue_draw();
            }
        });
        drawing_area.add_controller(motion_ctrl);

        let state_click = state.clone();
        let on_select_rc = Rc::new(on_select);
        let click_ctrl = gtk4::GestureClick::new();
        click_ctrl.set_button(1);
        click_ctrl.connect_released(move |_, _, _, _| {
            let st = state_click.borrow();
            if let Some(idx) = st.hovered_index {
                if let Some(slice) = st.slices.get(idx) {
                    if slice.enabled {
                        on_select_rc(slice.action.clone(), st.active_files.clone());
                    }
                }
            }
        });
        drawing_area.add_controller(click_ctrl);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.append(&drawing_area);

        Self {
            container,
            drawing_area,
            state,
        }
    }

    pub fn update_context(&self, files: Vec<PathBuf>, mode: WheelMode, slices: Vec<WheelSlice>) {
        {
            let mut st = self.state.borrow_mut();
            st.active_files = files;
            st.mode = mode;
            st.slices = slices;
            st.hovered_index = if st.slices.is_empty() { None } else { Some(0) };
        }
        self.drawing_area.queue_draw();
    }

    fn draw_rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
        let radius = r.min(w / 2.0).min(h / 2.0);
        cr.new_sub_path();
        cr.arc(x + w - radius, y + radius, radius, -PI / 2.0, 0.0);
        cr.arc(x + w - radius, y + h - radius, radius, 0.0, PI / 2.0);
        cr.arc(x + radius, y + h - radius, radius, PI / 2.0, PI);
        cr.arc(x + radius, y + radius, radius, PI, 3.0 * PI / 2.0);
        cr.close_path();
    }

    fn build_rounded_wedge_path(
        cr: &cairo::Context,
        cx: f64,
        cy: f64,
        r_in: f64,
        r_out: f64,
        a0: f64,
        a1: f64,
        corner_r: f64,
    ) {
        let span = (a1 - a0).max(0.05);
        let d_out = (corner_r / r_out).min(span * 0.28);
        let d_in = (corner_r / r_in).min(span * 0.28);

        let r_out_in = (r_out - corner_r).max(r_in + 4.0);
        let r_in_out = (r_in + corner_r).min(r_out - 4.0);

        cr.new_sub_path();

        let p_in_start_x = cx + r_in * (a0 + d_in).cos();
        let p_in_start_y = cy + r_in * (a0 + d_in).sin();
        cr.move_to(p_in_start_x, p_in_start_y);

        let p_out_pre_x = cx + r_out_in * a0.cos();
        let p_out_pre_y = cy + r_out_in * a0.sin();
        let p_in_pre_x = cx + r_in_out * a0.cos();
        let p_in_pre_y = cy + r_in_out * a0.sin();
        cr.line_to(p_in_pre_x, p_in_pre_y);
        cr.line_to(p_out_pre_x, p_out_pre_y);

        let c_out_a0_x = cx + r_out * a0.cos();
        let c_out_a0_y = cy + r_out * a0.sin();
        let p_out_arc0_x = cx + r_out * (a0 + d_out).cos();
        let p_out_arc0_y = cy + r_out * (a0 + d_out).sin();
        cr.curve_to(
            c_out_a0_x,
            c_out_a0_y,
            c_out_a0_x,
            c_out_a0_y,
            p_out_arc0_x,
            p_out_arc0_y,
        );

        if a1 - d_out > a0 + d_out {
            cr.arc(cx, cy, r_out, a0 + d_out, a1 - d_out);
        }

        let c_out_a1_x = cx + r_out * a1.cos();
        let c_out_a1_y = cy + r_out * a1.sin();
        let p_out_post_x = cx + r_out_in * a1.cos();
        let p_out_post_y = cy + r_out_in * a1.sin();
        cr.curve_to(
            c_out_a1_x,
            c_out_a1_y,
            c_out_a1_x,
            c_out_a1_y,
            p_out_post_x,
            p_out_post_y,
        );

        let p_in_post_x = cx + r_in_out * a1.cos();
        let p_in_post_y = cy + r_in_out * a1.sin();
        cr.line_to(p_in_post_x, p_in_post_y);

        let c_in_a1_x = cx + r_in * a1.cos();
        let c_in_a1_y = cy + r_in * a1.sin();
        let p_in_arc1_x = cx + r_in * (a1 - d_in).cos();
        let p_in_arc1_y = cy + r_in * (a1 - d_in).sin();
        cr.curve_to(
            c_in_a1_x,
            c_in_a1_y,
            c_in_a1_x,
            c_in_a1_y,
            p_in_arc1_x,
            p_in_arc1_y,
        );

        if a1 - d_in > a0 + d_in {
            cr.arc_negative(cx, cy, r_in, a1 - d_in, a0 + d_in);
        }

        let c_in_a0_x = cx + r_in * a0.cos();
        let c_in_a0_y = cy + r_in * a0.sin();
        cr.curve_to(
            c_in_a0_x,
            c_in_a0_y,
            c_in_a0_x,
            c_in_a0_y,
            p_in_pre_x,
            p_in_pre_y,
        );

        cr.close_path();
    }

    fn draw_slice_icon(cr: &cairo::Context, icon: SliceIcon, x: f64, y: f64, active: bool) {
        if active {
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.98);
        } else {
            cr.set_source_rgba(0.92, 0.93, 0.96, 0.92);
        }
        cr.set_line_width(1.9);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);

        match icon {
            SliceIcon::None => {}
            SliceIcon::Compress => {
                cr.move_to(x - 7.0, y - 7.0);
                cr.line_to(x - 2.0, y - 2.0);
                cr.move_to(x - 6.0, y - 2.0);
                cr.line_to(x - 2.0, y - 2.0);
                cr.line_to(x - 2.0, y - 6.0);

                cr.move_to(x + 7.0, y + 7.0);
                cr.line_to(x + 2.0, y + 2.0);
                cr.move_to(x + 6.0, y + 2.0);
                cr.line_to(x + 2.0, y + 2.0);
                cr.line_to(x + 2.0, y + 6.0);
                let _ = cr.stroke();
            }
            SliceIcon::Metadata => {
                Self::draw_rounded_rect(cr, x - 6.5, y - 5.5, 13.0, 11.0, 2.5);
                cr.move_to(x - 8.0, y + 7.0);
                cr.line_to(x + 8.0, y - 7.0);
                let _ = cr.stroke();
            }
            SliceIcon::Edit => {
                for (idx, dy) in [-5.0, 0.0, 5.0].iter().enumerate() {
                    cr.move_to(x - 7.5, y + dy);
                    cr.line_to(x + 7.5, y + dy);
                    let _ = cr.stroke();
                    let knob_x = if idx == 1 { x + 3.0 } else { x - 3.0 };
                    cr.arc(knob_x, y + dy, 2.0, 0.0, 2.0 * PI);
                    let _ = cr.fill();
                }
            }
            SliceIcon::Annotate => {
                cr.move_to(x - 6.0, y + 6.0);
                cr.line_to(x - 2.0, y + 6.0);
                cr.line_to(x + 6.5, y - 2.5);
                cr.line_to(x + 2.5, y - 6.5);
                cr.line_to(x - 6.0, y + 2.0);
                cr.close_path();
                let _ = cr.stroke();
            }
            SliceIcon::AddBg => {
                Self::draw_rounded_rect(cr, x - 7.0, y - 6.5, 14.0, 13.0, 2.5);
                cr.move_to(x - 4.5, y + 4.0);
                cr.line_to(x - 1.0, y - 1.0);
                cr.line_to(x + 2.0, y + 2.0);
                cr.line_to(x + 5.0, y - 2.0);
                let _ = cr.stroke();
            }
            SliceIcon::Crop => {
                cr.move_to(x - 5.0, y - 8.0);
                cr.line_to(x - 5.0, y + 5.0);
                cr.line_to(x + 8.0, y + 5.0);
                cr.move_to(x - 8.0, y - 5.0);
                cr.line_to(x + 5.0, y - 5.0);
                cr.line_to(x + 5.0, y + 8.0);
                let _ = cr.stroke();
            }
            SliceIcon::Redact => {
                cr.arc(x, y, 6.5, PI * 1.15, PI * 1.85);
                let _ = cr.stroke();
                cr.arc(x, y, 6.5, PI * 0.15, PI * 0.85);
                let _ = cr.stroke();
                cr.move_to(x - 7.5, y + 6.5);
                cr.line_to(x + 7.5, y - 6.5);
                let _ = cr.stroke();
            }
            SliceIcon::Mute => {
                cr.move_to(x - 7.0, y - 3.0);
                cr.line_to(x - 3.5, y - 3.0);
                cr.line_to(x + 0.5, y - 6.5);
                cr.line_to(x + 0.5, y + 6.5);
                cr.line_to(x - 3.5, y + 3.0);
                cr.line_to(x - 7.0, y + 3.0);
                cr.close_path();
                cr.move_to(x + 3.5, y - 3.5);
                cr.line_to(x + 8.0, y + 3.5);
                cr.move_to(x + 8.0, y - 3.5);
                cr.line_to(x + 3.5, y + 3.5);
                let _ = cr.stroke();
            }
            SliceIcon::Trim => {
                cr.arc(x - 4.5, y - 4.0, 2.5, 0.0, 2.0 * PI);
                let _ = cr.stroke();
                cr.arc(x - 4.5, y + 4.0, 2.5, 0.0, 2.0 * PI);
                let _ = cr.stroke();
                cr.move_to(x - 2.0, y - 3.0);
                cr.line_to(x + 7.0, y + 5.0);
                cr.move_to(x - 2.0, y + 3.0);
                cr.line_to(x + 7.0, y - 5.0);
                let _ = cr.stroke();
            }
            SliceIcon::Speed => {
                cr.arc(x, y + 2.0, 7.5, PI, 2.0 * PI);
                cr.move_to(x, y + 2.0);
                cr.line_to(x + 4.5, y - 3.5);
                let _ = cr.stroke();
            }
            SliceIcon::Snapshot => {
                Self::draw_rounded_rect(cr, x - 7.5, y - 5.0, 15.0, 11.0, 2.5);
                let _ = cr.stroke();
                cr.arc(x, y + 0.5, 3.0, 0.0, 2.0 * PI);
                let _ = cr.stroke();
            }
            SliceIcon::Split | SliceIcon::Channels => {
                Self::draw_rounded_rect(cr, x - 7.0, y - 6.0, 5.5, 12.0, 1.5);
                Self::draw_rounded_rect(cr, x + 1.5, y - 6.0, 5.5, 12.0, 1.5);
                let _ = cr.stroke();
            }
            SliceIcon::Normalize | SliceIcon::Bleep => {
                for (dx, h) in [(-6.0, 4.0), (-2.0, 7.5), (2.0, 5.5), (6.0, 3.0)] {
                    cr.move_to(x + dx, y - h);
                    cr.line_to(x + dx, y + h);
                }
                let _ = cr.stroke();
            }
            SliceIcon::Visualize => {
                Self::draw_rounded_rect(cr, x - 7.5, y - 6.0, 15.0, 12.0, 2.0);
                cr.move_to(x - 5.0, y + 1.0);
                cr.line_to(x - 2.0, y - 3.0);
                cr.line_to(x + 1.0, y + 3.0);
                cr.line_to(x + 4.5, y - 1.5);
                let _ = cr.stroke();
            }
            SliceIcon::Organize => {
                for (dx, dy) in [(-5.5, -5.5), (1.5, -5.5), (-5.5, 1.5), (1.5, 1.5)] {
                    Self::draw_rounded_rect(cr, x + dx, y + dy, 4.2, 4.2, 1.0);
                }
                let _ = cr.stroke();
            }
            SliceIcon::Extract => {
                Self::draw_rounded_rect(cr, x - 7.0, y - 5.5, 14.0, 11.5, 2.0);
                cr.move_to(x, y + 3.5);
                cr.line_to(x, y - 2.5);
                cr.move_to(x - 3.0, y + 0.5);
                cr.line_to(x, y - 2.5);
                cr.line_to(x + 3.0, y + 0.5);
                let _ = cr.stroke();
            }
            SliceIcon::AirDrop => {
                cr.arc(x, y - 1.0, 3.0, PI * 0.8, PI * 2.2);
                let _ = cr.stroke();
                cr.arc(x, y - 1.0, 6.2, PI * 0.85, PI * 2.15);
                let _ = cr.stroke();
                cr.move_to(x, y + 1.0);
                cr.line_to(x - 4.0, y + 7.0);
                cr.line_to(x + 4.0, y + 7.0);
                cr.close_path();
                let _ = cr.stroke();
            }
            SliceIcon::Float => {
                cr.move_to(x - 4.5, y - 6.0);
                cr.line_to(x + 4.5, y - 6.0);
                cr.line_to(x + 3.0, y);
                cr.line_to(x - 3.0, y);
                cr.close_path();
                cr.move_to(x, y);
                cr.line_to(x, y + 7.0);
                let _ = cr.stroke();
            }
            SliceIcon::Copy => {
                Self::draw_rounded_rect(cr, x - 6.5, y - 6.5, 9.0, 9.0, 1.8);
                Self::draw_rounded_rect(cr, x - 2.5, y - 2.5, 9.0, 9.0, 1.8);
                let _ = cr.stroke();
            }
            SliceIcon::Move => {
                Self::draw_rounded_rect(cr, x - 7.5, y - 5.5, 15.0, 11.0, 2.0);
                cr.move_to(x - 3.0, y);
                cr.line_to(x + 3.5, y);
                cr.move_to(x + 0.5, y - 3.0);
                cr.line_to(x + 3.5, y);
                cr.line_to(x + 0.5, y + 3.0);
                let _ = cr.stroke();
            }
            SliceIcon::CopyPath => {
                Self::draw_rounded_rect(cr, x - 6.0, y - 7.0, 12.0, 14.0, 2.0);
                cr.move_to(x - 3.0, y - 2.0);
                cr.line_to(x + 3.0, y - 2.0);
                cr.move_to(x - 3.0, y + 2.0);
                cr.line_to(x + 3.0, y + 2.0);
                let _ = cr.stroke();
            }
            SliceIcon::Terminal => {
                Self::draw_rounded_rect(cr, x - 7.5, y - 6.0, 15.0, 12.0, 2.2);
                cr.move_to(x - 4.5, y - 2.5);
                cr.line_to(x - 1.5, y);
                cr.line_to(x - 4.5, y + 2.5);
                cr.move_to(x + 0.5, y + 2.5);
                cr.line_to(x + 4.5, y + 2.5);
                let _ = cr.stroke();
            }
        }
    }

    fn render_wheel(cr: &cairo::Context, width: f64, _height: f64, st: &RadialWheelState) {
        let cx = width / 2.0;
        let cy = 200.0;
        let r_out = st.outer_radius;
        let r_in = st.inner_radius;

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        let _ = cr.paint();

        // Multi-pass soft circular glow / drop shadow (zero rectangular border)
        for i in (1..=8).rev() {
            let spread = (i as f64) * 2.8;
            let alpha = 0.04 * ((9 - i) as f64 / 8.0);
            cr.arc(cx, cy + 4.0, r_out + 10.0 + spread, 0.0, 2.0 * PI);
            cr.set_source_rgba(0.0, 0.0, 0.0, alpha);
            let _ = cr.fill();
        }

        // Main dark frosted glass outer ring
        let ring_grad = cairo::LinearGradient::new(
            cx,
            cy - r_out - 10.0,
            cx,
            cy + r_out + 10.0,
        );
        ring_grad.add_color_stop_rgba(0.0, 0.16, 0.17, 0.21, 0.94);
        ring_grad.add_color_stop_rgba(1.0, 0.09, 0.10, 0.13, 0.96);

        cr.arc(cx, cy, r_out + 10.0, 0.0, 2.0 * PI);
        let _ = cr.set_source(&ring_grad);
        let _ = cr.fill();

        let n = st.slices.len().max(1);
        let step = (2.0 * PI) / (n as f64);
        let gap = if n == 1 { 0.03 } else { 0.036 };

        for (i, slice) in st.slices.iter().enumerate() {
            let center_angle = -PI / 2.0 + (i as f64) * step;
            let start_angle = center_angle - step / 2.0 + gap;
            let end_angle = center_angle + step / 2.0 - gap;
            let is_hovered = st.hovered_index == Some(i);

            let slice_r_out = if is_hovered && slice.enabled {
                r_out + 4.0
            } else {
                r_out
            };

            Self::build_rounded_wedge_path(
                cr,
                cx,
                cy,
                r_in,
                slice_r_out,
                start_angle,
                end_angle,
                14.0,
            );

            if !slice.enabled {
                cr.set_source_rgba(0.14, 0.15, 0.18, 0.55);
            } else if is_hovered {
                let sx = cx + r_out * center_angle.cos();
                let sy = cy + r_out * center_angle.sin();
                let grad = cairo::LinearGradient::new(cx, cy, sx, sy);
                grad.add_color_stop_rgba(0.0, 1.0, 0.34, 0.04, 0.99);
                grad.add_color_stop_rgba(1.0, 1.0, 0.22, 0.0, 0.99);
                let _ = cr.set_source(&grad);
            } else {
                let sx = cx + r_out * center_angle.cos();
                let sy = cy + r_out * center_angle.sin();
                let grad = cairo::LinearGradient::new(cx, cy, sx, sy);
                grad.add_color_stop_rgba(0.0, 0.22, 0.23, 0.28, 0.90);
                grad.add_color_stop_rgba(1.0, 0.16, 0.17, 0.21, 0.94);
                let _ = cr.set_source(&grad);
            }
            let _ = cr.fill();

            let mid_r = (slice_r_out + r_in) / 2.0;
            let tx = cx + mid_r * center_angle.cos();
            let ty = cy + mid_r * center_angle.sin();

            if slice.icon != SliceIcon::None {
                Self::draw_slice_icon(cr, slice.icon, tx, ty - 8.5, is_hovered);
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(9.5);
                if is_hovered {
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                } else {
                    cr.set_source_rgba(0.92, 0.93, 0.96, 0.95);
                }
                if let Ok(ext) = cr.text_extents(&slice.label) {
                    cr.move_to(tx - ext.width() / 2.0, ty + 11.5);
                    let _ = cr.show_text(&slice.label);
                }
            } else {
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(13.5);
                if !slice.enabled {
                    cr.set_source_rgba(0.50, 0.50, 0.55, 0.75);
                } else if is_hovered {
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                } else {
                    cr.set_source_rgba(0.95, 0.96, 0.98, 0.96);
                }
                if let Ok(ext) = cr.text_extents(&slice.label) {
                    cr.move_to(tx - ext.width() / 2.0, ty + ext.height() / 2.0 - 1.0);
                    let _ = cr.show_text(&slice.label);
                }
            }
        }

        // Inner circular well
        cr.arc(cx, cy, r_in - 5.0, 0.0, 2.0 * PI);
        cr.set_source_rgba(0.08, 0.09, 0.11, 0.96);
        let _ = cr.fill();

        // Center Floating Pill Badge
        let active_slice = st
            .hovered_index
            .and_then(|idx| st.slices.get(idx))
            .or_else(|| st.slices.first());

        let badge_text = active_slice
            .map(|s| s.center_label.as_str())
            .unwrap_or("SELECT");
        let subtitle_text = active_slice
            .map(|s| s.subtitle.as_str())
            .unwrap_or("Select option");

        Self::draw_rounded_rect(cr, cx - 48.0, cy - 17.0, 96.0, 38.0, 19.0);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
        let _ = cr.fill();

        let pill_grad = cairo::LinearGradient::new(cx, cy - 19.0, cx, cy + 19.0);
        pill_grad.add_color_stop_rgba(0.0, 0.23, 0.24, 0.29, 0.98);
        pill_grad.add_color_stop_rgba(1.0, 0.16, 0.17, 0.21, 0.98);
        Self::draw_rounded_rect(cr, cx - 48.0, cy - 19.0, 96.0, 38.0, 19.0);
        let _ = cr.set_source(&pill_grad);
        let _ = cr.fill();

        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(11.0);
        cr.set_source_rgba(0.98, 0.98, 1.0, 0.98);
        if let Ok(ext) = cr.text_extents(badge_text) {
            cr.move_to(cx - ext.width() / 2.0, cy + 4.0);
            let _ = cr.show_text(badge_text);
        }

        // Bottom caption below the wheel
        let cap_y = cy + r_out + 34.0;
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(12.5);
        cr.set_source_rgba(0.85, 0.86, 0.92, 0.96);
        if let Ok(ext) = cr.text_extents(subtitle_text) {
            cr.move_to(cx - ext.width() / 2.0, cap_y);
            let _ = cr.show_text(subtitle_text);
        }
    }
}
