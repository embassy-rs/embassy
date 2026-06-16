// Rhai: WHAT (can_tx via natives). Rust: WHEN (touch repeat, CAN RX on change, idle refresh).
//
// can_tx_set(i, v) — button index i → set that button's CAN byte to v
// can_tx_on(i)       — button index i → set one-hot bit (OR, for multiple buttons)
// can_tx(i)          — read byte for button index i

let ui = [];

fn init() {
    for i in button_indices() {
        ui.push(false);
    }
}

fn cycle() {
    can_tx_clear();
    for (v, i) in ui {
        if i%2 == 0 {
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
