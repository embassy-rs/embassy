use crate::pac;
use embassy::util::Steal;
use pac::rcc::{self, vals};

#[derive(Default)]
pub struct Config {}

pub unsafe fn init(config: Config) {
    let rcc = pac::RCC;

    let enabled = vals::Iophen::ENABLED;
    rcc.iopenr().write(|w| {
        w.set_iopaen(enabled);
        w.set_iopben(enabled);
        w.set_iopcen(enabled);
        w.set_iopden(enabled);
        w.set_iopeen(enabled);
        w.set_iophen(enabled);
    });
}
