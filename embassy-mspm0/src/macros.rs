#![macro_use]

macro_rules! new_pin {
    ($name: ident, $pf_type: expr) => {{
        let pin = $name;
        pin.set_as_pf(pin.pf_num(), $pf_type);
        Some(pin.into())
    }};
}
