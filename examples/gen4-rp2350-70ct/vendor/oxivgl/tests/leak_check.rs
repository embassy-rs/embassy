// SPDX-License-Identifier: MIT OR Apache-2.0
//! Memory leak detection tests.
//!
//! Each test forks a child process with a fresh LVGL instance and counting
//! global allocator. This gives perfect isolation — no cross-test cache
//! contamination, no warmup needed.
//!
//! The counting allocator tracks ALL malloc/free calls (Rust + LVGL C).
//! After N create/destroy iterations, net allocation must be zero.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test leak_check
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

// Tests exercise the deprecated inline style setters to verify they still work.
#![allow(deprecated)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

// ── Counting allocator ───────────────────────────────────────────────────────

/// Counting allocator — only tracks when TRACKING_ENABLED is true.
/// This allows the child process to enable tracking only during the
/// measurement window, excluding test harness overhead.
struct CountingAlloc;
static ALLOC_BALANCE: AtomicIsize = AtomicIsize::new(0);
static TRACKING_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() && TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_add(new_size as isize - layout.size() as isize, Ordering::Relaxed);
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

fn total_alloc_bytes() -> isize {
    ALLOC_BALANCE.load(Ordering::Relaxed)
}

fn start_tracking() {
    ALLOC_BALANCE.store(0, Ordering::Relaxed);
    TRACKING_ENABLED.store(true, Ordering::Relaxed);
}

fn stop_tracking() {
    TRACKING_ENABLED.store(false, Ordering::Relaxed);
}

// ── Forked test runner ───────────────────────────────────────────────────────

const MEASURE: isize = 100;
const NOISE_FLOOR: isize = 32;

/// Fork a child process, run `test_fn` in a fresh LVGL instance.
/// Uses libc fork() directly — child inherits the counting allocator
/// but gets a completely fresh LVGL via LvglDriver::init.
fn run_isolated(name: &str, test_fn: fn() -> isize) {
    use std::io::Read;
    use std::os::unix::io::FromRawFd;

    unsafe extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn fork() -> i32;
        fn _exit(status: i32) -> !;
        fn close(fd: i32) -> i32;
        fn write(fd: i32, buf: *const u8, count: usize) -> isize;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }

    let mut fds = [0i32; 2];
    assert_eq!(unsafe { pipe(fds.as_mut_ptr()) }, 0);
    let (read_fd, write_fd) = (fds[0], fds[1]);

    let pid = unsafe { fork() };
    assert!(pid >= 0, "fork failed");

    if pid == 0 {
        // ── Child: fresh LVGL, run test, write result, exit ──
        unsafe { close(read_fd) };

        // Silence child output to avoid confusing the test harness
        unsafe extern "C" { fn open(path: *const u8, flags: i32) -> i32; fn dup2(old: i32, new: i32) -> i32; }
        let devnull = unsafe { open(b"/dev/null\0".as_ptr(), 2) }; // O_RDWR=2
        if devnull >= 0 {
            unsafe { dup2(devnull, 1); dup2(devnull, 2); close(devnull); }
        }

        // Init LVGL with tracking disabled — don't count init allocations
        let _driver = oxivgl::driver::LvglDriver::init(320, 240);

        let per_iter = test_fn();

        let bytes = per_iter.to_le_bytes();
        unsafe { write(write_fd, bytes.as_ptr(), bytes.len()) };
        unsafe { close(write_fd) };
        unsafe { _exit(0) };
    }

    // ── Parent: read result, assert ──
    unsafe { close(write_fd) };

    let mut status = 0i32;
    unsafe { waitpid(pid, &mut status, 0) };
    // Check WIFEXITED && WEXITSTATUS == 0
    let exited_normally = (status & 0x7f) == 0;
    let exit_code = (status >> 8) & 0xff;
    assert!(
        exited_normally && exit_code == 0,
        "{name}: child crashed or failed (raw status {status:#x})"
    );

    let mut buf = [0u8; 8];
    let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
    file.read_exact(&mut buf).expect("failed to read from child");
    let per_iter = isize::from_le_bytes(buf);

    assert!(
        per_iter.abs() <= NOISE_FLOOR,
        "{name}: leaked {per_iter} bytes/iter (noise floor ±{NOISE_FLOOR})"
    );
}

