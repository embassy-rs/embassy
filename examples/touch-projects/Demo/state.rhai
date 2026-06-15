// Button highlight state machine for oxivgl_touch_can.
//
// Required:
//   fn button_active(index) -> bool
//
// Optional:
//   fn on_can_rx(id) { ... }   // called after each CAN frame is stored
//
// CAN helpers:
//   can_len(id)                latest payload length for that standard ID
//   can_byte(id, byte_index)   raw byte (0 if missing)
//   can_bit(id, byte_index, bit_index)
//
// minp helpers (from can_config.json):
//   minp_can_id(index), minp_byte(index), minp_bit(index), minp_active_val(index)
//   button_count(), minp_count()

fn button_active(index) {
    let active_val = minp_active_val(index);
    if active_val == 0 {
        return false;
    }

    let id = minp_can_id(index);
    let byte = minp_byte(index);
    let bit = minp_bit(index);
    if can_len(id) <= byte {
        return false;
    }

    can_bit(id, byte, bit)
}
