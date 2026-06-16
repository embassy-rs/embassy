// Rhai: WHAT (can_tx, ui). Rust: WHEN (touch repeat, CAN RX once, idle refresh).

let ui = [];
let can_tx = CAN_TX_RELEASE;

fn init() {
    for i in button_indices() {
        ui.push(minp_in(i));
    }
}

fn cycle() {
    can_tx = CAN_TX_RELEASE;
    for (v, i) in ui {
        if i%2 != 0 {
            ui[i] = minp_in(i);
            if btn_held(i) {
                can_tx = i;
            }
        } else {
            ui[i] ^= minp_in(i);
            if ui[i] {
                can_tx = i;
            }
        }
    }
}
