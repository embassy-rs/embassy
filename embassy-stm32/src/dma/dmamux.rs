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

pub(crate) mod dmamux_sealed {
    pub trait MuxChannel {}
}

/// DMAMUX1 instance.
pub struct DMAMUX1;
/// DMAMUX2 instance.
#[cfg(stm32h7)]
pub struct DMAMUX2;

/// DMAMUX channel trait.
pub trait MuxChannel: dmamux_sealed::MuxChannel {
    /// DMAMUX instance this channel is on.
    type Mux;
}

macro_rules! dmamux_channel_impl {
    ($channel_peri:ident, $dmamux:ident) => {
        impl crate::dma::dmamux_sealed::MuxChannel for crate::peripherals::$channel_peri {}
        impl crate::dma::MuxChannel for crate::peripherals::$channel_peri {
            type Mux = crate::dma::$dmamux;
        }
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(_cs: critical_section::CriticalSection) {
    crate::_generated::init_dmamux();
}
