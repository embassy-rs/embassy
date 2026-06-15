// Rhai: WHAT (can_tx, ui). Rust: WHEN (touch repeat, CAN RX on change, idle refresh).
//
// btn_held(i)  — touch press
// minp_in(i)   — CAN RX edge
// can_tx       — -1 none, 0..254 button, 255 release

let ui = [];
let can_tx = CAN_TX_RELEASE;

fn init() {
    for i in button_indices() {
        ui.push(false);
    }
}

fn cycle() {
    can_tx = CAN_TX_RELEASE;
    for (v, i) in ui {
        ui[i] ^= minp_in(i);
        if ui[i] {
            can_tx = i;
        }
    }
}
