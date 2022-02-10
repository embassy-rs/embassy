//! Sub-GHz radio operating in the 150 - 960 MHz ISM band
//!
//! ## LoRa user notice
//!
//! The Sub-GHz radio may have an undocumented erratum, see this ST community
//! post for more information: [link]
//!
//! [link]: https://community.st.com/s/question/0D53W00000hR8kpSAC/stm32wl55-erratum-clairification
//!
//! NOTE: This HAL is based on https://github.com/newAM/stm32wl-hal, but adopted for use with the stm32-metapac
//! and SPI HALs.

mod bit_sync;
mod cad_params;
mod calibrate;
mod fallback_mode;
mod hse_trim;
mod irq;
mod lora_sync_word;
mod mod_params;
mod ocp;
mod op_error;
mod pa_config;
mod packet_params;
mod packet_status;
mod packet_type;
mod pkt_ctrl;
mod pmode;
mod pwr_ctrl;
mod reg_mode;
mod rf_frequency;
mod rx_timeout_stop;
mod sleep_cfg;
mod smps;
mod standby_clk;
mod stats;
mod status;
mod tcxo_mode;
mod timeout;
mod tx_params;
mod value_error;

pub use bit_sync::BitSync;
pub use cad_params::{CadParams, ExitMode, NbCadSymbol};
pub use calibrate::{Calibrate, CalibrateImage};
pub use fallback_mode::FallbackMode;
pub use hse_trim::HseTrim;
pub use irq::{CfgIrq, Irq, IrqLine};
pub use lora_sync_word::LoRaSyncWord;
pub use mod_params::BpskModParams;
pub use mod_params::{CodingRate, LoRaBandwidth, LoRaModParams, SpreadingFactor};
pub use mod_params::{FskBandwidth, FskBitrate, FskFdev, FskModParams, FskPulseShape};
pub use ocp::Ocp;
pub use op_error::OpError;
pub use pa_config::{PaConfig, PaSel};
pub use packet_params::{
    AddrComp, BpskPacketParams, CrcType, GenericPacketParams, HeaderType, LoRaPacketParams,
    PreambleDetection,
};
pub use packet_status::{FskPacketStatus, LoRaPacketStatus};
pub use packet_type::PacketType;
pub use pkt_ctrl::{InfSeqSel, PktCtrl};
pub use pmode::PMode;
pub use pwr_ctrl::{CurrentLim, PwrCtrl};
pub use reg_mode::RegMode;
pub use rf_frequency::RfFreq;
pub use rx_timeout_stop::RxTimeoutStop;
pub use sleep_cfg::{SleepCfg, Startup};
pub use smps::SmpsDrv;
pub use standby_clk::StandbyClk;
pub use stats::{FskStats, LoRaStats, Stats};
pub use status::{CmdStatus, Status, StatusMode};
pub use tcxo_mode::{TcxoMode, TcxoTrim};
pub use timeout::Timeout;
pub use tx_params::{RampTime, TxParams};
pub use value_error::ValueError;

use embassy_hal_common::ratio::Ratio;

use crate::{
    dma::NoDma,
    pac,
    peripherals::SUBGHZSPI,
    rcc::sealed::RccPeripheral,
    spi::{BitOrder, Config as SpiConfig, MisoPin, MosiPin, SckPin, Spi, MODE_0},
    time::Hertz,
};
use embassy::util::Unborrow;

/// Passthrough for SPI errors (for now)
pub type Error = crate::spi::Error;

struct Nss {
    _priv: (),
}

impl Nss {
    pub fn new() -> Nss {
        Self::clear();
        Nss { _priv: () }
    }

    /// Clear NSS, enabling SPI transactions
    #[inline(always)]
    fn clear() {
        let pwr = pac::PWR;
        unsafe {
            pwr.subghzspicr()
                .modify(|w| w.set_nss(pac::pwr::vals::Nss::LOW));
        }
    }

    /// Set NSS, disabling SPI transactions
    #[inline(always)]
    fn set() {
        let pwr = pac::PWR;
        unsafe {
            pwr.subghzspicr()
                .modify(|w| w.set_nss(pac::pwr::vals::Nss::HIGH));
        }
    }
}

impl Drop for Nss {
    fn drop(&mut self) {
        Self::set()
    }
}

/// Wakeup the radio from sleep mode.
///
/// # Safety
///
/// 1. This must not be called when the SubGHz radio is in use.
/// 2. This must not be called when the SubGHz SPI bus is in use.
///
/// # Example
///
/// See [`SubGhz::set_sleep`]
#[inline]
unsafe fn wakeup() {
    Nss::clear();
    // RM0453 rev 2 page 171 section 5.7.2 "Sleep mode"
    // on a firmware request via the sub-GHz radio SPI NSS signal
    // (keeping sub-GHz radio SPI NSS low for at least 20 μs)
    //
    // I have found this to be a more reliable mechanism for ensuring NSS is
    // pulled low for long enough to wake the radio.
    while rfbusys() {}
    Nss::set();
}

/// Returns `true` if the radio is busy.
///
/// This may not be set immediately after NSS going low.
///
/// See RM0461 Rev 4 section 5.3 page 181 "Radio busy management" for more
/// details.
#[inline]
fn rfbusys() -> bool {
    // safety: atmoic read with no side-effects
    //unsafe { (*pac::PWR::ptr()).sr2.read().rfbusys().is_busy() }
    let pwr = pac::PWR;
    unsafe { pwr.sr2().read().rfbusys() == pac::pwr::vals::Rfbusys::BUSY }
}

/*
/// Returns `true` if the radio is busy or NSS is low.
///
/// See RM0461 Rev 4 section 5.3 page 181 "Radio busy management" for more
/// details.
#[inline]
fn rfbusyms() -> bool {
    let pwr = pac::PWR;
    unsafe { pwr.sr2().read().rfbusyms() == pac::pwr::vals::Rfbusyms::BUSY }
}
*/

/// Sub-GHz radio peripheral
pub struct SubGhz<'d, Tx, Rx> {
    spi: Spi<'d, SUBGHZSPI, Tx, Rx>,
}

