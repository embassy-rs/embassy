//! DSIHOST Modes

use super::panel::DsiPanel;
use super::{DsiHost, Error, Instance};
use crate::ltdc::PolarityActive;
use crate::peripherals::LTDC;
use crate::rcc::SealedRccPeripheral;

/// DSI Color Mode.
///
/// The DSI wrapper LTDC interface supports different
/// color encodings for RGB565 and RGB666.
///
/// See RM0339 Table 281
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum DsiColor {
    /// RGB565 Config 1
    ///
    /// D[0..5] = B[0..5]
    /// D[5..11] = G[0..6]
    /// D[11..16] = B[0..5]
    Rgb565Config1 = 0,

    /// RGB565 Config 2
    ///
    /// D[0..5] = B[0..5]
    /// D[8..14] = G[0..6]
    /// D[16..21] = B[0..5]
    Rgb565Config2 = 1,

    /// RGB565 Config 2
    ///
    /// D[1..6] = B[0..5]
    /// D[8..14] = G[0..6]
    /// D[17..22] = B[0..5]
    Rgb565Config3 = 2,

    /// RGB666 Config 1
    ///
    /// D[0..6] = B[0..6]
    /// D[6..12] = G[0..6]
    /// D[12..18] = B[0..6]
    Rgb666Config1 = 3,

    /// RGB666 Config 2
    ///
    /// D[0..6] = B[0..6]
    /// D[8..14] = G[0..6]
    /// D[16..22] = B[0..6]
    Rgb666Config2 = 4,

    /// RGB888
    ///
    /// D[0..9] = B[0..8]
    /// D[9..16] = G[0..8]
    /// D[16..24] = B[0..8]
    Rgb888 = 5,
}

/// DSI LTDC Refresh Mode
pub enum DsiLtdcRefreshMode {
    /// Refresh is manually initiated by calling [`DsiHost::ltds_refresh`]
    Manual,
    /// Frames are automatically requested from LTDC from tearing events
    Automatic,
}

/// Tear event source selection
pub enum DsiTearEventSource {
    /// Tear event over DSI link
    Dsi,
    /// Tear event over external GPIO pin
    Gpio,
}

/// DSI Video Burst Mode
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum DsiVideoMode {
    /// Sync pulse mode
    SyncPulse = 0,
    /// Sync event mode
    SyncEvent = 1,
    /// Burst mode
    Burst = 2,
}

/// DSI Mode
pub enum DsiHostMode {
    /// Video Mode
    Video(DsiVideoConfig),

    /// Adapted Command Mode
    AdaptedCommand(DsiCommandConfig),
}

/// DSI Video configuration
pub struct DsiVideoConfig {
    /// Burst mode
    pub mode: DsiVideoMode,

    /// Color coding
    pub color: DsiColor,

    /// Virtual Channel ID
    pub channel: u8,

    /// Bus turnaround acknowledge enable
    pub bta: bool,

    /// Low power vertical sync active
    pub lpvsa: bool,

    /// Low power vertical back porch
    pub lpvbp: bool,

    /// Low power vertical front porch
    pub lpvfp: bool,

    /// Low power vertical active
    pub lpva: bool,

    /// Low power horizontal back porch
    pub lphbp: bool,

    /// Low power horizontal front porch
    pub lphfp: bool,

    /// Lower power command transmission enable
    pub lpcmd: bool,
}

impl Default for DsiVideoConfig {
    fn default() -> Self {
        Self {
            mode: DsiVideoMode::Burst,
            color: DsiColor::Rgb888,
            channel: 0,
            bta: false,
            lpvsa: true,
            lpvbp: true,
            lpvfp: true,
            lpva: true,
            lphbp: true,
            lphfp: true,
            lpcmd: true,
        }
    }
}

/// DSI Adapted Command Mode configuration
pub struct DsiCommandConfig {
    /// Color mode
    pub color: DsiColor,

    /// Virtual Channel ID
    pub channel: u8,

    /// Refresh mode
    pub refresh: DsiLtdcRefreshMode,

    /// LTDC VSYNC polarity
    pub vsync_polarity: PolarityActive,

    /// Tear event source
    pub te_source: DsiTearEventSource,

    /// Tear event polarity
    pub te_polarity: PolarityActive,
}

