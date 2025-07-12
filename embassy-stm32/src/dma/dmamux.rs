#![macro_use]

use crate::pac;

pub(crate) struct DmamuxInfo {
    pub(crate) mux: pac::dmamux::Dmamux,
    pub(crate) num: usize,
}

pub(crate) fn configure_dmamux(info: &DmamuxInfo, request: u8) {
    let ch_mux_regs = info.mux.ccr(info.num);
    ch_mux_regs.write(|reg| {
        reg.set_nbreq(0);
        reg.set_dmareq_id(request);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
    });
}

/// safety: must be called only once
pub(crate) unsafe fn init(_cs: critical_section::CriticalSection) {
    crate::_generated::init_dmamux();
}