impl<'d, Tx, Rx> SubGhz<'d, Tx, Rx> {
    fn pulse_radio_reset() {
        let rcc = pac::RCC;
        unsafe {
            rcc.csr().modify(|w| w.set_rfrst(true));
            rcc.csr().modify(|w| w.set_rfrst(false));
        }
    }

    // TODO: This should be replaced with async handling based on IRQ
    fn poll_not_busy(&self) {
        let mut count: u32 = 1_000_000;
        while rfbusys() {
            count -= 1;
            if count == 0 {
                let pwr = pac::PWR;
                unsafe {
                    panic!(
                        "rfbusys timeout pwr.sr2=0x{:X} pwr.subghzspicr=0x{:X} pwr.cr1=0x{:X}",
                        pwr.sr2().read().0,
                        pwr.subghzspicr().read().0,
                        pwr.cr1().read().0
                    );
                }
            }
        }
    }

    /// Create a new sub-GHz radio driver from a peripheral.
    ///
    /// This will reset the radio and the SPI bus, and enable the peripheral
    /// clock.
    pub fn new(
        peri: impl Unborrow<Target = SUBGHZSPI> + 'd,
        sck: impl Unborrow<Target = impl SckPin<SUBGHZSPI>> + 'd,
        mosi: impl Unborrow<Target = impl MosiPin<SUBGHZSPI>> + 'd,
        miso: impl Unborrow<Target = impl MisoPin<SUBGHZSPI>> + 'd,
        txdma: impl Unborrow<Target = Tx> + 'd,
        rxdma: impl Unborrow<Target = Rx> + 'd,
    ) -> Self {
        Self::pulse_radio_reset();

        // see RM0453 rev 1 section 7.2.13 page 291
        // The SUBGHZSPI_SCK frequency is obtained by PCLK3 divided by two.
        // The SUBGHZSPI_SCK clock maximum speed must not exceed 16 MHz.
        let clk = Hertz(core::cmp::min(SUBGHZSPI::frequency().0 / 2, 16_000_000));
        let mut config = SpiConfig::default();
        config.mode = MODE_0;
        config.bit_order = BitOrder::MsbFirst;
        let spi = Spi::new(peri, sck, mosi, miso, txdma, rxdma, clk, config);

        unsafe { wakeup() };

        SubGhz { spi }
    }

    pub fn is_busy(&mut self) -> bool {
        rfbusys()
    }

    pub fn reset(&mut self) {
        Self::pulse_radio_reset();
    }
}

impl<'d> SubGhz<'d, NoDma, NoDma> {
    fn read(&mut self, opcode: OpCode, data: &mut [u8]) -> Result<(), Error> {
        self.poll_not_busy();
        {
            let _nss: Nss = Nss::new();
            self.spi.blocking_write(&[opcode as u8])?;
            self.spi.blocking_transfer_in_place(data)?;
        }
        self.poll_not_busy();
        Ok(())
    }

    /// Read one byte from the sub-Ghz radio.
    fn read_1(&mut self, opcode: OpCode) -> Result<u8, Error> {
        let mut buf: [u8; 1] = [0; 1];
        self.read(opcode, &mut buf)?;
        Ok(buf[0])
    }

    /// Read a fixed number of bytes from the sub-Ghz radio.
    fn read_n<const N: usize>(&mut self, opcode: OpCode) -> Result<[u8; N], Error> {
        let mut buf: [u8; N] = [0; N];
        self.read(opcode, &mut buf)?;
        Ok(buf)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.poll_not_busy();
        {
            let _nss: Nss = Nss::new();
            self.spi.blocking_write(data)?;
        }
        self.poll_not_busy();
        Ok(())
    }

    pub fn write_buffer(&mut self, offset: u8, data: &[u8]) -> Result<(), Error> {
        self.poll_not_busy();
        {
            let _nss: Nss = Nss::new();
            self.spi
                .blocking_write(&[OpCode::WriteBuffer as u8, offset])?;
            self.spi.blocking_write(data)?;
        }
        self.poll_not_busy();

        Ok(())
    }

    /// Read the radio buffer at the given offset.
    ///
    /// The offset and length of a received packet is provided by
    /// [`rx_buffer_status`](Self::rx_buffer_status).
    pub fn read_buffer(&mut self, offset: u8, buf: &mut [u8]) -> Result<Status, Error> {
        let mut status_buf: [u8; 1] = [0];

        self.poll_not_busy();
        {
            let _nss: Nss = Nss::new();
            self.spi
                .blocking_write(&[OpCode::ReadBuffer as u8, offset])?;
            self.spi.blocking_transfer_in_place(&mut status_buf)?;
            self.spi.blocking_transfer_in_place(buf)?;
        }
        self.poll_not_busy();

        Ok(status_buf[0].into())
    }
}

// helper to pack register writes into a single buffer to avoid multiple DMA
// transfers
macro_rules! wr_reg {
    [$reg:ident, $($data:expr),+] => {
        &[
            OpCode::WriteRegister as u8,
            Register::$reg.address().to_be_bytes()[0],
            Register::$reg.address().to_be_bytes()[1],
            $($data),+
        ]
    };
}

// 5.8.2
/// Register access
impl<'d> SubGhz<'d, NoDma, NoDma> {
    // register write with variable length data
    fn write_register(&mut self, register: Register, data: &[u8]) -> Result<(), Error> {
        let addr: [u8; 2] = register.address().to_be_bytes();

        self.poll_not_busy();
        {
            let _nss: Nss = Nss::new();
            self.spi
                .blocking_write(&[OpCode::WriteRegister as u8, addr[0], addr[1]])?;
            self.spi.blocking_write(data)?;
        }
        self.poll_not_busy();

        Ok(())
    }

    /// Set the LoRa bit synchronization.
    pub fn set_bit_sync(&mut self, bs: BitSync) -> Result<(), Error> {
        self.write(wr_reg![GBSYNC, bs.as_bits()])
    }

    /// Set the generic packet control register.
    pub fn set_pkt_ctrl(&mut self, pkt_ctrl: PktCtrl) -> Result<(), Error> {
        self.write(wr_reg![GPKTCTL1A, pkt_ctrl.as_bits()])
    }

    /// Set the initial value for generic packet whitening.
    ///
    /// This sets the first 8 bits, the 9th bit is set with
    /// [`set_pkt_ctrl`](Self::set_pkt_ctrl).
    pub fn set_init_whitening(&mut self, init: u8) -> Result<(), Error> {
        self.write(wr_reg![GWHITEINIRL, init])
    }

