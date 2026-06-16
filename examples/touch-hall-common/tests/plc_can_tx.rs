//! Host-only PLC smoke test (Rhai → Rust can_tx buffer).

use touch_hall_common::can_bridge::release_payload;
use touch_hall_common::rhai_state::Plc;

#[test]
fn plc_cycle_produces_release_when_idle() {
    let mut plc = Plc::new().expect("PLC should load DemoHost state.rhai");
    plc.cycle().expect("cycle");
    assert_eq!(plc.can_tx_payload(), release_payload());
}