// ── Test helpers (run inside forked child) ───────────────────────────────────
// Note: these do NOT use tests/common — each child has its own fresh LVGL
// instance created by run_isolated. The helpers are intentionally minimal.

fn screen() -> oxivgl::widgets::Screen {
    oxivgl::widgets::Screen::active().expect("no active screen")
}

fn pump_child() {
    unsafe { oxivgl_sys::lv_refr_now(core::ptr::null_mut()) };
}

/// Measure per-iteration leak for a widget test closure.
fn measure_widget(f: impl Fn(&oxivgl::widgets::Screen)) -> isize {
    let screen = screen();
    pump_child();
    start_tracking();
    let before = total_alloc_bytes();
    for _ in 0..MEASURE {
        f(&screen);
        pump_child();
    }
    let after = total_alloc_bytes();
    stop_tracking();
    (after - before) / MEASURE
}

/// Measure per-iteration leak for a pure Rust closure (no widgets).
fn measure_rust(f: impl Fn()) -> isize {
    start_tracking();
    let before = total_alloc_bytes();
    for _ in 0..MEASURE {
        f();
    }
    let after = total_alloc_bytes();
    stop_tracking();
    (after - before) / MEASURE
}

// ── Imports for test closures ────────────────────────────────────────────────

use oxivgl::snapshot::Snapshot;
use oxivgl::{
    anim::anim_path_linear,
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::{
        color_make, palette_main, props, GradDsc, GradExtend, Palette, Selector, StyleBuilder,
        TransitionDsc,
    },
    enums::{ObjFlag, ObjState, ScrollDir},
    widgets::{
        AnimImg, Arc, ArcLabel, Bar, BarMode, Button, Buttonmatrix, Calendar, CalendarDate, Canvas, Chart,
        ChartAxis, ChartType, Checkbox, Dropdown, Imagebutton, ImagebuttonState, Keyboard,
        KeyboardMode, Label, Led, Line, Menu, Msgbox, Obj, Part, Roller, RollerMode, Slider,
        Spangroup, Spinbox, Spinner, Subject, Switch, Table, Tabview, Textarea, Tileview, ValueLabel, Win,
        lv_color_t,
    },
};

// ── Declarative test macros ─────────────────────────────────────────────────

/// Generates a `#[test] fn $name()` that calls `run_isolated(label, || measure_widget(|s| { ... }))`.
macro_rules! leak_widget_test {
    ($( $name:ident, $label:expr, |$s:ident| $body:block );+ $(;)?) => {
        $(
            #[test]
            fn $name() {
                run_isolated($label, || measure_widget(|$s| $body))
            }
        )+
    };
}

/// Generates a `#[test] fn $name()` that calls `run_isolated(label, || measure_rust(|| { ... }))`.
macro_rules! leak_rust_test {
    ($( $name:ident, $label:expr, || $body:block );+ $(;)?) => {
        $(
            #[test]
            fn $name() {
                run_isolated($label, || measure_rust(|| $body))
            }
        )+
    };
}

// ── LVGL C baseline (custom — not using measure_widget/measure_rust) ────────

#[test]
fn leak_aa_lvgl_baseline() {
    run_isolated("LVGL C baseline", || {
        let screen_ptr = unsafe { oxivgl_sys::lv_screen_active() };
        start_tracking();
        let before = total_alloc_bytes();
        for _ in 0..MEASURE {
            unsafe {
                let obj = oxivgl_sys::lv_obj_create(screen_ptr);
                oxivgl_sys::lv_obj_set_size(obj, 100, 50);
                oxivgl_sys::lv_obj_delete(obj);
                oxivgl_sys::lv_refr_now(core::ptr::null_mut());
            }
        }
        let after = total_alloc_bytes();
        stop_tracking();
        (after - before) / MEASURE
    });
}

// ── Navigator toast (show/dismiss cycle) ─────────────────────────────────────