    /// Set the initial value for generic packet CRC polynomial.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// sg.set_crc_polynomial(0x1D0F)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_crc_polynomial(&mut self, polynomial: u16) -> Result<(), Error> {
        let bytes: [u8; 2] = polynomial.to_be_bytes();
        self.write(wr_reg![GCRCINIRH, bytes[0], bytes[1]])
    }

    /// Set the generic packet CRC polynomial.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// sg.set_initial_crc_polynomial(0x1021)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_initial_crc_polynomial(&mut self, polynomial: u16) -> Result<(), Error> {
        let bytes: [u8; 2] = polynomial.to_be_bytes();
        self.write(wr_reg![GCRCPOLRH, bytes[0], bytes[1]])
    }

    /// Set the synchronization word registers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// const SYNC_WORD: [u8; 8] = [0x79, 0x80, 0x0C, 0xC0, 0x29, 0x95, 0xF8, 0x4A];
    ///
    /// sg.set_sync_word(&SYNC_WORD)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_sync_word(&mut self, sync_word: &[u8; 8]) -> Result<(), Error> {
        self.write_register(Register::GSYNC7, sync_word)
    }

    /// Set the LoRa synchronization word registers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{LoRaSyncWord, PacketType};
    ///
    /// sg.set_packet_type(PacketType::LoRa)?;
    /// sg.set_lora_sync_word(LoRaSyncWord::Public)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_lora_sync_word(&mut self, sync_word: LoRaSyncWord) -> Result<(), Error> {
        let bytes: [u8; 2] = sync_word.bytes();
        self.write(wr_reg![LSYNCH, bytes[0], bytes[1]])
    }

    /// Set the RX gain control.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PMode;
    ///
    /// sg.set_rx_gain(PMode::Boost)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_rx_gain(&mut self, pmode: PMode) -> Result<(), Error> {
        self.write(wr_reg![RXGAINC, pmode as u8])
    }

    /// Set the power amplifier over current protection.
    ///
    /// # Example
    ///
    /// Maximum 60mA for LP PA mode.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Ocp;
    ///
    /// sg.set_pa_ocp(Ocp::Max60m)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// Maximum 60mA for HP PA mode.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Ocp;
    ///
    /// sg.set_pa_ocp(Ocp::Max140m)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_pa_ocp(&mut self, ocp: Ocp) -> Result<(), Error> {
        self.write(wr_reg![PAOCP, ocp as u8])
    }

    /// Set the HSE32 crystal OSC_IN load capaitor trimming.
    ///
    /// # Example
    ///
    /// Set the trim to the lowest value.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::HseTrim;
    ///
    /// sg.set_hse_in_trim(HseTrim::MIN)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_hse_in_trim(&mut self, trim: HseTrim) -> Result<(), Error> {
        self.write(wr_reg![HSEINTRIM, trim.into()])
    }

    /// Set the HSE32 crystal OSC_OUT load capaitor trimming.
    ///
    /// # Example
    ///
    /// Set the trim to the lowest value.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::HseTrim;
    ///
    /// sg.set_hse_out_trim(HseTrim::MIN)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_hse_out_trim(&mut self, trim: HseTrim) -> Result<(), Error> {
        self.write(wr_reg![HSEOUTTRIM, trim.into()])
    }

    /// Set the SMPS clock detection enabled.
    ///
    /// SMPS clock detection must be enabled fore enabling the SMPS.
    pub fn set_smps_clock_det_en(&mut self, en: bool) -> Result<(), Error> {
        self.write(wr_reg![SMPSC0, (en as u8) << 6])
    }

    /// Set the power current limiting.
    pub fn set_pwr_ctrl(&mut self, pwr_ctrl: PwrCtrl) -> Result<(), Error> {
        self.write(wr_reg![PC, pwr_ctrl.as_bits()])
    }

    /// Set the maximum SMPS drive capability.
    pub fn set_smps_drv(&mut self, drv: SmpsDrv) -> Result<(), Error> {
        self.write(wr_reg![SMPSC2, (drv as u8) << 1])
    }
}

