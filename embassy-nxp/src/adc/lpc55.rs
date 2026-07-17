#![macro_use]

pub(crate) fn init() {
    // TODO
}

pub trait Instance {
    // TODO
}

pub trait AdcPin<T: Instance> {
    // TODO
    fn channel(&self) -> u8;
}

pub enum Resolution {
    Bits16,
    Bits12,
}

pub enum Averaging {
    None,
    Samples2,
    Samples4,
    Samples8,
    Samples16,
    Samples32,
    Samples64,
    Samples128,
}

pub struct Config {
    pub resolution: Resolution,
    pub averaging: Averaging,
}

pub struct Adc<'d, T: Instance> {
    // TODO
}