#[test]
fn leak_navigator_toast_show_dismiss() {
    // A trivial root view for the navigator stack.
    #[derive(Default)]
    struct EmptyRoot;
    impl oxivgl::view::View for EmptyRoot {
        fn create(
            &mut self,
            _container: &oxivgl::widgets::Obj<'static>,
        ) -> Result<(), oxivgl::widgets::WidgetError> {
            Ok(())
        }
    }

    // A passive toast view (one label).
    #[derive(Default)]
    struct LabelToast;
    impl oxivgl::view::View for LabelToast {
        fn create(
            &mut self,
            container: &oxivgl::widgets::Obj<'static>,
        ) -> Result<(), oxivgl::widgets::WidgetError> {
            let lbl = oxivgl::widgets::Label::new(container)?;
            lbl.text("status");
            Ok(())
        }
    }

    run_isolated("Navigator toast show/dismiss", || {
        let mut nav = oxivgl::navigator::Navigator::new();
        nav.push_root(EmptyRoot);
        pump_child();
        start_tracking();
        let before = total_alloc_bytes();
        for _ in 0..MEASURE {
            nav.show_toast(LabelToast, None);
            nav.dismiss_toast().expect("dismiss_toast");
            pump_child();
        }
        let after = total_alloc_bytes();
        stop_tracking();
        (after - before) / MEASURE
    });
}

// ── Statics referenced by test closures ─────────────────────────────────────

static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_COLOR, props::BG_OPA, props::LAST];

static LINE_POINTS: [oxivgl::widgets::lv_point_precise_t; 3] = [
    oxivgl::widgets::lv_point_precise_t { x: 0.0, y: 0.0 },
    oxivgl::widgets::lv_point_precise_t { x: 50.0, y: 30.0 },
    oxivgl::widgets::lv_point_precise_t { x: 100.0, y: 0.0 },
];

oxivgl::image_declare!(img_cogwheel_argb);

#[repr(transparent)]
struct SyncPtr(*const core::ffi::c_void);
unsafe impl Sync for SyncPtr {}

mod animimg_frames {
    unsafe extern "C" {
        #[allow(non_upper_case_globals)]
        pub static img_cogwheel_argb: oxivgl::widgets::lv_image_dsc_t;
    }
    pub static FRAMES: [super::SyncPtr; 2] = [
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
    ];
}

fn animimg_frame_ptrs() -> &'static [*const core::ffi::c_void] {
    unsafe {
        core::slice::from_raw_parts(
            animimg_frames::FRAMES.as_ptr().cast(),
            animimg_frames::FRAMES.len(),
        )
    }
}

use oxivgl::translation::{self, StaticCStr as S};

static LEAK_TRANS_LANGS: [S; 3] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL];
static LEAK_TRANS_TAGS: [S; 2] = [S::from_cstr(c"hello"), S::NULL];
static LEAK_TRANS_VALUES: [S; 2] = [S::from_cstr(c"Hello"), S::from_cstr(c"Hallo")];

// ── Pure Rust leak tests ────────────────────────────────────────────────────

/// `'static` keypad states for the KeypadIndev leak tests (LVGL stores a
/// pointer to each for the device's lifetime).
static KP_LEAK: oxivgl::indev::KeypadState = oxivgl::indev::KeypadState::new();
static KP_LEAK_EVT: oxivgl::indev::KeypadState = oxivgl::indev::KeypadState::new();

