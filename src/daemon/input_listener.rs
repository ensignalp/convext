use crate::daemon::{DaemonEvent, WheelMode};
use async_channel::Sender;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[repr(C)]
struct LinuxInputEvent {
    tv_sec: libc::time_t,
    tv_usec: libc::suseconds_t,
    type_: u16,
    code: u16,
    value: i32,
}

const EV_KEY: u16 = 0x01;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;
const KEY_LEFTCTRL: u16 = 29;
const KEY_RIGHTCTRL: u16 = 97;
const KEY_ESC: u16 = 1;
const BTN_RIGHT: u16 = 0x111;

#[allow(non_camel_case_types)]
type Display = libc::c_void;
#[allow(non_camel_case_types)]
type Window = libc::c_ulong;

#[link(name = "X11")]
unsafe extern "C" {
    fn XOpenDisplay(display_name: *const libc::c_char) -> *mut Display;
    fn XDefaultRootWindow(display: *mut Display) -> Window;
    fn XQueryPointer(
        display: *mut Display,
        w: Window,
        root_return: *mut Window,
        child_return: *mut Window,
        root_x_return: *mut libc::c_int,
        root_y_return: *mut libc::c_int,
        win_x_return: *mut libc::c_int,
        win_y_return: *mut libc::c_int,
        mask_return: *mut libc::c_uint,
    ) -> libc::c_int;
    fn XQueryKeymap(display: *mut Display, keys_return: *mut libc::c_char) -> libc::c_int;
    fn XKeysymToKeycode(display: *mut Display, keysym: libc::c_ulong) -> libc::c_uchar;
}

const XK_SHIFT_L: libc::c_ulong = 0xffe1;
const XK_SHIFT_R: libc::c_ulong = 0xffe2;
const XK_CONTROL_L: libc::c_ulong = 0xffe3;
const XK_CONTROL_R: libc::c_ulong = 0xffe4;
const SHIFT_MASK: libc::c_uint = 1 << 0;
const CONTROL_MASK: libc::c_uint = 1 << 2;

#[derive(Debug, Clone)]
pub struct GestureState {
    pub pointer_x: f64,
    pub pointer_y: f64,
}

impl Default for GestureState {
    fn default() -> Self {
        Self {
            pointer_x: 500.0,
            pointer_y: 400.0,
        }
    }
}

pub struct InputListener {
    state: Rc<RefCell<GestureState>>,
    tx: Sender<DaemonEvent>,
}

impl InputListener {
    pub fn new(tx: Sender<DaemonEvent>) -> Self {
        Self {
            state: Rc::new(RefCell::new(GestureState::default())),
            tx,
        }
    }

