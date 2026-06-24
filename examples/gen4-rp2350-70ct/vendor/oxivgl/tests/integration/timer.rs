use crate::common::{fresh_screen, pump};
use oxivgl::timer::Timer;

#[test]
fn timer_create_and_triggered() {
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    // Not yet triggered before first pump (long period).
    assert!(!timer.triggered());
    // Force fire via ready(), then pump to execute.
    timer.ready();
    pump();
    assert!(timer.triggered());
    // triggered() clears flag — second call returns false.
    assert!(!timer.triggered());
}

#[test]
fn timer_set_period_and_repeat_count() {
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    timer.set_period(100_000).set_repeat_count(1);
    // Force fire and verify triggered.
    timer.ready();
    pump();
    assert!(timer.triggered());
}

#[test]
fn timer_ready_fires_immediately() {
    let _screen = fresh_screen();
    let timer = Timer::new(999_999).unwrap(); // very long period
    timer.ready(); // force fire on next tick
    pump();
    assert!(timer.triggered());
}

#[test]
fn timer_pause_resume() {
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    timer.pause();
    timer.ready(); // mark ready, but paused — should not fire
    pump();
    assert!(!timer.triggered(), "paused timer should not fire");
    timer.resume();
    timer.ready();
    pump();
    assert!(timer.triggered(), "resumed timer should fire");
}

#[test]
fn timer_drop_cleans_up() {
    let _screen = fresh_screen();
    let timer = Timer::new(10).unwrap();
    drop(timer); // should not panic or leak
    pump();
}
