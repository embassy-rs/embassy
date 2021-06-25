use crate::pac;

pub use pac::resets::regs::Peripherals;

pub const ALL_PERIPHERALS: Peripherals = Peripherals(0x01ffffff);

pub unsafe fn reset(peris: Peripherals) {
    pac::RESETS.reset().write_value(peris);
}

pub unsafe fn unreset_wait(peris: Peripherals) {
    // TODO use the "atomic clear" register version
    pac::RESETS
        .reset()
        .modify(|v| *v = Peripherals(v.0 & !peris.0));
    while ((!pac::RESETS.reset_done().read().0) & peris.0) != 0 {}
}
