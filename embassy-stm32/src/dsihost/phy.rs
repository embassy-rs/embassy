//! DSIHOST Physical Layer (PHY)

use stm32_metapac::RCC;
use stm32_metapac::rcc::vals::Dsisel;

use super::DsiHost;
use super::Instance;
use crate::rcc;

/// Select the clock mux for the DSI lane byte clock
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DsiPixelClockSource {
    /// DSI PHY Clock / 8
    DsiPhy,
    /// Main PLL2Q
    Pll2Q,
}

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

    /// Mux selection of the pixel clock between PHY DSI clock / 8 or PLL2Q
    pub pixel_clock_source: DsiPixelClockSource,

    /// Stop wait time. Minimum wait period to request a high speed transmission after the stop state
    pub stop_wait_time: u8,

    /// TX escape clock div relative to lane byte clock
    pub tx_escape_div: u8,

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
    pub data_hs2lp: u8,

    /// Data low power to high speed timing
    pub data_lp2hs: u8,

    /// Data maximum read time
    pub data_mrd: u16,
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Initialize DSI D-PHY
    pub fn phy_init(&mut self, config: &DsiHostPhyConfig) {
        let phy_clock = self
            .phy_freq
            .to_hertz()
            .expect("DSI PHY clock must be enabled before init");

        let lane_clock = match config.pixel_clock_source {
            DsiPixelClockSource::DsiPhy => {
                RCC.d1ccipr().modify(|w| w.set_dsisel(Dsisel::DSI_PHY));
                phy_clock / 8u32
            }
            DsiPixelClockSource::Pll2Q => {
                RCC.d1ccipr().modify(|w| w.set_dsisel(Dsisel::PLL2_Q));

                let clocks = unsafe { rcc::get_freqs() };
                clocks.pll2_q.to_hertz().expect("PLL2Q must be configured for DSI PHY")
            }
        };

        let tx_escape_clock = lane_clock / config.tx_escape_div;

        // Calculate UIX4
        let uix4 = (4_000_000_000u64) / phy_clock.0 as u64;
        let uix4 = uix4.max(1).min(63) as u8;

        #[cfg(feature = "defmt")]
        {
            debug!(
                "DSI lane byte clock: {} tx escape clock: {}, uix4: {}",
                lane_clock, tx_escape_clock, uix4
            );
        }

        self.lane_byte_clock = Some(lane_clock).into();
        self.tx_escape_clock = Some(tx_escape_clock).into();

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
            w.set_txeckdiv(config.tx_escape_div);
        });

        T::regs().wpcr0().modify(|w| {
            w.set_uix4(uix4);
        });

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