impl Default for DsiCommandConfig {
    fn default() -> Self {
        Self {
            color: DsiColor::Rgb888,
            channel: 0,
            refresh: DsiLtdcRefreshMode::Manual,
            vsync_polarity: PolarityActive::ActiveLow,
            te_source: DsiTearEventSource::Dsi,
            te_polarity: PolarityActive::ActiveLow,
        }
    }
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Set the DSI mode
    pub fn set_mode<Panel: DsiPanel>(&mut self, mode: &DsiHostMode) -> Result<(), Error> {
        T::regs().lpcr().modify(|w| {
            let dep = match Panel::DATA_ENABLE_POLARITY {
                PolarityActive::ActiveLow => true,
                PolarityActive::ActiveHigh => false,
            };
            w.set_dep(dep);

            let vsp = match Panel::VSYNC_POLARITY {
                PolarityActive::ActiveLow => true,
                PolarityActive::ActiveHigh => false,
            };
            w.set_vsp(vsp);

            let hsp = match Panel::HSYNC_POLARITY {
                PolarityActive::ActiveLow => true,
                PolarityActive::ActiveHigh => false,
            };
            w.set_hsp(hsp);
        });

        T::regs().lpmcr().modify(|w| {
            w.set_lpsize(Panel::LP_MAX_PACKET_SIZE);
            w.set_vlpsize(Panel::VACT_MAX_PACKET_SIZE);
        });

        match mode {
            DsiHostMode::Video(config) => {
                self.set_color::<Panel>(config.color);
                // Set the Virtual Channel ID for LTDC
                T::regs().lvcidr().modify(|w| w.set_vcid(config.channel));

                self.set_video_config::<Panel>(config);
            }
            DsiHostMode::AdaptedCommand(config) => {
                self.set_color::<Panel>(config.color);

                // Set the Virtual Channel ID for LTDC
                T::regs().lvcidr().modify(|w| w.set_vcid(config.channel));

                T::regs().mcr().modify(|w| w.set_cmdm(true));
                T::regs().wcfgr().modify(|w| w.set_dsim(true));

                // Maximum size for memory write command
                T::regs().lccr().modify(|w| w.set_cmdsize(Panel::ACTIVE_WIDTH));

                // Send command to enable tearing output
                self.write_cmd(0, 0x35, &[0x0])?;

                let ar = match config.refresh {
                    DsiLtdcRefreshMode::Manual => false,
                    DsiLtdcRefreshMode::Automatic => true,
                };

                T::regs().wcfgr().modify(|w| w.set_ar(ar));

                // Tearing acknowledgement request enable
                T::regs().cmcr().modify(|w| w.set_teare(true));
                T::regs().pcr().modify(|w| w.set_btae(true));

                T::regs().wcfgr().modify(|w| {
                    w.set_tesrc(match config.te_source {
                        DsiTearEventSource::Dsi => false,
                        DsiTearEventSource::Gpio => true,
                    });
                    w.set_tepol(match config.te_polarity {
                        PolarityActive::ActiveLow => false,
                        PolarityActive::ActiveHigh => true,
                    });
                    w.set_vspol(match config.vsync_polarity {
                        PolarityActive::ActiveLow => false,
                        PolarityActive::ActiveHigh => true,
                    });
                });
            }
        }

        Ok(())
    }

    /// Manual refresh of a frame from LTDC
    pub fn ltdc_refresh(&self) {
        T::regs().wcr().modify(|w| w.set_ltdcen(true));
    }

    fn set_color<Panel: DsiPanel>(&mut self, color: DsiColor) {
        T::regs().lcolcr().modify(|w| {
            w.set_colc(color as u8);
            w.set_lpe(match color {
                DsiColor::Rgb666Config1 | DsiColor::Rgb666Config2 => Panel::LOOSELY_PACKED,
                _ => false,
            });
        });

        // Set color coding in DSI wrapper COLMUX
        T::regs().wcfgr().modify(|w| {
            w.set_colmux(color as u8);
        });
    }