// 5.8.3
/// Operating mode commands
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Put the radio into sleep mode.
    ///
    /// This command is only accepted in standby mode.
    /// The cfg argument allows some optional functions to be maintained
    /// in sleep mode.
    ///
    /// # Safety
    ///
    /// 1. After the `set_sleep` command, the sub-GHz radio NSS must not go low
    ///    for 500 μs.
    ///    No reason is provided, the reference manual (RM0453 rev 2) simply
    ///    says "you must".
    /// 2. The radio cannot be used while in sleep mode.
    /// 3. The radio must be woken up with [`wakeup`] before resuming use.
    ///
    /// # Example
    ///
    /// Put the radio into sleep mode.
    ///
    /// ```no_run
    /// # let dp = unsafe { embassy_stm32::pac::Peripherals::steal() };
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::{
    ///     subghz::{wakeup, SleepCfg, StandbyClk},
    /// };
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// unsafe { sg.set_sleep(SleepCfg::default())? };
    /// embassy::time::Timer::after(embassy::time::Duration::from_micros(500)).await;
    /// unsafe { wakeup() };
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub unsafe fn set_sleep(&mut self, cfg: SleepCfg) -> Result<(), Error> {
        self.write(&[OpCode::SetSleep as u8, u8::from(cfg)])
    }

    /// Put the radio into standby mode.
    ///
    /// # Examples
    ///
    /// Put the radio into standby mode using the RC 13MHz clock.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::StandbyClk;
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// Put the radio into standby mode using the HSE32 clock.
    ///
    /// ```no_run
    /// # let mut dp = unsafe { embassy_stm32::pac::Peripherals::steal() };
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::StandbyClk;
    ///
    /// dp.RCC
    ///     .cr
    ///     .modify(|_, w| w.hseon().enabled().hsebyppwr().vddtcxo());
    /// while dp.RCC.cr.read().hserdy().is_not_ready() {}
    ///
    /// sg.set_standby(StandbyClk::Hse)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_standby(&mut self, standby_clk: StandbyClk) -> Result<(), Error> {
        self.write(&[OpCode::SetStandby as u8, u8::from(standby_clk)])
    }

    /// Put the subghz radio into frequency synthesis mode.
    ///
    /// The RF-PLL frequency must be set with [`set_rf_frequency`] before using
    /// this command.
    ///
    /// Check the datasheet for more information, this is a test command but
    /// I honestly do not see any use for it.  Please update this description
    /// if you know more than I do.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::RfFreq;
    ///
    /// sg.set_rf_frequency(&RfFreq::from_frequency(915_000_000))?;
    /// sg.set_fs()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`set_rf_frequency`]: crate::subghz::SubGhz::set_rf_frequency
    pub fn set_fs(&mut self) -> Result<(), Error> {
        self.write(&[OpCode::SetFs.into()])
    }

    /// Set the sub-GHz radio in TX mode.
    ///
    /// # Example
    ///
    /// Transmit with no timeout.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Timeout;
    ///
    /// sg.set_tx(Timeout::DISABLED)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tx(&mut self, timeout: Timeout) -> Result<(), Error> {
        let tobits: u32 = timeout.into_bits();
        self.write(&[
            OpCode::SetTx.into(),
            (tobits >> 16) as u8,
            (tobits >> 8) as u8,
            tobits as u8,
        ])
    }

    /// Set the sub-GHz radio in RX mode.
    ///
    /// # Example
    ///
    /// Receive with a 1 second timeout.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use core::time::Duration;
    /// use embassy_stm32::subghz::Timeout;
    ///
    /// sg.set_rx(Timeout::from_duration_sat(Duration::from_secs(1)))?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_rx(&mut self, timeout: Timeout) -> Result<(), Error> {
        let tobits: u32 = timeout.into_bits();
        self.write(&[
            OpCode::SetRx.into(),
            (tobits >> 16) as u8,
            (tobits >> 8) as u8,
            tobits as u8,
        ])
    }

    /// Allows selection of the receiver event which stops the RX timeout timer.
    ///
    /// # Example
    ///
    /// Set the RX timeout timer to stop on preamble detection.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::RxTimeoutStop;
    ///
    /// sg.set_rx_timeout_stop(RxTimeoutStop::Preamble)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_rx_timeout_stop(&mut self, rx_timeout_stop: RxTimeoutStop) -> Result<(), Error> {
        self.write(&[
            OpCode::SetStopRxTimerOnPreamble.into(),
            rx_timeout_stop.into(),
        ])
    }

    /// Put the radio in non-continuous RX mode.
    ///
    /// This command must be sent in Standby mode.
    /// This command is only functional with FSK and LoRa packet type.
    ///
    /// The following steps are performed:
    /// 1. Save sub-GHz radio configuration.
    /// 2. Enter Receive mode and listen for a preamble for the specified `rx_period`.
    /// 3. Upon the detection of a preamble, the `rx_period` timeout is stopped
    ///    and restarted with the value 2 x `rx_period` + `sleep_period`.
    ///    During this new period, the sub-GHz radio looks for the detection of
    ///    a synchronization word when in (G)FSK modulation mode,
    ///    or a header when in LoRa modulation mode.
    /// 4. If no packet is received during the listen period defined by
    ///    2 x `rx_period` + `sleep_period`, the sleep mode is entered for a
    ///    duration of `sleep_period`. At the end of the receive period,
    ///    the sub-GHz radio takes some time to save the context before starting
    ///    the sleep period.
    /// 5. After the sleep period, a new listening period is automatically
    ///    started. The sub-GHz radio restores the sub-GHz radio configuration
    ///    and continuous with step 2.
    ///
    /// The listening mode is terminated in one of the following cases:
    /// * if a packet is received during the listening period: the sub-GHz radio
    ///   issues a [`RxDone`] interrupt and enters standby mode.
    /// * if [`set_standby`] is sent during the listening period or after the
    ///   sub-GHz has been requested to exit sleep mode by sub-GHz radio SPI NSS
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use core::time::Duration;
    /// use embassy_stm32::subghz::{StandbyClk, Timeout};
    ///
    /// const RX_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_millis(100));
    /// const SLEEP_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_secs(1));
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// sg.set_rx_duty_cycle(RX_PERIOD, SLEEP_PERIOD)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`RxDone`]: crate::subghz::Irq::RxDone
    /// [`set_rf_frequency`]: crate::subghz::SubGhz::set_rf_frequency
    /// [`set_standby`]: crate::subghz::SubGhz::set_standby
    pub fn set_rx_duty_cycle(
        &mut self,
        rx_period: Timeout,
        sleep_period: Timeout,
    ) -> Result<(), Error> {
        let rx_period_bits: u32 = rx_period.into_bits();
        let sleep_period_bits: u32 = sleep_period.into_bits();
        self.write(&[
            OpCode::SetRxDutyCycle.into(),
            (rx_period_bits >> 16) as u8,
            (rx_period_bits >> 8) as u8,
            rx_period_bits as u8,
            (sleep_period_bits >> 16) as u8,
            (sleep_period_bits >> 8) as u8,
            sleep_period_bits as u8,
        ])
    }

    /// Channel Activity Detection (CAD) with LoRa packets.
    ///
    /// The channel activity detection (CAD) is a specific LoRa operation mode,
    /// where the sub-GHz radio searches for a LoRa radio signal.
    /// After the search is completed, the Standby mode is automatically
    /// entered, CAD is done and IRQ is generated.
    /// When a LoRa radio signal is detected, the CAD detected IRQ is also
    /// generated.
    ///
    /// The length of the search must be configured with [`set_cad_params`]
    /// prior to calling `set_cad`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use core::time::Duration;
    /// use embassy_stm32::subghz::{CadParams, ExitMode, NbCadSymbol, StandbyClk, Timeout};
    ///
    /// const RX_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_millis(100));
    /// const SLEEP_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_secs(1));
    /// const CAD_PARAMS: CadParams = CadParams::new()
    ///     .set_num_symbol(NbCadSymbol::S4)
    ///     .set_det_peak(0x18)
    ///     .set_det_min(0x10)
    ///     .set_exit_mode(ExitMode::Standby);
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// sg.set_cad_params(&CAD_PARAMS)?;
    /// sg.set_cad()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`set_cad_params`]: crate::subghz::SubGhz::set_cad_params
    pub fn set_cad(&mut self) -> Result<(), Error> {
        self.write(&[OpCode::SetCad.into()])
    }

    /// Generate a continuous transmit tone at the RF-PLL frequency.
    ///
    /// The sub-GHz radio remains in continuous transmit tone mode until a mode
    /// configuration command is received.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// sg.set_tx_continuous_wave()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tx_continuous_wave(&mut self) -> Result<(), Error> {
        self.write(&[OpCode::SetTxContinuousWave as u8])
    }

    /// Generate an infinite preamble at the RF-PLL frequency.
    ///
    /// The preamble is an alternating 0s and 1s sequence in generic (G)FSK and
    /// (G)MSK modulations.
    /// The preamble is symbol 0 in LoRa modulation.
    /// The sub-GHz radio remains in infinite preamble mode until a mode
    /// configuration command is received.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// sg.set_tx_continuous_preamble()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tx_continuous_preamble(&mut self) -> Result<(), Error> {
        self.write(&[OpCode::SetTxContinuousPreamble as u8])
    }
}

