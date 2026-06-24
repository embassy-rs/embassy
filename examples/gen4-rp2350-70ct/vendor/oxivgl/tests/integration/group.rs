use crate::common::{fresh_screen, pump};

use oxivgl::gridnav::{gridnav_add, GridnavCtrl};
use oxivgl::group::{group_get_default, group_remove_obj, Group};
use oxivgl::widgets::Obj;

// ── Group ─────────────────────────────────────────────────────────────────────

#[test]
fn group_create_and_drop() {
    let _screen = fresh_screen();
    let group = Group::new().unwrap();
    pump();
    drop(group);
}

#[test]
fn group_add_obj() {
    let screen = fresh_screen();
    let group = Group::new().unwrap();
    let obj = Obj::new(&screen).unwrap();
    group.add_obj(&obj);
    pump();
}

#[test]
fn group_set_default_and_get() {
    let _screen = fresh_screen();
    let group = Group::new().unwrap();
    group.set_default();
    pump();
    assert!(group_get_default().is_some());
}

#[test]
fn group_remove_obj_no_crash() {
    let screen = fresh_screen();
    let group = Group::new().unwrap();
    let obj = Obj::new(&screen).unwrap();
    group.add_obj(&obj);
    pump();
    group_remove_obj(&obj);
    pump();
}

#[test]
fn group_assign_to_keyboard_indevs() {
    let _screen = fresh_screen();
    let group = Group::new().unwrap();
    group.assign_to_keyboard_indevs();
    pump();
}

// ── Gridnav ───────────────────────────────────────────────────────────────────

#[test]
fn gridnav_add_to_container() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    gridnav_add(&obj, GridnavCtrl::NONE);
    pump();
}

#[test]
fn gridnav_add_rollover() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    gridnav_add(&obj, GridnavCtrl::ROLLOVER);
    pump();
}
