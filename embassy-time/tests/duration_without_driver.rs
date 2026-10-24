use embassy_time::Duration;

#[test]
fn durations_work_without_a_time_driver() {
    let duration = Duration::from_millis(core::hint::black_box(10));
    assert_eq!(duration.as_millis(), 10);
    assert_eq!(format!("{duration}"), format!("{} ticks", duration.as_ticks()));
}
