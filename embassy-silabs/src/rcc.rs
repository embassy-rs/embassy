//! Reset and Clock Management Unit (CMU).

use critical_section::CriticalSection;

use crate::pac::CMU;

pub(crate) fn init_clocks(_cs: CriticalSection) {
    CMU.clken0().modify(|w| {
        w.set_gpio(true);
        w.set_timer0(true);
    });
}