    /// Set video mode registers from a [`DsiVideoConfig`]
    fn set_video_config<Panel: DsiPanel>(&mut self, config: &DsiVideoConfig) {
        let lane_byte_clock = self.lane_byte_clock.to_hertz().expect("DSI lane byte clock");
        let ltdc_clock = LTDC::frequency();

        T::regs().mcr().modify(|w| w.set_cmdm(false));
        T::regs().wcfgr().modify(|w| w.set_dsim(false));

        T::regs().vmcr().modify(|w| {
            w.set_vmt(config.mode as u8);
            w.set_lpvsae(config.lpvsa);
            w.set_lpvbpe(config.lpvbp);
            w.set_lpvfpe(config.lpvfp);
            w.set_lpvae(config.lpva);
            w.set_lphbpe(config.lphbp);
            w.set_lphfpe(config.lphfp);
            w.set_fbtaae(config.bta);
            w.set_lpce(config.lpcmd);
        });

        match config.mode {
            DsiVideoMode::SyncPulse | DsiVideoMode::SyncEvent => {
                // Size of each chunk
                T::regs().vpcr().modify(|w| w.set_vpsize(Panel::ACTIVE_WIDTH / 4));
                // Number of packets to send per video line
                T::regs().vccr().modify(|w| w.set_numc(4));
            }
            DsiVideoMode::Burst => {
                // In burst mode, set VPCR.VPSIZE to the active line period in pixels
                T::regs().vpcr().modify(|w| w.set_vpsize(Panel::ACTIVE_WIDTH));
                T::regs().vccr().modify(|w| w.set_numc(0));
            }
        }

        T::regs().vnpcr().modify(|w| w.set_npsize(Panel::NULL_PACKET_SIZE));

        let lane_byte_clock_khz = lane_byte_clock.0 / 1000;
        let ltdc_clock_khz = ltdc_clock.0 / 1000;

        let hline = Panel::HLINE_TOTAL as u32 * lane_byte_clock_khz / ltdc_clock_khz;
        let hsa = Panel::HSYNC as u32 * lane_byte_clock_khz / ltdc_clock_khz;
        let hbp = Panel::HBP as u32 * lane_byte_clock_khz / ltdc_clock_khz;

        #[cfg(feature = "defmt")]
        {
            debug!("LTDC clock: {}", ltdc_clock);
            debug!("DSI hline: {} hsa: {} hbp: {}", hline, hsa, hbp);
        }

        T::regs().vlcr().modify(|w| w.set_hline(hline as u16));
        T::regs().vhsacr().modify(|w| w.set_hsa(hsa as u16));
        T::regs().vhbpcr().modify(|w| w.set_hbp(hbp as u16));

        // Vertical timing configuration
        T::regs().vvsacr().modify(|w| w.set_vsa(Panel::VSYNC));
        T::regs().vvbpcr().modify(|w| w.set_vbp(Panel::VBP));
        T::regs().vvfpcr().modify(|w| w.set_vfp(Panel::VFP));
        T::regs().vvacr().modify(|w| w.set_va(Panel::ACTIVE_HEIGHT));
    }

    /// Eneable video mode pattern generator.
    /// The DSI host must be initialized in video more before enabling the pattern generator
    pub fn enable_pattern_generator(&mut self, enable: bool) {
        T::regs().vmcr().modify(|w| {
            w.set_pgm(false);
            w.set_pgo(false);
            w.set_pge(enable);
        });
    }

    /// Debug dump registers to defmt
    #[cfg(feature = "defmt")]
    pub fn debug_registers() {
        debug!("{}", T::regs().mcr().read());
        debug!("{}", T::regs().wcfgr().read());
        debug!("{}", T::regs().vmcr().read());
        debug!("{}", T::regs().wcr().read());
        debug!("{}", T::regs().vpcr().read());
        debug!("{}", T::regs().vccr().read());
        debug!("{}", T::regs().vnpcr().read());
        debug!("{}", T::regs().vhsacr().read());
        debug!("{}", T::regs().vhbpcr().read());
        debug!("{}", T::regs().vlcr().read());
        debug!("{}", T::regs().vvsacr().read());
        debug!("{}", T::regs().vvbpcr().read());
        debug!("{}", T::regs().vvfpcr().read());
        debug!("{}", T::regs().vvacr().read());
        debug!("{}", T::regs().lcolcr().read());
        debug!("{}", T::regs().gpsr().read());
        debug!("{}", T::regs().wisr().read());
        debug!("{}", T::regs().isr0().read());
        debug!("{}", T::regs().isr1().read());
        debug!("{}", T::regs().cr().read());
    }
}