leak_rust_test! {
    leak_keypad_indev, "KeypadIndev create/drop", || {
        let kp = oxivgl::indev::KeypadIndev::new(&KP_LEAK).unwrap();
        drop(kp);
    };

    leak_keypad_event, "KeypadIndev EVENT create/drop", || {
        // create() (EVENT mode) + Drop is the allocation surface owned here;
        // send()/read() allocate nothing (fixed ring + atomics), and the
        // read-drain path is covered by the integration tests (real screen).
        let kp = oxivgl::indev::KeypadIndev::new_event(&KP_LEAK_EVT).unwrap();
        KP_LEAK_EVT.send(oxivgl::enums::Key::NEXT); // exercises the ring; no alloc
        drop(kp);
    };

    leak_style_build_drop, "Style build/drop", || {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).radius(5).border_width(2).border_color_hex(0x00FF00);
        drop(sb.build());
    };

    leak_style_with_grad_no_widget, "Style+GradDsc (no widget)", || {
        let mut grad = GradDsc::new();
        grad.init_stops(
            &[palette_main(Palette::Blue), palette_main(Palette::Red)],
            &[255, 255], &[0, 255],
        ).linear(0, 0, 100, 0, GradExtend::Pad);
        let mut sb = StyleBuilder::new();
        sb.bg_opa(255).bg_grad(grad);
        drop(sb.build());
    };

    leak_style_with_transition_no_widget, "Style+TransitionDsc (no widget)", || {
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
        drop(sb.build());
    };

    leak_translation, "Translation", || {
        translation::add_static(&LEAK_TRANS_LANGS, &LEAK_TRANS_TAGS, &LEAK_TRANS_VALUES);
        translation::set_language(c"en");
        translation::set_language(c"de");
    };

    leak_subject_int, "Subject::new_int", || {
        let subject = Subject::new_int(42);
        drop(subject);
    };

    leak_subject_group, "Subject group (3 int + 1 group)", || {
        let s0 = Subject::new_int(1);
        let s1 = Subject::new_int(2);
        let s2 = Subject::new_int(3);
        let group = Subject::new_group(&[&s0, &s1, &s2]);
        drop(group);
        drop(s2);
        drop(s1);
        drop(s0);
    };

    leak_subject_on_change, "Subject::on_change", || {
        let subject = Subject::new_int(0);
        subject.on_change(|_| {});
        subject.set_int(1);
    };

    leak_style_bg_grad_double_set, "bg_grad double-set", || {
        let mut g1 = GradDsc::new();
        g1.init_stops(
            &[palette_main(Palette::Blue), palette_main(Palette::Red)],
            &[255, 255],
            &[0, 255],
        )
        .horizontal();
        let mut g2 = GradDsc::new();
        g2.init_stops(
            &[palette_main(Palette::Green), palette_main(Palette::Yellow)],
            &[255, 255],
            &[0, 255],
        )
        .horizontal();
        let mut sb = StyleBuilder::new();
        sb.bg_opa(255).bg_grad(g1).bg_grad(g2);
        drop(sb.build());
    };

    leak_style_transition_double_set, "transition double-set", || {
        let t1 = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let t2 = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 300, 50);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(t1).transition(t2);
        drop(sb.build());
    };
}

// ── Widget leak tests ───────────────────────────────────────────────────────

