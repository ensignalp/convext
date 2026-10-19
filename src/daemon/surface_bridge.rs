use gtk4::prelude::*;
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_ulong, c_void};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CompositorBackend {
    Wayland,
    X11,
    Unknown,
}

#[repr(C)]
struct XSetWindowAttributes {
    background_pixmap: c_ulong,
    background_pixel: c_ulong,
    border_pixmap: c_ulong,
    border_pixel: c_ulong,
    bit_gravity: c_int,
    win_gravity: c_int,
    backing_store: c_int,
    backing_planes: c_ulong,
    backing_pixel: c_ulong,
    save_under: c_int,
    event_mask: c_long,
    do_not_propagate_mask: c_long,
    override_redirect: c_int,
    colormap: c_ulong,
    cursor: c_ulong,
}

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(display_name: *const c_char) -> *mut c_void;
    fn XCloseDisplay(display: *mut c_void) -> c_int;
    fn XInternAtom(display: *mut c_void, atom_name: *const c_char, only_if_exists: c_int) -> c_ulong;
    fn XChangeProperty(
        display: *mut c_void,
        w: c_ulong,
        property: c_ulong,
        type_: c_ulong,
        format: c_int,
        mode: c_int,
        data: *const c_uchar,
        nelements: c_int,
    ) -> c_int;
    fn XMoveWindow(display: *mut c_void, w: c_ulong, x: c_int, y: c_int) -> c_int;
    fn XFlush(display: *mut c_void) -> c_int;
    fn XGetInputFocus(display: *mut c_void, focus_return: *mut c_ulong, revert_to_return: *mut c_int) -> c_int;
}

pub struct SurfaceBridge;

impl SurfaceBridge {
    pub fn detect_backend() -> CompositorBackend {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            CompositorBackend::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            CompositorBackend::X11
        } else {
            CompositorBackend::Unknown
        }
    }

    pub fn configure_overlay_window(window: &gtk4::Window, width: i32, height: i32) {
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_deletable(false);
        window.set_default_size(width, height);
        window.remove_css_class("background");
        window.remove_css_class("csd");
        window.remove_css_class("solid-csd");
        window.add_css_class("citrus-transparent-window");
    }

    pub fn configure_floating_panel(window: &gtk4::Window, title: &str, width: i32, height: i32) {
        window.set_title(Some(title));
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_default_size(width, height);
        window.remove_css_class("background");
        window.remove_css_class("csd");
        window.remove_css_class("solid-csd");
    }

    pub fn present_warm_surface(window: &gtk4::Window, x: i32, y: i32) {
        window.remove_css_class("background");
        window.remove_css_class("csd");
        window.remove_css_class("solid-csd");
        window.set_visible(true);
        window.present();

        // Apply borderless X11 window type hints & center on cursor when running under X11/XWayland
        let target_x = (x - 210).max(0);
        let target_y = (y - 200).max(0);
        unsafe {
            let dpy = XOpenDisplay(std::ptr::null());
            if !dpy.is_null() {
                let mut focused_win: c_ulong = 0;
                let mut revert: c_int = 0;
                XGetInputFocus(dpy, &mut focused_win, &mut revert);
                if focused_win > 1 {
                    let wm_type = XInternAtom(dpy, b"_NET_WM_WINDOW_TYPE\0".as_ptr() as *const c_char, 0);
                    let wm_splash = XInternAtom(dpy, b"_NET_WM_WINDOW_TYPE_SPLASH\0".as_ptr() as *const c_char, 0);
                    let xa_atom: c_ulong = 4;
                    XChangeProperty(
                        dpy,
                        focused_win,
                        wm_type,
                        xa_atom,
                        32,
                        0, // PropModeReplace
                        &wm_splash as *const c_ulong as *const c_uchar,
                        1,
                    );
                    if x > 0 || y > 0 {
                        XMoveWindow(dpy, focused_win, target_x, target_y);
                    }
                    XFlush(dpy);
                }
                XCloseDisplay(dpy);
            }
        }
    }

    pub fn hide_warm_surface(window: &gtk4::Window) {
        window.set_visible(false);
    }
}