// 5.8.4
/// Radio configuration commands
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Set the packet type (modulation scheme).
    ///
    /// # Examples
    ///
    /// FSK (frequency shift keying):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PacketType;
    ///
    /// sg.set_packet_type(PacketType::Fsk)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// LoRa (long range):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PacketType;
    ///
    /// sg.set_packet_type(PacketType::LoRa)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// BPSK (binary phase shift keying):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PacketType;
    ///
    /// sg.set_packet_type(PacketType::Bpsk)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// MSK (minimum shift keying):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PacketType;
    ///
    /// sg.set_packet_type(PacketType::Msk)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_packet_type(&mut self, packet_type: PacketType) -> Result<(), Error> {
        self.write(&[OpCode::SetPacketType as u8, packet_type as u8])
    }

    /// Get the packet type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::PacketType;
    ///
    /// sg.set_packet_type(PacketType::LoRa)?;
    /// assert_eq!(sg.packet_type()?, Ok(PacketType::LoRa));
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn packet_type(&mut self) -> Result<Result<PacketType, u8>, Error> {
        let pkt_type: [u8; 2] = self.read_n(OpCode::GetPacketType)?;
        Ok(PacketType::from_raw(pkt_type[1]))
    }

    /// Set the radio carrier frequency.
    ///
    /// # Example
    ///
    /// Set the frequency to 915MHz (Australia and North America).
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::RfFreq;
    ///
    /// sg.set_rf_frequency(&RfFreq::F915)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_rf_frequency(&mut self, freq: &RfFreq) -> Result<(), Error> {
        self.write(freq.as_slice())
    }

    /// Set the transmit output power and the PA ramp-up time.
    ///
    /// # Example
    ///
    /// Set the output power to +10 dBm (low power mode) and a ramp up time of
    /// 40 microseconds.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{PaConfig, PaSel, RampTime, TxParams};
    ///
    /// const TX_PARAMS: TxParams = TxParams::new()
    ///     .set_ramp_time(RampTime::Micros40)
    ///     .set_power(0x0D);
    /// const PA_CONFIG: PaConfig = PaConfig::new()
    ///     .set_pa(PaSel::Lp)
    ///     .set_pa_duty_cycle(0x1)
    ///     .set_hp_max(0x0);
    ///
    /// sg.set_pa_config(&PA_CONFIG)?;
    /// sg.set_tx_params(&TX_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tx_params(&mut self, params: &TxParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Power amplifier configuation.
    ///
    /// Used to customize the maximum output power and efficiency.
    ///
    /// # Example
    ///
    /// Set the output power to +22 dBm (high power mode) and a ramp up time of
    /// 200 microseconds.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{PaConfig, PaSel, RampTime, TxParams};
    ///
    /// const TX_PARAMS: TxParams = TxParams::new()
    ///     .set_ramp_time(RampTime::Micros200)
    ///     .set_power(0x16);
    /// const PA_CONFIG: PaConfig = PaConfig::new()
    ///     .set_pa(PaSel::Hp)
    ///     .set_pa_duty_cycle(0x4)
    ///     .set_hp_max(0x7);
    ///
    /// sg.set_pa_config(&PA_CONFIG)?;
    /// sg.set_tx_params(&TX_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_pa_config(&mut self, pa_config: &PaConfig) -> Result<(), Error> {
        self.write(pa_config.as_slice())
    }

    /// Operating mode to enter after a successful packet transmission or
    /// packet reception.
    ///
    /// # Example
    ///
    /// Set the fallback mode to standby mode.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::FallbackMode;
    ///
    /// sg.set_tx_rx_fallback_mode(FallbackMode::Standby)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tx_rx_fallback_mode(&mut self, fm: FallbackMode) -> Result<(), Error> {
        self.write(&[OpCode::SetTxRxFallbackMode as u8, fm as u8])
    }

    /// Set channel activity detection (CAD) parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use core::time::Duration;
    /// use embassy_stm32::subghz::{CadParams, ExitMode, NbCadSymbol, StandbyClk, Timeout};
    ///
    /// const RX_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_millis(100));
    /// const SLEEP_PERIOD: Timeout = Timeout::from_duration_sat(Duration::from_secs(1));
    /// const CAD_PARAMS: CadParams = CadParams::new()
    ///     .set_num_symbol(NbCadSymbol::S4)
    ///     .set_det_peak(0x18)
    ///     .set_det_min(0x10)
    ///     .set_exit_mode(ExitMode::Standby);
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// sg.set_cad_params(&CAD_PARAMS)?;
    /// sg.set_cad()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_cad_params(&mut self, params: &CadParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the data buffer base address for the packet handling in TX and RX.
    ///
    /// There is a 256B TX buffer and a 256B RX buffer.
    /// These buffers are not memory mapped, they are accessed via the
    /// [`read_buffer`] and [`write_buffer`] methods.
    ///
    /// # Example
    ///
    /// Set the TX and RX buffer base to the start.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// sg.set_buffer_base_address(0, 0)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`read_buffer`]: SubGhz::read_buffer
    /// [`write_buffer`]: SubGhz::write_buffer
    pub fn set_buffer_base_address(&mut self, tx: u8, rx: u8) -> Result<(), Error> {
        self.write(&[OpCode::SetBufferBaseAddress as u8, tx, rx])
    }

    /// Set the (G)FSK modulation parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{
    ///     FskBandwidth, FskBitrate, FskFdev, FskModParams, FskPulseShape, PacketType,
    /// };
    ///
    /// const BITRATE: FskBitrate = FskBitrate::from_bps(32_000);
    /// const PULSE_SHAPE: FskPulseShape = FskPulseShape::Bt03;
    /// const BW: FskBandwidth = FskBandwidth::Bw9;
    /// const FDEV: FskFdev = FskFdev::from_hertz(31_250);
    ///
    /// const MOD_PARAMS: FskModParams = FskModParams::new()
    ///     .set_bitrate(BITRATE)
    ///     .set_pulse_shape(PULSE_SHAPE)
    ///     .set_bandwidth(BW)
    ///     .set_fdev(FDEV);
    ///
    /// sg.set_packet_type(PacketType::Fsk)?;
    /// sg.set_fsk_mod_params(&MOD_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_fsk_mod_params(&mut self, params: &FskModParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the LoRa modulation parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{
    ///     CodingRate, LoRaBandwidth, LoRaModParams, PacketType, SpreadingFactor,
    /// };
    ///
    /// const MOD_PARAMS: LoRaModParams = LoRaModParams::new()
    ///     .set_sf(SpreadingFactor::Sf7)
    ///     .set_bw(LoRaBandwidth::Bw125)
    ///     .set_cr(CodingRate::Cr45)
    ///     .set_ldro_en(false);
    ///
    /// sg.set_packet_type(PacketType::LoRa)?;
    /// sg.set_lora_mod_params(&MOD_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_lora_mod_params(&mut self, params: &LoRaModParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the BPSK modulation parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{BpskModParams, FskBitrate, PacketType};
    ///
    /// const MOD_PARAMS: BpskModParams = BpskModParams::new().set_bitrate(FskBitrate::from_bps(600));
    ///
    /// sg.set_packet_type(PacketType::Bpsk)?;
    /// sg.set_bpsk_mod_params(&MOD_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_bpsk_mod_params(&mut self, params: &BpskModParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the generic (FSK) packet parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{
    ///     AddrComp, CrcType, GenericPacketParams, HeaderType, PacketType, PreambleDetection,
    /// };
    ///
    /// const PKT_PARAMS: GenericPacketParams = GenericPacketParams::new()
    ///     .set_preamble_len(8)
    ///     .set_preamble_detection(PreambleDetection::Disabled)
    ///     .set_sync_word_len(2)
    ///     .set_addr_comp(AddrComp::Disabled)
    ///     .set_header_type(HeaderType::Fixed)
    ///     .set_payload_len(128)
    ///     .set_crc_type(CrcType::Byte2)
    ///     .set_whitening_enable(true);
    ///
    /// sg.set_packet_type(PacketType::Fsk)?;
    /// sg.set_packet_params(&PKT_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_packet_params(&mut self, params: &GenericPacketParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the BPSK packet parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{BpskPacketParams, PacketType};
    ///
    /// sg.set_packet_type(PacketType::Bpsk)?;
    /// sg.set_bpsk_packet_params(&BpskPacketParams::new().set_payload_len(64))?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_bpsk_packet_params(&mut self, params: &BpskPacketParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the LoRa packet parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{HeaderType, LoRaPacketParams, PacketType};
    ///
    /// const PKT_PARAMS: LoRaPacketParams = LoRaPacketParams::new()
    ///     .set_preamble_len(5 * 8)
    ///     .set_header_type(HeaderType::Fixed)
    ///     .set_payload_len(64)
    ///     .set_crc_en(true)
    ///     .set_invert_iq(true);
    ///
    /// sg.set_packet_type(PacketType::LoRa)?;
    /// sg.set_lora_packet_params(&PKT_PARAMS)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_lora_packet_params(&mut self, params: &LoRaPacketParams) -> Result<(), Error> {
        self.write(params.as_slice())
    }

    /// Set the number of LoRa symbols to be received before starting the
    /// reception of a LoRa packet.
    ///
    /// Packet reception is started after `n` + 1 symbols are detected.
    ///
    /// # Example
    ///
    /// Start reception after a single LoRa word is detected
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    ///
    /// // ... setup the radio for LoRa RX
    ///
    /// sg.set_lora_symb_timeout(0)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_lora_symb_timeout(&mut self, n: u8) -> Result<(), Error> {
        self.write(&[OpCode::SetLoRaSymbTimeout.into(), n])
    }
}