    pub fn spawn_global_listener(&self) {
        let tx_evdev = self.tx.clone();
        let evdev_active = Arc::new(AtomicBool::new(false));
        let evdev_flag = evdev_active.clone();

        std::thread::spawn(move || {
            let _ = Self::run_evdev_loop(tx_evdev, evdev_flag);
        });

        let tx_x11 = self.tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            Self::run_x11_global_poll_loop(tx_x11, evdev_active);
        });
    }

    fn discover_event_nodes() -> Vec<PathBuf> {
        let mut nodes = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("event") {
                        nodes.push(path);
                    }
                }
            }
        }
        nodes
    }

    fn run_evdev_loop(
        tx: Sender<DaemonEvent>,
        active_flag: Arc<AtomicBool>,
    ) -> std::io::Result<()> {
        let paths = Self::discover_event_nodes();
        let mut files: Vec<File> = Vec::new();

        for p in paths {
            if let Ok(f) = File::open(&p) {
                let fd = f.as_raw_fd();
                unsafe {
                    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }
                files.push(f);
            }
        }

        if files.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "No readable /dev/input/event* nodes",
            ));
        }

        active_flag.store(true, Ordering::SeqCst);

        let mut pollfds: Vec<libc::pollfd> = files
            .iter()
            .map(|f| libc::pollfd {
                fd: f.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();

        let mut shift_down_at: Option<Instant> = None;
        let mut ctrl_down_at: Option<Instant> = None;
        let mut other_key_pressed = false;
        let mut last_trigger = Instant::now() - Duration::from_secs(5);

        let event_size = std::mem::size_of::<LinuxInputEvent>();

        loop {
            let rc =
                unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as libc::nfds_t, 50) };
            if rc > 0 {
                for pfd in &mut pollfds {
                    if (pfd.revents & libc::POLLIN) != 0 {
                        let mut ev = LinuxInputEvent {
                            tv_sec: 0,
                            tv_usec: 0,
                            type_: 0,
                            code: 0,
                            value: 0,
                        };
                        while unsafe {
                            libc::read(
                                pfd.fd,
                                (&mut ev as *mut LinuxInputEvent).cast::<libc::c_void>(),
                                event_size,
                            )
                        } == event_size as isize
                        {
                            if ev.type_ == EV_KEY {
                                let is_down = ev.value == 1;
                                let is_up = ev.value == 0;

                                match ev.code {
                                    KEY_LEFTSHIFT | KEY_RIGHTSHIFT => {
                                        if is_down {
                                            shift_down_at = Some(Instant::now());
                                            other_key_pressed = false;
                                        } else if is_up {
                                            if let Some(t0) = shift_down_at.take() {
                                                if !other_key_pressed
                                                    && t0.elapsed() < Duration::from_millis(450)
                                                    && last_trigger.elapsed()
                                                        > Duration::from_millis(200)
                                                {
                                                    last_trigger = Instant::now();
                                                    let (px, py) = Self::query_pointer_coords()
                                                        .unwrap_or((640, 420));
                                                    let _ = tx.try_send(
                                                        DaemonEvent::SwitchWheelMode(
                                                            WheelMode::ConvertFormats,
                                                        ),
                                                    );
                                                    let _ = tx.try_send(
                                                        DaemonEvent::ShowWheelAt { x: px, y: py },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    KEY_LEFTCTRL | KEY_RIGHTCTRL => {
                                        if is_down {
                                            ctrl_down_at = Some(Instant::now());
                                            other_key_pressed = false;
                                        } else if is_up {
                                            if let Some(t0) = ctrl_down_at.take() {
                                                if !other_key_pressed
                                                    && t0.elapsed() < Duration::from_millis(450)
                                                    && last_trigger.elapsed()
                                                        > Duration::from_millis(200)
                                                {
                                                    last_trigger = Instant::now();
                                                    let (px, py) = Self::query_pointer_coords()
                                                        .unwrap_or((640, 420));
                                                    let _ = tx.try_send(
                                                        DaemonEvent::SwitchWheelMode(
                                                            WheelMode::FileActions,
                                                        ),
                                                    );
                                                    let _ = tx.try_send(
                                                        DaemonEvent::ShowWheelAt { x: px, y: py },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    KEY_ESC if is_down => {
                                        other_key_pressed = true;
                                        let _ = tx.try_send(DaemonEvent::HideWheel);
                                    }
                                    BTN_RIGHT if is_down => {
                                        let _ = tx.try_send(DaemonEvent::CycleWheelMode);
                                    }
                                    _ => {
                                        if is_down {
                                            other_key_pressed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn query_pointer_coords() -> Option<(i32, i32)> {
        unsafe {
            let dpy = XOpenDisplay(std::ptr::null());
            if dpy.is_null() {
                return None;
            }
            let root = XDefaultRootWindow(dpy);
            let mut r_ret: Window = 0;
            let mut c_ret: Window = 0;
            let mut rx: libc::c_int = 0;
            let mut ry: libc::c_int = 0;
            let mut wx: libc::c_int = 0;
            let mut wy: libc::c_int = 0;
            let mut mask: libc::c_uint = 0;
            let ok = XQueryPointer(
                dpy,
                root,
                &mut r_ret,
                &mut c_ret,
                &mut rx,
                &mut ry,
                &mut wx,
                &mut wy,
                &mut mask,
            );
            if ok != 0 {
                Some((rx, ry))
            } else {
                None
            }
        }
    }

    fn run_x11_global_poll_loop(tx: Sender<DaemonEvent>, evdev_active: Arc<AtomicBool>) {
        unsafe {
            let dpy = XOpenDisplay(std::ptr::null());
            if dpy.is_null() {
                return;
            }
            let root = XDefaultRootWindow(dpy);
            let kc_shift_l = XKeysymToKeycode(dpy, XK_SHIFT_L);
            let kc_shift_r = XKeysymToKeycode(dpy, XK_SHIFT_R);
            let kc_ctrl_l = XKeysymToKeycode(dpy, XK_CONTROL_L);
            let kc_ctrl_r = XKeysymToKeycode(dpy, XK_CONTROL_R);

            let mut prev_shift = false;
            let mut prev_ctrl = false;
            let mut shift_pressed_at: Option<Instant> = None;
            let mut ctrl_pressed_at: Option<Instant> = None;

            loop {
                if evdev_active.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_secs(2));
                    continue;
                }

                let mut r_ret: Window = 0;
                let mut c_ret: Window = 0;
                let mut rx: libc::c_int = 0;
                let mut ry: libc::c_int = 0;
                let mut wx: libc::c_int = 0;
                let mut wy: libc::c_int = 0;
                let mut mask: libc::c_uint = 0;

                let ok = XQueryPointer(
                    dpy,
                    root,
                    &mut r_ret,
                    &mut c_ret,
                    &mut rx,
                    &mut ry,
                    &mut wx,
                    &mut wy,
                    &mut mask,
                );

                let mut keymap = [0i8; 32];
                XQueryKeymap(dpy, keymap.as_mut_ptr());

                let is_pressed = |kc: u8| -> bool {
                    if kc == 0 {
                        return false;
                    }
                    let byte = keymap[(kc >> 3) as usize] as u8;
                    (byte & (1 << (kc & 7))) != 0
                };

                let shift_down = (mask & SHIFT_MASK) != 0
                    || is_pressed(kc_shift_l)
                    || is_pressed(kc_shift_r);
                let ctrl_down = (mask & CONTROL_MASK) != 0
                    || is_pressed(kc_ctrl_l)
                    || is_pressed(kc_ctrl_r);

                if shift_down && !prev_shift {
                    shift_pressed_at = Some(Instant::now());
                } else if !shift_down && prev_shift {
                    if let Some(t0) = shift_pressed_at.take() {
                        if ok != 0 && t0.elapsed() < Duration::from_millis(450) {
                            let _ = tx.try_send(DaemonEvent::SwitchWheelMode(
                                WheelMode::ConvertFormats,
                            ));
                            let _ = tx.try_send(DaemonEvent::ShowWheelAt { x: rx, y: ry });
                        }
                    }
                }

                if ctrl_down && !prev_ctrl {
                    ctrl_pressed_at = Some(Instant::now());
                } else if !ctrl_down && prev_ctrl {
                    if let Some(t0) = ctrl_pressed_at.take() {
                        if ok != 0 && t0.elapsed() < Duration::from_millis(450) {
                            let _ =
                                tx.try_send(DaemonEvent::SwitchWheelMode(WheelMode::FileActions));
                            let _ = tx.try_send(DaemonEvent::ShowWheelAt { x: rx, y: ry });
                        }
                    }
                }

                prev_shift = shift_down;
                prev_ctrl = ctrl_down;

                std::thread::sleep(Duration::from_millis(16));
            }
        }
    }

    pub fn attach_to_window(&self, window: &gtk4::Window) {
        let state_motion = self.state.clone();
        let motion_ctrl = gtk4::EventControllerMotion::new();
        motion_ctrl.connect_motion(move |_, x, y| {
            let mut st = state_motion.borrow_mut();
            st.pointer_x = x;
            st.pointer_y = y;
        });
        window.add_controller(motion_ctrl);

        let tx_click = self.tx.clone();
        let click_ctrl = gtk4::GestureClick::new();
        click_ctrl.set_button(3);
        click_ctrl.connect_pressed(move |_, _, _, _| {
            let _ = tx_click.try_send(DaemonEvent::CycleWheelMode);
        });
        window.add_controller(click_ctrl);

        let tx_key = self.tx.clone();
        let key_ctrl = gtk4::EventControllerKey::new();
        key_ctrl.connect_key_pressed(move |_, keyval, _keycode, _modifier| {
            let key_name = keyval.name().map(|s| s.to_string()).unwrap_or_default();
            match key_name.as_str() {
                "Escape" => {
                    let _ = tx_key.try_send(DaemonEvent::HideWheel);
                    return glib::Propagation::Stop;
                }
                "Shift_L" | "Shift_R" => {
                    let _ =
                        tx_key.try_send(DaemonEvent::SwitchWheelMode(WheelMode::ConvertFormats));
                }
                "Control_L" | "Control_R" => {
                    let _ = tx_key.try_send(DaemonEvent::SwitchWheelMode(WheelMode::FileActions));
                }
                "Tab" | "space" => {
                    let _ = tx_key.try_send(DaemonEvent::CycleWheelMode);
                    return glib::Propagation::Stop;
                }
                _ => {}
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_ctrl);
    }
}
