#![macro_use]

use crate::pac;
use crate::peripherals;

pub(crate) unsafe fn configure_dmamux(
    dmamux_regs: pac::dmamux::Dmamux,
    dmamux_ch_num: u8,
    request: u8,
) {
    let ch_mux_regs = dmamux_regs.ccr(dmamux_ch_num as _);
    ch_mux_regs.write(|reg| {
        reg.set_nbreq(0);
        reg.set_dmareq_id(request);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
    });
}

pub(crate) mod sealed {
    use super::*;
    pub trait MuxChannel {
        const DMAMUX_CH_NUM: u8;
        const DMAMUX_REGS: pac::dmamux::Dmamux;
    }
}

pub struct DMAMUX1;
#[cfg(rcc_h7)]
pub struct DMAMUX2;

pub trait MuxChannel: sealed::MuxChannel + super::Channel {
    type Mux;
}

pac::dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $version:ident, $channel_num:expr, {dmamux: $dmamux:ident, dmamux_channel: $dmamux_channel:expr}) => {
        impl sealed::MuxChannel for peripherals::$channel_peri {
            const DMAMUX_CH_NUM: u8 = $dmamux_channel;
            const DMAMUX_REGS: pac::dmamux::Dmamux = pac::$dmamux;
        }
        impl MuxChannel for peripherals::$channel_peri {
            type Mux = $dmamux;
        }
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    pac::peripherals! {
        (dmamux, $peri:ident) => {
            {
                pac::peripheral_rcc! {
                    ($peri, $clock:ident, $en_reg:ident, $rst_reg:ident, $en_fn:ident, $rst_fn:ident) => {
                        use crate::rcc::sealed::RccPeripheral;
                        crate::peripherals::$peri::enable()
                    };
                }
            }
        };
    }
}