// 5.8.5
/// Communication status and information commands
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Get the radio status.
    ///
    /// The hardware (or documentation) appears to have many bugs where this
    /// will return reserved values.
    /// See this thread in the ST community for details: [link]
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Status;
    ///
    /// let status: Status = sg.status()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [link]: https://community.st.com/s/question/0D53W00000hR9GQSA0/stm32wl55-getstatus-command-returns-reserved-cmdstatus
    pub fn status(&mut self) -> Result<Status, Error> {
        Ok(self.read_1(OpCode::GetStatus)?.into())
    }

    /// Get the RX buffer status.
    ///
    /// The return tuple is (status, payload_length, buffer_pointer).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CmdStatus, Timeout};
    ///
    /// sg.set_rx(Timeout::DISABLED)?;
    /// loop {
    ///     let (status, len, ptr) = sg.rx_buffer_status()?;
    ///
    ///     if status.cmd() == Ok(CmdStatus::Avaliable) {
    ///         let mut buf: [u8; 256] = [0; 256];
    ///         let data: &mut [u8] = &mut buf[..usize::from(len)];
    ///         sg.read_buffer(ptr, data)?;
    ///         // ... do things with the data
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn rx_buffer_status(&mut self) -> Result<(Status, u8, u8), Error> {
        let data: [u8; 3] = self.read_n(OpCode::GetRxBufferStatus)?;
        Ok((data[0].into(), data[1], data[2]))
    }

    /// Returns information on the last received (G)FSK packet.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fmt::Write;
    /// # let mut uart = String::new();
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CmdStatus, Timeout};
    ///
    /// sg.set_rx(Timeout::DISABLED)?;
    /// loop {
    ///     let pkt_status = sg.fsk_packet_status()?;
    ///
    ///     if pkt_status.status().cmd() == Ok(CmdStatus::Avaliable) {
    ///         let rssi = pkt_status.rssi_avg();
    ///         writeln!(&mut uart, "Avg RSSI: {} dBm", rssi);
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn fsk_packet_status(&mut self) -> Result<FskPacketStatus, Error> {
        Ok(FskPacketStatus::from(self.read_n(OpCode::GetPacketStatus)?))
    }

    /// Returns information on the last received LoRa packet.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fmt::Write;
    /// # let mut uart = String::new();
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CmdStatus, Timeout};
    ///
    /// sg.set_rx(Timeout::DISABLED)?;
    /// loop {
    ///     let pkt_status = sg.lora_packet_status()?;
    ///
    ///     if pkt_status.status().cmd() == Ok(CmdStatus::Avaliable) {
    ///         let snr = pkt_status.snr_pkt();
    ///         writeln!(&mut uart, "SNR: {} dB", snr);
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn lora_packet_status(&mut self) -> Result<LoRaPacketStatus, Error> {
        Ok(LoRaPacketStatus::from(
            self.read_n(OpCode::GetPacketStatus)?,
        ))
    }

    /// Get the instantaneous signal strength during packet reception.
    ///
    /// The units are in dbm.
    ///
    /// # Example
    ///
    /// Log the instantaneous signal strength to UART.
    ///
    /// ```no_run
    /// # use std::fmt::Write;
    /// # let mut uart = String::new();
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CmdStatus, Timeout};
    ///
    /// sg.set_rx(Timeout::DISABLED)?;
    /// let (_, rssi) = sg.rssi_inst()?;
    /// writeln!(&mut uart, "RSSI: {} dBm", rssi);
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn rssi_inst(&mut self) -> Result<(Status, Ratio<i16>), Error> {
        let data: [u8; 2] = self.read_n(OpCode::GetRssiInst)?;
        let status: Status = data[0].into();
        let rssi: Ratio<i16> = Ratio::new_raw(i16::from(data[1]), -2);

        Ok((status, rssi))
    }

    /// (G)FSK packet stats.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{FskStats, Stats};
    ///
    /// let stats: Stats<FskStats> = sg.fsk_stats()?;
    /// // ... use stats
    /// sg.reset_stats()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn fsk_stats(&mut self) -> Result<Stats<FskStats>, Error> {
        let data: [u8; 7] = self.read_n(OpCode::GetStats)?;
        Ok(Stats::from_raw_fsk(data))
    }

    /// LoRa packet stats.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{LoRaStats, Stats};
    ///
    /// let stats: Stats<LoRaStats> = sg.lora_stats()?;
    /// // ... use stats
    /// sg.reset_stats()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn lora_stats(&mut self) -> Result<Stats<LoRaStats>, Error> {
        let data: [u8; 7] = self.read_n(OpCode::GetStats)?;
        Ok(Stats::from_raw_lora(data))
    }

    /// Reset the stats as reported in [`lora_stats`] and [`fsk_stats`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    ///
    /// sg.reset_stats()?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`lora_stats`]: crate::subghz::SubGhz::lora_stats
    /// [`fsk_stats`]: crate::subghz::SubGhz::fsk_stats
    pub fn reset_stats(&mut self) -> Result<(), Error> {
        const RESET_STATS: [u8; 7] = [0x00; 7];
        self.write(&RESET_STATS)
    }
}