leak_widget_test! {
    leak_obj_create_destroy, "Obj", |s| {
        drop(Obj::new(s).unwrap());
    };

    leak_label, "Label", |s| {
        let l = Label::new(s).unwrap(); l.text("hello world"); drop(l);
    };

    leak_button_with_label, "Button+Label", |s| {
        let btn = Button::new(s).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Click me"); drop(lbl); drop(btn);
    };

    leak_style_add_remove, "Style add/remove", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x0000FF).bg_opa(200);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        obj.remove_style_all(); drop(obj); drop(style);
    };

    leak_style_shared, "Style shared", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x123456).bg_opa(255);
        let style = sb.build();
        let o1 = Obj::new(s).unwrap(); let o2 = Obj::new(s).unwrap();
        o1.add_style(&style, Selector::DEFAULT);
        o2.add_style(&style, Selector::DEFAULT);
        drop(o1); drop(o2); drop(style);
    };

    leak_style_with_grad, "Style+GradDsc", |s| {
        let mut grad = GradDsc::new();
        grad.init_stops(
            &[palette_main(Palette::Blue), palette_main(Palette::Red)],
            &[255, 255], &[0, 255],
        ).linear(0, 0, 100, 0, GradExtend::Pad);
        let mut sb = StyleBuilder::new();
        sb.bg_opa(255).bg_grad(grad);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT); obj.size(80, 40);
        drop(obj); drop(style);
    };

    leak_style_with_transition, "Style+TransitionDsc", |s| {
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(obj); drop(style);
    };

    leak_style_drop_before_widget, "Style dropped before widget", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF00FF).bg_opa(255).radius(10);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(style); drop(obj);
    };

    leak_nested_widgets, "Nested widgets", |s| {
        let c = Obj::new(s).unwrap(); c.size(200, 200);
        let btn = Button::new(&c).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Nested"); drop(lbl); drop(btn); drop(c);
    };

    leak_arc, "Arc", |s| {
        let arc = Arc::new(s).unwrap();
        arc.set_range(100.0); arc.set_value(50.0);
        arc.set_rotation(135).set_bg_angles(0, 270);
        drop(arc);
    };

    leak_bar, "Bar", |s| {
        let bar = Bar::new(s).unwrap();
        bar.set_range_raw(0, 100); bar.set_mode(BarMode::Range);
        bar.set_value_raw(80, false); bar.set_start_value_raw(20, false);
        drop(bar);
    };

    leak_slider, "Slider", |s| {
        let sl = Slider::new(s).unwrap();
        sl.set_range(-20, 80); sl.set_value(30); drop(sl);
    };

    leak_dropdown, "Dropdown", |s| {
        let dd = Dropdown::new(s).unwrap();
        dd.set_options("Apple\nBanana\nOrange"); dd.set_selected(1); drop(dd);
    };

    leak_checkbox, "Checkbox", |s| {
        let cb = Checkbox::new(s).unwrap();
        cb.text("Accept"); cb.add_state(ObjState::CHECKED); drop(cb);
    };

    leak_roller, "Roller", |s| {
        let r = Roller::new(s).unwrap();
        r.set_options("A\nB\nC\nD", RollerMode::Normal);
        r.set_visible_row_count(3); r.set_selected(2, false); drop(r);
    };

    leak_switch, "Switch", |s| {
        let sw = Switch::new(s).unwrap();
        sw.add_state(ObjState::CHECKED); drop(sw);
    };

    leak_led, "Led", |s| {
        let led = Led::new(s).unwrap();
        led.on(); led.set_brightness(128); drop(led);
    };

    leak_line, "Line", |s| {
        let l = Line::new(s).unwrap(); l.set_points(&LINE_POINTS); drop(l);
    };

    leak_image, "Image", |s| {
        use oxivgl::widgets::Image;
        let img = Image::new(s).unwrap();
        img.set_src(img_cogwheel_argb()); drop(img);
    };

    leak_value_label, "ValueLabel", |s| {
        let mut vl = ValueLabel::new(s, "V").unwrap();
        vl.set_value(14.2).unwrap(); drop(vl);
    };

    leak_style_on_part, "Style on Part::Indicator", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x00FF00).bg_opa(200);
        let style = sb.build();
        let bar = Bar::new(s).unwrap();
        bar.set_range_raw(0, 100); bar.set_value_raw(50, false);
        bar.add_style(&style, Part::Indicator);
        drop(bar); drop(style);
    };

    leak_complex_ui, "Complex UI", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x111111).bg_opa(255).radius(8)
            .border_width(2).border_color_hex(0x00FF00).pad_all(10);
        let style = sb.build();
        let c = Obj::new(s).unwrap();
        c.add_style(&style, Selector::DEFAULT); c.size(300, 220);
        let bar = Bar::new(&c).unwrap(); bar.set_range(100.0); bar.set_value(75.0);
        let lbl = Label::new(&c).unwrap(); lbl.text("Status: OK");
        let btn = Button::new(&c).unwrap();
        let bl = Label::new(&btn).unwrap(); bl.text("Reset");
        let arc = Arc::new(&c).unwrap(); arc.set_range(100.0); arc.set_value(33.0);
        drop(bl); drop(btn); drop(arc); drop(bar); drop(lbl); drop(c); drop(style);
    };

    leak_zz_anim_start_widget_delete, "Anim start+widget delete", |s| {
        use oxivgl::anim::{anim_set_x, Anim};
        let obj = Obj::new(s).unwrap(); obj.size(100, 50);
        let mut a = Anim::new();
        a.set_var(&obj).set_values(0, 100).set_duration(500)
            .set_exec_cb(Some(anim_set_x));
        let _h = a.start();
        pump_child();
        drop(obj);
        pump_child();
    };

    leak_textarea, "Textarea", |s| {
        let ta = Textarea::new(s).unwrap();
        ta.set_one_line(true); ta.set_text("test"); ta.add_text(" more"); drop(ta);
    };

    leak_buttonmatrix, "Buttonmatrix", |s| {
        drop(Buttonmatrix::new(s).unwrap());
    };

    leak_keyboard, "Keyboard", |s| {
        let kb = Keyboard::new(s).unwrap();
        kb.set_mode(KeyboardMode::Number); drop(kb);
    };

    leak_list, "List", |s| {
        use oxivgl::widgets::List;
        let list = List::new(s).unwrap();
        list.add_text("Section");
        list.add_button(Some(&oxivgl::symbols::FILE), "Open");
        list.add_button(None, "Close");
        drop(list);
    };

    leak_menu, "Menu", |s| {
        let menu = Menu::new(s).unwrap();
        let page = menu.page_create(None);
        let cont = Menu::cont_create(&page);
        let lbl = Label::new(&cont).unwrap(); lbl.text("Item");
        menu.set_page(&page);
        drop(lbl); drop(cont); drop(page); drop(menu);
    };

    leak_msgbox, "Msgbox", |s| {
        let mbox = Msgbox::new(Some(s)).unwrap();
        mbox.add_title("Test"); mbox.add_text("Body"); mbox.add_close_button();
        drop(mbox);
    };

    leak_chart, "Chart", |s| {
        let chart = Chart::new(s).unwrap();
        chart.set_type(ChartType::Line); chart.set_point_count(5);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        let color = lv_color_t { blue: 0, green: 0, red: 255 };
        let series = chart.add_series(color, ChartAxis::PrimaryY);
        chart.set_next_value(&series, 50); chart.refresh(); drop(chart);
    };

    leak_canvas, "Canvas", |s| {
        let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
        let canvas = Canvas::new(s, buf).unwrap();
        canvas.fill_bg(color_make(100, 100, 100), 255);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(255, 0, 0));
            layer.draw_rect(&dsc, Area { x1: 5, y1: 5, x2: 45, y2: 45 });
        }
        drop(canvas);
    };

    leak_table, "Table", |s| {
        let t = Table::new(s).unwrap();
        t.set_row_count(5); t.set_column_count(2);
        for row in 0..5u32 { t.set_cell_value(row, 0, "Name"); t.set_cell_value(row, 1, "Val"); }
        drop(t);
    };

    leak_tabview, "Tabview", |s| {
        let tv = Tabview::new(s).unwrap();
        let _t1 = tv.add_tab("Alpha"); let _t2 = tv.add_tab("Beta"); drop(tv);
    };

    leak_calendar, "Calendar", |s| {
        let cal = Calendar::new(s).unwrap();
        cal.set_today_date(2024, 3, 22).set_month_shown(2024, 3);
        cal.set_highlighted_dates(&[CalendarDate::new(2024, 3, 5), CalendarDate::new(2024, 3, 15)]);
        let _hdr = cal.add_header_arrow(); drop(cal);
    };

    leak_spinner, "Spinner", |s| {
        let sp = Spinner::new(s).unwrap(); sp.set_anim_params(1000, 200); drop(sp);
    };

    leak_spinbox, "Spinbox", |s| {
        let sb = Spinbox::new(s).unwrap();
        sb.set_range(-100, 100).set_value(42).set_step(10); sb.increment(); drop(sb);
    };

    leak_spangroup, "Spangroup", |s| {
        let sg = Spangroup::new(s).unwrap(); sg.width(200);
        let span = sg.add_span().unwrap(); span.set_text(c"leak test");
        sg.refresh(); drop(sg);
    };

    leak_imagebutton, "Imagebutton", |s| {
        let btn = Imagebutton::new(s).unwrap();
        btn.set_state(ImagebuttonState::Pressed);
        btn.set_src(ImagebuttonState::Released, None, None, None); drop(btn);
    };

    leak_win, "Win", |s| {
        let win = Win::new(s).unwrap();
        let _b = win.add_button(&oxivgl::symbols::CLOSE, 40);
        let _t = win.add_title("Leak test");
        let c = win.get_content();
        let _l = Label::new(&c).unwrap();
        drop(win);
    };

    leak_tileview, "Tileview", |s| {
        let tv = Tileview::new(s).unwrap();
        let _t1 = tv.add_tile(0, 0, ScrollDir::HOR);
        let _t2 = tv.add_tile(1, 0, ScrollDir::HOR);
        drop(tv);
    };

    leak_animimg, "AnimImg", |s| {
        let a = AnimImg::new(s).unwrap();
        a.set_src(animimg_frame_ptrs()).set_duration(500).set_repeat_count(1).start();
        drop(a);
    };

    leak_scale_rotation_anim, "Scale+rotation anim", |s| {
        use oxivgl::anim::{anim_set_scale_rotation, Anim};
        use oxivgl::widgets::{Scale, ScaleMode};
        let scale = Scale::new(s).unwrap();
        scale.set_mode(ScaleMode::RoundInner)
            .set_range(0, 360)
            .set_total_tick_count(9)
            .set_major_tick_every(1)
            .set_angle_range(360)
            .set_rotation(0);
        scale.size(100, 100);
        let mut a = Anim::new();
        a.set_var(&scale)
            .set_values(0, 360)
            .set_duration(500)
            .set_exec_cb(Some(anim_set_scale_rotation));
        let _h = a.start();
        pump_child();
        drop(scale);
        pump_child();
    };

    leak_arclabel, "ArcLabel", |s| {
        let al = ArcLabel::new(s).unwrap();
        al.set_text_static(c"Leak test");
        al.set_radius(40);
        drop(al);
    };

    leak_subject_with_bound_widgets, "Subject + bound Slider + Label", |s| {
        let subject = Subject::new_int(50);
        let slider = Slider::new(s).unwrap();
        slider.bind_value(&subject);
        let label = Label::new(s).unwrap();
        label.bind_text(&subject, c"%d");
        drop(label);
        drop(slider);
        drop(subject);
    };

    leak_subject_bind_state, "Subject bind_state_if_eq", |s| {
        let subject = Subject::new_int(0);
        let obj = Obj::new(s).unwrap();
        obj.bind_state_if_eq(&subject, ObjState::DISABLED, 1);
        subject.set_int(1);
        drop(obj);
        drop(subject);
    };

    leak_subject_bind_checked, "Subject bind_checked", |s| {
        let subject = Subject::new_int(0);
        let obj = Obj::new(s).unwrap();
        obj.add_flag(ObjFlag::CHECKABLE);
        obj.bind_checked(&subject);
        subject.set_int(1);
        drop(obj);
        drop(subject);
    };

    leak_subject_bind_text_map, "Subject bind_text_map", |s| {
        let subject = Subject::new_int(0);
        let label = Label::new(s).unwrap();
        label.bind_text_map(&subject, |v| if v == 1 { "on" } else { "off" });
        subject.set_int(1);
        drop(label);
        drop(subject);
    };

    leak_remove_style_none, "remove_style(None)", |s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        pump_child();
        obj.remove_style(None, Selector::DEFAULT);
        drop(obj);
        drop(style);
    };

    leak_group, "Group", |s| {
        use oxivgl::group::Group;
        let group = Group::new().unwrap();
        let obj = Obj::new(s).unwrap();
        group.add_obj(&obj);
        drop(obj);
        drop(group);
    };

    leak_gridnav, "Gridnav", |s| {
        use oxivgl::gridnav::{gridnav_add, GridnavCtrl};
        let obj = Obj::new(s).unwrap();
        gridnav_add(&obj, GridnavCtrl::NONE);
        drop(obj);
    };

    leak_snapshot, "Snapshot", |s| {
        let obj = Obj::new(s).unwrap();
        obj.size(100, 100).center().bg_color(0xff0000).bg_opa(255);
        let _snap = Snapshot::take_widget(&obj).expect("snapshot allocation");
        pump_child();
        drop(_snap);
        drop(obj);
    };

    leak_snapshot_with_image, "Snapshot+Image", |s| {
        use oxivgl::widgets::Image;
        let obj = Obj::new(s).unwrap();
        obj.size(100, 100).center().bg_color(0xff0000).bg_opa(255);
        let snap = Snapshot::take_widget(&obj).expect("snapshot allocation");
        let img = Image::new(s).unwrap();
        img.set_src_snapshot(&snap);
        pump_child();
        // img drops before snap — correct order
        drop(img);
        drop(snap);
        drop(obj);
    };
}
