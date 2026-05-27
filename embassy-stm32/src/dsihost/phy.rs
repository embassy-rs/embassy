//! DSIHOST Physical Layer (PHY)

use super::{DsiHost, Instance};
#[cfg(dsihost_v1)]
use crate::rcc::get_freqs;

/// Number of DSI PHY data lanes
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DsiHostPhyLanes {
    /// One data lane
    One = 0,
    /// Two data lanes
    Two = 1,
}

/// DSI PHY Configuration
pub struct DsiHostPhyConfig {
    /// Number of DSI PHY lanes
    pub lanes: DsiHostPhyLanes,

    /// Stop wait time. Minimum wait period to request a high speed transmission after the stop state
    pub stop_wait_time: u8,

    /// D-PHY automatic clock lane control. Stops providing clock automatically.
    pub acr: bool,

    /// Enable CRC RX enable
    pub crc_rx: bool,

    /// Enable ECC RX enable
    pub ecc_rx: bool,

    /// EoTp RX enable
    pub eotp_rx: bool,

    /// EoTp TX enable
    pub eotp_tx: bool,

    /// Bus Turnaround enable
    pub bta: bool,

    /// Clock high speed to low power timing
    pub clock_hs2lp: u16,

    /// Clock low power to high speed timing
    pub clock_lp2hs: u16,

    /// Data high speed to low power timing
    #[cfg(dsihost_v1)]
    pub data_hs2lp: u8,

    /// Data high speed to low power timing
    #[cfg(dsihost_u5)]
    pub data_hs2lp: u16,

    /// Data low power to high speed timing
    #[cfg(dsihost_v1)]
    pub data_lp2hs: u8,

    /// Data low power to high speed timing
    #[cfg(dsihost_u5)]
    pub data_lp2hs: u16,

    /// Data maximum read time
    pub data_mrd: u16,
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Initialize DSI D-PHY
    pub fn phy_init(&mut self, config: &DsiHostPhyConfig) {
        let lane_clock = T::frequency();

        // Timeout clock: target ~= 20 MHz. Choose ceiling below 20MHz
        const ESCAPE_TARGET_HZ: u32 = 20_000_000;
        let escape_div = ((lane_clock.0 + (ESCAPE_TARGET_HZ - 1)) / ESCAPE_TARGET_HZ) as u8; // ceil
        assert!(
            escape_div > 1 && escape_div <= 32,
            "DSI escape clock divider out of range"
        );
        let tx_escape_clock = lane_clock / escape_div;

        // Timeout clock: target ~= 1 MHz
        const TIMEOUT_TARGET_HZ: u32 = 1_000_000;
        let timeout_div = ((lane_clock.0 + (TIMEOUT_TARGET_HZ / 2)) / TIMEOUT_TARGET_HZ) as u8;
        assert!(timeout_div > 0, "DSI timeout clock divider out of range");
        let timeout_clock = lane_clock / timeout_div;

        #[cfg(feature = "defmt")]
        {
            debug!(
                "DSI lane byte clock: {} tx escape clock: {} timeout clock: {}",
                lane_clock, tx_escape_clock, timeout_clock
            );
        }

        self.lane_byte_clock = Some(lane_clock).into();
        self.tx_escape_clock = Some(tx_escape_clock).into();
        self.timeout_clock = Some(timeout_clock).into();

        self.phy_clock_enable(true);

        T::regs().clcr().modify(|w| {
            w.set_dpcc(true);
            w.set_acr(config.acr);
        });

        T::regs().pconfr().modify(|w| {
            w.set_nl(config.lanes as u8);
            w.set_sw_time(config.stop_wait_time);
        });

        T::regs().ccr().modify(|w| {
            w.set_tockdiv(timeout_div);
            w.set_txeckdiv(escape_div);
        });

        #[cfg(dsihost_v1)]
        {
            let clocks = unsafe { get_freqs() };
            let phy_clock = clocks
                .dsi_phy
                .to_hertz()
                .expect("DSI PHY clock must be enabled before init");

            // Calculate UIX4
            let uix4 = (4_000_000_000u64) / phy_clock.0 as u64;
            let uix4 = uix4.max(1).min(63) as u8;
            T::regs().wpcr0().modify(|w| {
                w.set_uix4(uix4);
            });
        }

        T::regs().pcr().modify(|w| {
            w.set_crcrxe(config.crc_rx);
            w.set_eccrxe(config.ecc_rx);
            w.set_etrxe(config.eotp_rx);
            w.set_ettxe(config.eotp_tx);
            w.set_btae(config.bta);
        });

        T::regs().cltcr().modify(|w| {
            w.set_hs2lp_time(config.clock_hs2lp);
            w.set_lp2hs_time(config.clock_lp2hs);
        });

        T::regs().dltcr().modify(|w| {
            #[cfg(dsihost_v1)]
            w.set_mrd_time(config.data_mrd);
            w.set_hs2lp_time(config.data_hs2lp);
            w.set_lp2hs_time(config.data_lp2hs);
        });
    }

    /// Enable the PHY clocks and digital domain
    fn phy_clock_enable(&mut self, enable: bool) {
        T::regs().pctlr().modify(|w| {
            // Enable D-PHY clock lane module
            w.set_cke(enable);
            // Enable digital section of D-PHY
            w.set_den(enable);
        });
    }
}
