use crate::pac;

pub use pac::resets::regs::Peripherals;

pub const ALL_PERIPHERALS: Peripherals = Peripherals(0x01ffffff);

pub struct Resets {}

impl Resets {
    pub fn new() -> Self {
        Self {}
    }

    pub fn reset(&self, peris: Peripherals) {
        unsafe {
            pac::RESETS.reset().write_value(peris);
        }
    }

    pub fn unreset_wait(&self, peris: Peripherals) {
        unsafe {
            // TODO use the "atomic clear" register version
            pac::RESETS
                .reset()
                .modify(|v| *v = Peripherals(v.0 & !peris.0));
            while ((!pac::RESETS.reset_done().read().0) & peris.0) != 0 {}
        }
    }
}
