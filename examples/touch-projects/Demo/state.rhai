// Rhai: WHAT (can_tx via natives). Rust: WHEN (touch repeat, CAN RX once, idle refresh).

let ui = [];

fn init() {
    for i in button_indices() {
        ui.push(minp_in(i));
    }
}

fn cycle() {
    can_tx_clear();
    for (v, i) in ui {
        if i%2 != 0 {
            ui[i] = minp_in(i);
            if btn_held(i) {
                can_tx_on(i);
            }
        } else {
            ui[i] ^= minp_in(i);
            if ui[i] {
                can_tx_on(i);
            }
        }
    }
}