// 5.8.6
/// IRQ commands
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Set the interrupt configuration.
    ///
    /// # Example
    ///
    /// Enable TX and timeout interrupts.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CfgIrq, Irq};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable_all(Irq::TxDone)
    ///     .irq_enable_all(Irq::Timeout);
    /// sg.set_irq_cfg(&IRQ_CFG)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_irq_cfg(&mut self, cfg: &CfgIrq) -> Result<(), Error> {
        self.write(cfg.as_slice())
    }

    /// Get the IRQ status.
    ///
    /// # Example
    ///
    /// Wait for TX to complete or timeout.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Irq;
    ///
    /// loop {
    ///     let (_, irq_status) = sg.irq_status()?;
    ///     sg.clear_irq_status(irq_status)?;
    ///     if irq_status & Irq::TxDone.mask() != 0 {
    ///         // handle TX done
    ///         break;
    ///     }
    ///     if irq_status & Irq::Timeout.mask() != 0 {
    ///         // handle timeout
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn irq_status(&mut self) -> Result<(Status, u16), Error> {
        let data: [u8; 3] = self.read_n(OpCode::GetIrqStatus)?;
        let irq_status: u16 = u16::from_be_bytes([data[1], data[2]]);
        Ok((data[0].into(), irq_status))
    }

    /// Clear the IRQ status.
    ///
    /// # Example
    ///
    /// Clear the [`TxDone`] and [`RxDone`] interrupts.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::Irq;
    ///
    /// sg.clear_irq_status(Irq::TxDone.mask() | Irq::RxDone.mask())?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`TxDone`]: crate::subghz::Irq::TxDone
    /// [`RxDone`]: crate::subghz::Irq::RxDone
    pub fn clear_irq_status(&mut self, mask: u16) -> Result<(), Error> {
        self.write(&[OpCode::ClrIrqStatus as u8, (mask >> 8) as u8, mask as u8])
    }
}

// 5.8.7
/// Miscellaneous commands
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Calibrate one or several blocks at any time when in standby mode.
    ///
    /// The blocks to calibrate are defined by `cal` argument.
    /// When the calibration is ongoing, BUSY is set.
    /// A falling edge on BUSY indicates the end of all enabled calibrations.
    ///
    /// This function will not poll for BUSY.
    ///
    /// # Example
    ///
    /// Calibrate the RC 13 MHz and PLL.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{Calibrate, StandbyClk, SubGhz};
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// sg.calibrate(Calibrate::Rc13M.mask() | Calibrate::Pll.mask())?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn calibrate(&mut self, cal: u8) -> Result<(), Error> {
        // bit 7 is reserved and must be kept at reset value.
        self.write(&[OpCode::Calibrate as u8, cal & 0x7F])
    }

    /// Calibrate the image at the given frequencies.
    ///
    /// Requires the radio to be in standby mode.
    ///
    /// # Example
    ///
    /// Calibrate the image for the 430 - 440 MHz ISM band.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{CalibrateImage, StandbyClk};
    ///
    /// sg.set_standby(StandbyClk::Rc)?;
    /// sg.calibrate_image(CalibrateImage::ISM_430_440)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn calibrate_image(&mut self, cal: CalibrateImage) -> Result<(), Error> {
        self.write(&[OpCode::CalibrateImage as u8, cal.0, cal.1])
    }

    /// Set the radio power supply.
    ///
    /// # Examples
    ///
    /// Use the linear dropout regulator (LDO):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::RegMode;
    ///
    /// sg.set_regulator_mode(RegMode::Ldo)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// Use the switch mode power supply (SPMS):
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::RegMode;
    ///
    /// sg.set_regulator_mode(RegMode::Smps)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_regulator_mode(&mut self, reg_mode: RegMode) -> Result<(), Error> {
        self.write(&[OpCode::SetRegulatorMode as u8, reg_mode as u8])
    }

    /// Get the radio operational errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::OpError;
    ///
    /// let (status, error_mask) = sg.op_error()?;
    /// if error_mask & OpError::PllLockError.mask() != 0 {
    ///     // ... handle PLL lock error
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn op_error(&mut self) -> Result<(Status, u16), Error> {
        let data: [u8; 3] = self.read_n(OpCode::GetError)?;
        Ok((data[0].into(), u16::from_le_bytes([data[1], data[2]])))
    }

    /// Clear all errors as reported by [`op_error`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::OpError;
    ///
    /// let (status, error_mask) = sg.op_error()?;
    /// // ignore all errors
    /// if error_mask != 0 {
    ///     sg.clear_error()?;
    /// }
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    ///
    /// [`op_error`]: crate::subghz::SubGhz::op_error
    pub fn clear_error(&mut self) -> Result<(), Error> {
        self.write(&[OpCode::ClrError as u8, 0x00])
    }
}

// 5.8.8
/// Set TCXO mode command
impl<'d> SubGhz<'d, NoDma, NoDma> {
    /// Set the TCXO trim and HSE32 ready timeout.
    ///
    /// # Example
    ///
    /// Setup the TCXO with 1.7V trim and a 10ms timeout.
    ///
    /// ```no_run
    /// # let mut sg = embassy_stm32::subghz::SubGhz::new(p.SUBGHZSPI, ...);
    /// use embassy_stm32::subghz::{TcxoMode, TcxoTrim, Timeout};
    ///
    /// const TCXO_MODE: TcxoMode = TcxoMode::new()
    ///     .set_txco_trim(TcxoTrim::Volts1pt7)
    ///     .set_timeout(Timeout::from_millis_sat(10));
    /// sg.set_tcxo_mode(&TCXO_MODE)?;
    /// # Ok::<(), embassy_stm32::subghz::Error>(())
    /// ```
    pub fn set_tcxo_mode(&mut self, tcxo_mode: &TcxoMode) -> Result<(), Error> {
        self.write(tcxo_mode.as_slice())
    }
}

/// sub-GHz radio opcodes.
///
/// See Table 41 "Sub-GHz radio SPI commands overview"
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) enum OpCode {
    Calibrate = 0x89,
    CalibrateImage = 0x98,
    CfgDioIrq = 0x08,
    ClrError = 0x07,
    ClrIrqStatus = 0x02,
    GetError = 0x17,
    GetIrqStatus = 0x12,
    GetPacketStatus = 0x14,
    GetPacketType = 0x11,
    GetRssiInst = 0x15,
    GetRxBufferStatus = 0x13,
    GetStats = 0x10,
    GetStatus = 0xC0,
    ReadBuffer = 0x1E,
    RegRegister = 0x1D,
    ResetStats = 0x00,
    SetBufferBaseAddress = 0x8F,
    SetCad = 0xC5,
    SetCadParams = 0x88,
    SetFs = 0xC1,
    SetLoRaSymbTimeout = 0xA0,
    SetModulationParams = 0x8B,
    SetPacketParams = 0x8C,
    SetPacketType = 0x8A,
    SetPaConfig = 0x95,
    SetRegulatorMode = 0x96,
    SetRfFrequency = 0x86,
    SetRx = 0x82,
    SetRxDutyCycle = 0x94,
    SetSleep = 0x84,
    SetStandby = 0x80,
    SetStopRxTimerOnPreamble = 0x9F,
    SetTcxoMode = 0x97,
    SetTx = 0x83,
    SetTxContinuousPreamble = 0xD2,
    SetTxContinuousWave = 0xD1,
    SetTxParams = 0x8E,
    SetTxRxFallbackMode = 0x93,
    WriteBuffer = 0x0E,
    WriteRegister = 0x0D,
}

impl From<OpCode> for u8 {
    fn from(opcode: OpCode) -> Self {
        opcode as u8
    }
}

#[repr(u16)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Register {
    /// Generic bit synchronization.
    GBSYNC = 0x06AC,
    /// Generic packet control.
    GPKTCTL1A = 0x06B8,
    /// Generic whitening.
    GWHITEINIRL = 0x06B9,
    /// Generic CRC initial.
    GCRCINIRH = 0x06BC,
    /// Generic CRC polynomial.
    GCRCPOLRH = 0x06BE,
    /// Generic synchronization word 7.
    GSYNC7 = 0x06C0,
    /// LoRa synchronization word MSB.
    LSYNCH = 0x0740,
    /// LoRa synchronization word LSB.
    #[allow(dead_code)]
    LSYNCL = 0x0741,
    /// Receiver gain control.
    RXGAINC = 0x08AC,
    /// PA over current protection.
    PAOCP = 0x08E7,
    /// HSE32 OSC_IN capacitor trim.
    HSEINTRIM = 0x0911,
    /// HSE32 OSC_OUT capacitor trim.
    HSEOUTTRIM = 0x0912,
    /// SMPS control 0.
    SMPSC0 = 0x0916,
    /// Power control.
    PC = 0x091A,
    /// SMPS control 2.
    SMPSC2 = 0x0923,
}

impl Register {
    pub const fn address(self) -> u16 {
        self as u16
    }
}
