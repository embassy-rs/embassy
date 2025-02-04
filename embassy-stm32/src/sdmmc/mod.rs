//! Secure Digital / MultiMedia Card (SDMMC)
#![macro_use]

use core::default::Default;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use sdio_host::common_cmd::{self, Resp, ResponseLen};
use sdio_host::emmc::{ExtCSD, EMMC};
use sdio_host::sd::{BusWidth, CardCapacity, CardStatus, CurrentState, SDStatus, CIC, CID, CSD, OCR, RCA, SCR, SD};
use sdio_host::{emmc_cmd, sd_cmd, Cmd};

#[cfg(sdmmc_v1)]
use crate::dma::ChannelAndRequest;
#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, AnyPin, OutputType, SealedPin, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::sdmmc::Sdmmc as RegBlock;
use crate::rcc::{self, RccPeripheral};
use crate::time::Hertz;
use crate::{interrupt, peripherals};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> InterruptHandler<T> {
    fn data_interrupts(enable: bool) {
        let regs = T::regs();
        regs.maskr().write(|w| {
            w.set_dcrcfailie(enable);
            w.set_dtimeoutie(enable);
            w.set_dataendie(enable);

            #[cfg(sdmmc_v1)]
            w.set_stbiterre(enable);
            #[cfg(sdmmc_v2)]
            w.set_dabortie(enable);
        });
    }
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        Self::data_interrupts(false);
        T::state().wake();
    }
}

/// Frequency used for SD Card initialization. Must be no higher than 400 kHz.
const SD_INIT_FREQ: Hertz = Hertz(400_000);

/// The signalling scheme used on the SDMMC bus
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Signalling {
    SDR12,
    SDR25,
    SDR50,
    SDR104,
    DDR50,
}

impl Default for Signalling {
    fn default() -> Self {
        Signalling::SDR12
    }
}

/// Aligned data block for SDMMC transfers.
///
/// This is a 512-byte array, aligned to 4 bytes to satisfy DMA requirements.
#[repr(align(4))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataBlock(pub [u8; 512]);

impl Deref for DataBlock {
    type Target = [u8; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Command Block buffer for SDMMC command transfers.
///
/// This is a 16-word array, exposed so that DMA commpatible memory can be used if required.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CmdBlock(pub [u32; 16]);

impl CmdBlock {
    /// Creates a new instance of CmdBlock
    pub const fn new() -> Self {
        Self([0u32; 16])
    }
}

impl Deref for CmdBlock {
    type Target = [u32; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CmdBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Errors
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Timeout reported by the hardware
    Timeout,
    /// Timeout reported by the software driver.
    SoftwareTimeout,
    /// Unsupported card version.
    UnsupportedCardVersion,
    /// Unsupported card type.
    UnsupportedCardType,
    /// Unsupported voltage.
    UnsupportedVoltage,
    /// CRC error.
    Crc,
    /// No card inserted.
    NoCard,
    /// 8-lane buses are not supported for SD cards.
    BusWidth,
    /// Bad clock supplied to the SDMMC peripheral.
    BadClock,
    /// Signaling switch failed.
    SignalingSwitchFailed,
    /// Underrun error
    Underrun,
    /// ST bit error.
    #[cfg(sdmmc_v1)]
    StBitErr,
}

#[derive(Clone, Copy, Debug, Default)]
/// SD Card
pub struct Card {
    /// The type of this card
    pub card_type: CardCapacity,
    /// Operation Conditions Register
    pub ocr: OCR<SD>,
    /// Relative Card Address
    pub rca: u16,
    /// Card ID
    pub cid: CID<SD>,
    /// Card Specific Data
    pub csd: CSD<SD>,
    /// SD CARD Configuration Register
    pub scr: SCR,
    /// SD Status
    pub status: SDStatus,
}

#[derive(Clone, Copy, Debug, Default)]
/// eMMC storage
pub struct Emmc {
    /// The capacity of this card
    pub capacity: CardCapacity,
    /// Operation Conditions Register
    pub ocr: OCR<EMMC>,
    /// Relative Card Address
    pub rca: u16,
    /// Card ID
    pub cid: CID<EMMC>,
    /// Card Specific Data
    pub csd: CSD<EMMC>,
    /// Extended Card Specific Data
    pub ext_csd: ExtCSD,
}

#[repr(u8)]
enum PowerCtrl {
    Off = 0b00,
    On = 0b11,
}

fn get_waitresp_val(rlen: ResponseLen) -> u8 {
    match rlen {
        common_cmd::ResponseLen::Zero => 0,
        common_cmd::ResponseLen::R48 => 1,
        common_cmd::ResponseLen::R136 => 3,
    }
}

/// Calculate clock divisor. Returns a SDMMC_CK less than or equal to
/// `sdmmc_ck` in Hertz.
///
/// Returns `(bypass, clk_div, clk_f)`, where `bypass` enables clock divisor bypass (only sdmmc_v1),
/// `clk_div` is the divisor register value and `clk_f` is the resulting new clock frequency.
#[cfg(sdmmc_v1)]
fn clk_div(ker_ck: Hertz, sdmmc_ck: u32) -> Result<(bool, u8, Hertz), Error> {
    // sdmmc_v1 maximum clock is 50 MHz
    if sdmmc_ck > 50_000_000 {
        return Err(Error::BadClock);
    }

    // bypass divisor
    if ker_ck.0 <= sdmmc_ck {
        return Ok((true, 0, ker_ck));
    }

    // `ker_ck / sdmmc_ck` rounded up
    let clk_div = match (ker_ck.0 + sdmmc_ck - 1) / sdmmc_ck {
        0 | 1 => Ok(0),
        x @ 2..=258 => Ok((x - 2) as u8),
        _ => Err(Error::BadClock),
    }?;

    // SDIO_CK frequency = SDIOCLK / [CLKDIV + 2]
    let clk_f = Hertz(ker_ck.0 / (clk_div as u32 + 2));
    Ok((false, clk_div, clk_f))
}

/// Calculate clock divisor. Returns a SDMMC_CK less than or equal to
/// `sdmmc_ck` in Hertz.
///
/// Returns `(bypass, clk_div, clk_f)`, where `bypass` enables clock divisor bypass (only sdmmc_v1),
/// `clk_div` is the divisor register value and `clk_f` is the resulting new clock frequency.
#[cfg(sdmmc_v2)]
fn clk_div(ker_ck: Hertz, sdmmc_ck: u32) -> Result<(bool, u16, Hertz), Error> {
    // `ker_ck / sdmmc_ck` rounded up
    match (ker_ck.0 + sdmmc_ck - 1) / sdmmc_ck {
        0 | 1 => Ok((false, 0, ker_ck)),
        x @ 2..=2046 => {
            // SDMMC_CK frequency = SDMMCCLK / [CLKDIV * 2]
            let clk_div = ((x + 1) / 2) as u16;
            let clk = Hertz(ker_ck.0 / (clk_div as u32 * 2));

            Ok((false, clk_div, clk))
        }
        _ => Err(Error::BadClock),
    }
}

#[cfg(sdmmc_v1)]
type Transfer<'a> = crate::dma::Transfer<'a>;
#[cfg(sdmmc_v2)]
struct Transfer<'a> {
    _dummy: PhantomData<&'a ()>,
}

#[cfg(all(sdmmc_v1, dma))]
const DMA_TRANSFER_OPTIONS: crate::dma::TransferOptions = crate::dma::TransferOptions {
    pburst: crate::dma::Burst::Incr4,
    mburst: crate::dma::Burst::Incr4,
    flow_ctrl: crate::dma::FlowControl::Peripheral,
    fifo_threshold: Some(crate::dma::FifoThreshold::Full),
    priority: crate::dma::Priority::VeryHigh,
    circular: false,
    half_transfer_ir: false,
    complete_transfer_ir: true,
};
#[cfg(all(sdmmc_v1, not(dma)))]
const DMA_TRANSFER_OPTIONS: crate::dma::TransferOptions = crate::dma::TransferOptions {
    priority: crate::dma::Priority::VeryHigh,
    circular: false,
    half_transfer_ir: false,
    complete_transfer_ir: true,
};

/// SDMMC configuration
///
/// Default values:
/// data_transfer_timeout: 5_000_000
#[non_exhaustive]
pub struct Config {
    /// The timeout to be set for data transfers, in card bus clock periods
    pub data_transfer_timeout: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_transfer_timeout: 5_000_000,
        }
    }
}

/// Peripheral that can be operated over SDMMC
#[derive(Clone, Copy, Debug)]
pub enum SdmmcPeripheral {
    /// SD Card
    SdCard(Card),
    /// eMMC memory
    Emmc(Emmc),
}

impl SdmmcPeripheral {
    /// Get this peripheral's address on the SDMMC bus
    fn get_address(&self) -> u16 {
        match self {
            Self::SdCard(c) => c.rca,
            Self::Emmc(e) => e.rca,
        }
    }
    /// Is this a standard or high capacity peripheral?
    fn get_capacity(&self) -> CardCapacity {
        match self {
            Self::SdCard(c) => c.card_type,
            Self::Emmc(e) => e.capacity,
        }
    }
    /// Size in bytes
    fn size(&self) -> u64 {
        match self {
            // SDHC / SDXC / SDUC
            Self::SdCard(c) => u64::from(c.csd.block_count()) * 512,
            // capacity > 2GB
            Self::Emmc(e) => u64::from(e.ext_csd.sector_count()) * 512,
        }
    }

    /// Get a mutable reference to the SD Card.
    ///
    /// Panics if there is another peripheral instead.
    fn get_sd_card(&mut self) -> &mut Card {
        match *self {
            Self::SdCard(ref mut c) => c,
            _ => unreachable!("SD only"),
        }
    }

    /// Get a mutable reference to the eMMC.
    ///
    /// Panics if there is another peripheral instead.
    fn get_emmc(&mut self) -> &mut Emmc {
        match *self {
            Self::Emmc(ref mut e) => e,
            _ => unreachable!("eMMC only"),
        }
    }
}

/// Sdmmc device
pub struct Sdmmc<'d, T: Instance> {
    _peri: Peri<'d, T>,
    #[cfg(sdmmc_v1)]
    dma: ChannelAndRequest<'d>,

    clk: Peri<'d, AnyPin>,
    cmd: Peri<'d, AnyPin>,
    d0: Peri<'d, AnyPin>,
    d1: Option<Peri<'d, AnyPin>>,
    d2: Option<Peri<'d, AnyPin>>,
    d3: Option<Peri<'d, AnyPin>>,
    d4: Option<Peri<'d, AnyPin>>,
    d5: Option<Peri<'d, AnyPin>>,
    d6: Option<Peri<'d, AnyPin>>,
    d7: Option<Peri<'d, AnyPin>>,

    config: Config,
    /// Current clock to card
    clock: Hertz,
    /// Current signalling scheme to card
    signalling: Signalling,
    /// Card
    card: Option<SdmmcPeripheral>,

    /// An optional buffer to be used for commands
    /// This should be used if there are special memory location requirements for dma
    cmd_block: Option<&'d mut CmdBlock>,
}

const CLK_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v1)]
const CMD_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v2)]
const CMD_AF: AfType = AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up);
const DATA_AF: AfType = CMD_AF;

#[cfg(sdmmc_v1)]
impl<'d, T: Instance> Sdmmc<'d, T> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
            d1.set_as_af(d1.af_num(), DATA_AF);
            d2.set_as_af(d2.af_num(), DATA_AF);
            d3.set_as_af(d3.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            None,
            None,
            None,
            None,
            config,
        )
    }
}

#[cfg(sdmmc_v1)]
impl<'d, T: Instance> Sdmmc<'d, T> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
            d1.set_as_af(d1.af_num(), DATA_AF);
            d2.set_as_af(d2.af_num(), DATA_AF);
            d3.set_as_af(d3.af_num(), DATA_AF);
            d4.set_as_af(d4.af_num(), DATA_AF);
            d5.set_as_af(d5.af_num(), DATA_AF);
            d6.set_as_af(d6.af_num(), DATA_AF);
            d7.set_as_af(d7.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            Some(d4.into()),
            Some(d5.into()),
            Some(d6.into()),
            Some(d7.into()),
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
impl<'d, T: Instance> Sdmmc<'d, T> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
            d1.set_as_af(d1.af_num(), DATA_AF);
            d2.set_as_af(d2.af_num(), DATA_AF);
            d3.set_as_af(d3.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            None,
            None,
            None,
            None,
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
impl<'d, T: Instance> Sdmmc<'d, T> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            clk.set_as_af(clk.af_num(), CLK_AF);
            cmd.set_as_af(cmd.af_num(), CMD_AF);
            d0.set_as_af(d0.af_num(), DATA_AF);
            d1.set_as_af(d1.af_num(), DATA_AF);
            d2.set_as_af(d2.af_num(), DATA_AF);
            d3.set_as_af(d3.af_num(), DATA_AF);
            d4.set_as_af(d4.af_num(), DATA_AF);
            d5.set_as_af(d5.af_num(), DATA_AF);
            d6.set_as_af(d6.af_num(), DATA_AF);
            d7.set_as_af(d7.af_num(), DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            Some(d4.into()),
            Some(d5.into()),
            Some(d6.into()),
            Some(d7.into()),
            config,
        )
    }
}

impl<'d, T: Instance> Sdmmc<'d, T> {
    fn new_inner(
        sdmmc: Peri<'d, T>,
        #[cfg(sdmmc_v1)] dma: ChannelAndRequest<'d>,
        clk: Peri<'d, AnyPin>,
        cmd: Peri<'d, AnyPin>,
        d0: Peri<'d, AnyPin>,
        d1: Option<Peri<'d, AnyPin>>,
        d2: Option<Peri<'d, AnyPin>>,
        d3: Option<Peri<'d, AnyPin>>,
        d4: Option<Peri<'d, AnyPin>>,
        d5: Option<Peri<'d, AnyPin>>,
        d6: Option<Peri<'d, AnyPin>>,
        d7: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let regs = T::regs();
        regs.clkcr().write(|w| {
            w.set_pwrsav(false);
            w.set_negedge(false);

            // Hardware flow control is broken on SDIOv1 and causes clock glitches, which result in CRC errors.
            // See chip erratas for more details.
            #[cfg(sdmmc_v1)]
            w.set_hwfc_en(false);
            #[cfg(sdmmc_v2)]
            w.set_hwfc_en(true);

            #[cfg(sdmmc_v1)]
            w.set_clken(true);
        });

        // Power off, writen 00: Clock to the card is stopped;
        // D[7:0], CMD, and CK are driven high.
        regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::Off as u8));

        Self {
            _peri: sdmmc,
            #[cfg(sdmmc_v1)]
            dma,

            clk,
            cmd,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,

            config,
            clock: SD_INIT_FREQ,
            signalling: Default::default(),
            card: None,
            cmd_block: None,
        }
    }

    /// Data transfer is in progress
    #[inline]
    fn data_active() -> bool {
        let regs = T::regs();

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.rxact() || status.txact();
        #[cfg(sdmmc_v2)]
        return status.dpsmact();
    }

    /// Coammand transfer is in progress
    #[inline]
    fn cmd_active() -> bool {
        let regs = T::regs();

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.cmdact();
        #[cfg(sdmmc_v2)]
        return status.cpsmact();
    }

    /// Wait idle on CMDACT, RXACT and TXACT (v1) or DOSNACT and CPSMACT (v2)
    #[inline]
    fn wait_idle() {
        while Self::data_active() || Self::cmd_active() {}
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    #[allow(unused_variables)]
    fn prepare_datapath_read<'a>(
        config: &Config,
        #[cfg(sdmmc_v1)] dma: &'a mut ChannelAndRequest<'d>,
        buffer: &'a mut [u32],
        length_bytes: u32,
        block_size: u8,
    ) -> Transfer<'a> {
        assert!(block_size <= 14, "Block size up to 2^14 bytes");
        let regs = T::regs();

        // Command AND Data state machines must be idle
        Self::wait_idle();
        Self::clear_interrupt_flags();

        regs.dtimer().write(|w| w.set_datatime(config.data_transfer_timeout));
        regs.dlenr().write(|w| w.set_datalength(length_bytes));

        #[cfg(sdmmc_v1)]
        let transfer = unsafe { dma.read(regs.fifor().as_ptr() as *mut u32, buffer, DMA_TRANSFER_OPTIONS) };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_mut_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        regs.dctrl().modify(|w| {
            w.set_dblocksize(block_size);
            w.set_dtdir(true);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        transfer
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    fn prepare_datapath_write<'a>(&'a mut self, buffer: &'a [u32], length_bytes: u32, block_size: u8) -> Transfer<'a> {
        assert!(block_size <= 14, "Block size up to 2^14 bytes");
        let regs = T::regs();

        // Command AND Data state machines must be idle
        Self::wait_idle();
        Self::clear_interrupt_flags();

        regs.dtimer()
            .write(|w| w.set_datatime(self.config.data_transfer_timeout));
        regs.dlenr().write(|w| w.set_datalength(length_bytes));

        #[cfg(sdmmc_v1)]
        let transfer = unsafe {
            self.dma
                .write(buffer, regs.fifor().as_ptr() as *mut u32, DMA_TRANSFER_OPTIONS)
        };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        regs.dctrl().modify(|w| {
            w.set_dblocksize(block_size);
            w.set_dtdir(false);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        transfer
    }

    /// Stops the DMA datapath
    fn stop_datapath() {
        let regs = T::regs();

        #[cfg(sdmmc_v1)]
        regs.dctrl().modify(|w| {
            w.set_dmaen(false);
            w.set_dten(false);
        });
        #[cfg(sdmmc_v2)]
        regs.idmactrlr().modify(|w| w.set_idmaen(false));
    }

    /// Sets the CLKDIV field in CLKCR. Updates clock field in self
    fn clkcr_set_clkdiv(&mut self, freq: u32, width: BusWidth) -> Result<(), Error> {
        let regs = T::regs();

        let width_u32 = match width {
            BusWidth::One => 1u32,
            BusWidth::Four => 4u32,
            BusWidth::Eight => 8u32,
            _ => panic!("Invalid Bus Width"),
        };

        let ker_ck = T::frequency();
        let (_bypass, clkdiv, new_clock) = clk_div(ker_ck, freq)?;

        // Enforce AHB and SDMMC_CK clock relation. See RM0433 Rev 7
        // Section 55.5.8
        let sdmmc_bus_bandwidth = new_clock.0 * width_u32;
        assert!(ker_ck.0 > 3 * sdmmc_bus_bandwidth / 32);
        self.clock = new_clock;

        // CPSMACT and DPSMACT must be 0 to set CLKDIV
        Self::wait_idle();
        regs.clkcr().modify(|w| {
            w.set_clkdiv(clkdiv);
            #[cfg(sdmmc_v1)]
            w.set_bypass(_bypass);
        });

        Ok(())
    }

    /// Query the card status (CMD13, returns R1)
    fn read_status<Ext>(&self, card: &SdmmcPeripheral) -> Result<CardStatus<Ext>, Error>
    where
        CardStatus<Ext>: From<u32>,
    {
        let regs = T::regs();
        let rca = card.get_address();

        Self::cmd(common_cmd::card_status(rca, false), false)?; // CMD13

        let r1 = regs.respr(0).read().cardstatus();
        Ok(r1.into())
    }

    /// Select one card and place it into the _Tranfer State_
    ///
    /// If `None` is specifed for `card`, all cards are put back into
    /// _Stand-by State_
    fn select_card(&self, rca: Option<u16>) -> Result<(), Error> {
        // Determine Relative Card Address (RCA) of given card
        let rca = rca.unwrap_or(0);

        let r = Self::cmd(common_cmd::select_card(rca), false);
        match (r, rca) {
            (Err(Error::Timeout), 0) => Ok(()),
            _ => r,
        }
    }

    /// Clear flags in interrupt clear register
    #[inline]
    fn clear_interrupt_flags() {
        let regs = T::regs();
        regs.icr().write(|w| {
            w.set_ccrcfailc(true);
            w.set_dcrcfailc(true);
            w.set_ctimeoutc(true);
            w.set_dtimeoutc(true);
            w.set_txunderrc(true);
            w.set_rxoverrc(true);
            w.set_cmdrendc(true);
            w.set_cmdsentc(true);
            w.set_dataendc(true);
            w.set_dbckendc(true);
            w.set_sdioitc(true);
            #[cfg(sdmmc_v1)]
            w.set_stbiterrc(true);

            #[cfg(sdmmc_v2)]
            {
                w.set_dholdc(true);
                w.set_dabortc(true);
                w.set_busyd0endc(true);
                w.set_ackfailc(true);
                w.set_acktimeoutc(true);
                w.set_vswendc(true);
                w.set_ckstopc(true);
                w.set_idmatec(true);
                w.set_idmabtcc(true);
            }
        });
    }

    /// Send command to card
    #[allow(unused_variables)]
    fn cmd<R: Resp>(cmd: Cmd<R>, data: bool) -> Result<(), Error> {
        let regs = T::regs();

        Self::clear_interrupt_flags();
        // CP state machine must be idle
        while Self::cmd_active() {}

        // Command arg
        regs.argr().write(|w| w.set_cmdarg(cmd.arg));

        // Command index and start CP State Machine
        regs.cmdr().write(|w| {
            w.set_waitint(false);
            w.set_waitresp(get_waitresp_val(cmd.response_len()));
            w.set_cmdindex(cmd.cmd);
            w.set_cpsmen(true);

            #[cfg(sdmmc_v2)]
            {
                // Special mode in CP State Machine
                // CMD12: Stop Transmission
                let cpsm_stop_transmission = cmd.cmd == 12;
                w.set_cmdstop(cpsm_stop_transmission);
                w.set_cmdtrans(data);
            }
        });

        let mut status;
        if cmd.response_len() == ResponseLen::Zero {
            // Wait for CMDSENT or a timeout
            while {
                status = regs.star().read();
                !(status.ctimeout() || status.cmdsent())
            } {}
        } else {
            // Wait for CMDREND or CCRCFAIL or a timeout
            while {
                status = regs.star().read();
                !(status.ctimeout() || status.cmdrend() || status.ccrcfail())
            } {}
        }

        if status.ctimeout() {
            return Err(Error::Timeout);
        } else if status.ccrcfail() {
            return Err(Error::Crc);
        }
        Ok(())
    }

    fn on_drop() {
        let regs = T::regs();
        if Self::data_active() {
            Self::clear_interrupt_flags();
            // Send abort
            // CP state machine must be idle
            while Self::cmd_active() {}

            // Command arg
            regs.argr().write(|w| w.set_cmdarg(0));

            // Command index and start CP State Machine
            regs.cmdr().write(|w| {
                w.set_waitint(false);
                w.set_waitresp(get_waitresp_val(ResponseLen::R48));
                w.set_cmdindex(12);
                w.set_cpsmen(true);

                #[cfg(sdmmc_v2)]
                {
                    w.set_cmdstop(true);
                    w.set_cmdtrans(false);
                }
            });

            // Wait for the abort
            while Self::data_active() {}
        }
        InterruptHandler::<T>::data_interrupts(false);
        Self::clear_interrupt_flags();
        Self::stop_datapath();
    }

    /// Wait for a previously started datapath transfer to complete from an interrupt.
    #[inline]
    async fn complete_datapath_transfer() -> Result<(), Error> {
        let regs = T::regs();

        let res = poll_fn(|cx| {
            T::state().register(cx.waker());
            let status = regs.star().read();

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            }
            if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            }
            if status.txunderr() {
                return Poll::Ready(Err(Error::Underrun));
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                return Poll::Ready(Err(Error::StBitErr));
            }
            if status.dataend() {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        Self::clear_interrupt_flags();

        res
    }

    /// Read a data block.
    #[inline]
    pub async fn read_block(&mut self, block_idx: u32, buffer: &mut DataBlock) -> Result<(), Error> {
        let card_capacity = self.card()?.get_capacity();

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &mut *((&mut buffer.0) as *mut [u8; 512] as *mut [u32; 128]) };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        Self::cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            buffer,
            512,
            9,
        );
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(common_cmd::read_single_block(address), true)?;

        let res = Self::complete_datapath_transfer().await;

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);
        }
        res
    }

    /// Read multiple data blocks.
    #[inline]
    pub async fn read_blocks(&mut self, block_idx: u32, blocks: &mut [DataBlock]) -> Result<(), Error> {
        let card_capacity = self.card()?.get_capacity();

        // NOTE(unsafe) reinterpret buffer as &mut [u32]
        let buffer = unsafe {
            let ptr = blocks.as_mut_ptr() as *mut u32;
            let len = blocks.len() * 128;
            core::slice::from_raw_parts_mut(ptr, len)
        };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        Self::cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            buffer,
            512 * blocks.len() as u32,
            9,
        );
        InterruptHandler::<T>::data_interrupts(true);

        Self::cmd(common_cmd::read_multiple_blocks(address), true)?;

        let res = poll_fn(|cx| {
            T::state().register(cx.waker());
            let status = regs.star().read();

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            }
            if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                return Poll::Ready(Err(Error::StBitErr));
            }
            if status.dataend() {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        Self::cmd(common_cmd::stop_transmission(), false)?; // CMD12
        Self::clear_interrupt_flags();

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);
        }
        res
    }

    /// Write a data block.
    pub async fn write_block(&mut self, block_idx: u32, buffer: &DataBlock) -> Result<(), Error> {
        let card = self.card.as_mut().ok_or(Error::NoCard)?;

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &*((&buffer.0) as *const [u8; 512] as *const [u32; 128]) };

        // Always read 1 block of 512 bytes
        //  cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        Self::cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let on_drop = OnDrop::new(|| Self::on_drop());

        // sdmmc_v1 uses different cmd/dma order than v2, but only for writes
        #[cfg(sdmmc_v1)]
        Self::cmd(common_cmd::write_single_block(address), true)?;

        let transfer = self.prepare_datapath_write(buffer, 512, 9);
        InterruptHandler::<T>::data_interrupts(true);

        #[cfg(sdmmc_v2)]
        Self::cmd(common_cmd::write_single_block(address), true)?;

        let res = Self::complete_datapath_transfer().await;

        match res {
            Ok(_) => {
                on_drop.defuse();
                Self::stop_datapath();
                drop(transfer);

                // TODO: Make this configurable
                let mut timeout: u32 = 0x00FF_FFFF;

                let card = self.card.as_ref().unwrap();
                while timeout > 0 {
                    let ready_for_data = match card {
                        SdmmcPeripheral::Emmc(_) => self.read_status::<EMMC>(card)?.ready_for_data(),
                        SdmmcPeripheral::SdCard(_) => self.read_status::<SD>(card)?.ready_for_data(),
                    };

                    if ready_for_data {
                        return Ok(());
                    }
                    timeout -= 1;
                }
                Err(Error::SoftwareTimeout)
            }
            Err(e) => Err(e),
        }
    }

    /// Write multiple data blocks.
    pub async fn write_blocks(&mut self, block_idx: u32, blocks: &[DataBlock]) -> Result<(), Error> {
        let card = self.card.as_mut().ok_or(Error::NoCard)?;

        // NOTE(unsafe) reinterpret buffer as &[u32]
        let buffer = unsafe {
            let ptr = blocks.as_ptr() as *const u32;
            let len = blocks.len() * 128;
            core::slice::from_raw_parts(ptr, len)
        };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };

        Self::cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let block_count = blocks.len();

        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        #[cfg(sdmmc_v1)]
        Self::cmd(common_cmd::write_multiple_blocks(address), true)?; // CMD25

        // Setup write command
        let transfer = self.prepare_datapath_write(buffer, 512 * block_count as u32, 9);
        InterruptHandler::<T>::data_interrupts(true);

        #[cfg(sdmmc_v2)]
        Self::cmd(common_cmd::write_multiple_blocks(address), true)?; // CMD25

        let res = poll_fn(|cx| {
            T::state().register(cx.waker());

            let status = regs.star().read();

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            }
            if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            }
            if status.txunderr() {
                return Poll::Ready(Err(Error::Underrun));
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                return Poll::Ready(Err(Error::StBitErr));
            }
            if status.dataend() {
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        })
        .await;

        Self::cmd(common_cmd::stop_transmission(), false)?; // CMD12
        Self::clear_interrupt_flags();

        match res {
            Ok(_) => {
                on_drop.defuse();
                Self::stop_datapath();
                drop(transfer);

                // TODO: Make this configurable
                let mut timeout: u32 = 0x00FF_FFFF;

                // Try to read card status (ACMD13)
                while timeout > 0 {
                    match self.read_sd_status().await {
                        Ok(_) => return Ok(()),
                        Err(Error::Timeout) => (), // Try again
                        Err(e) => return Err(e),
                    }
                    timeout -= 1;
                }
                Err(Error::SoftwareTimeout)
            }
            Err(e) => Err(e),
        }
    }

    /// Get a reference to the initialized card
    ///
    /// # Errors
    ///
    /// Returns Error::NoCard if [`init_sd_card`](#method.init_sd_card) or
    /// [`init_emmc`](#method.init_emmc) has not previously succeeded
    #[inline]
    pub fn card(&self) -> Result<&SdmmcPeripheral, Error> {
        self.card.as_ref().ok_or(Error::NoCard)
    }

    /// Get the current SDMMC bus clock
    pub fn clock(&self) -> Hertz {
        self.clock
    }

    /// Set a specific cmd buffer rather than using the default stack allocated one.
    /// This is required if stack RAM cannot be used with DMA and usually manifests
    /// itself as an indefinite wait on a dma transfer because the dma peripheral
    /// cannot access the memory.
    pub fn set_cmd_block(&mut self, cmd_block: &'d mut CmdBlock) {
        self.cmd_block = Some(cmd_block)
    }

    async fn init_internal(&mut self, freq: Hertz, mut card: SdmmcPeripheral) -> Result<(), Error> {
        let regs = T::regs();
        let ker_ck = T::frequency();

        let bus_width = match (self.d3.is_some(), self.d7.is_some()) {
            (true, true) => {
                if matches!(card, SdmmcPeripheral::SdCard(_)) {
                    return Err(Error::BusWidth);
                }
                BusWidth::Eight
            }
            (true, false) => BusWidth::Four,
            _ => BusWidth::One,
        };

        // While the SD/SDIO card or eMMC is in identification mode,
        // the SDMMC_CK frequency must be no more than 400 kHz.
        let (_bypass, clkdiv, init_clock) = unwrap!(clk_div(ker_ck, SD_INIT_FREQ.0));
        self.clock = init_clock;

        // CPSMACT and DPSMACT must be 0 to set WIDBUS
        Self::wait_idle();

        regs.clkcr().modify(|w| {
            w.set_widbus(0);
            w.set_clkdiv(clkdiv);
            #[cfg(sdmmc_v1)]
            w.set_bypass(_bypass);
        });

        regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::On as u8));
        Self::cmd(common_cmd::idle(), false)?;

        match card {
            SdmmcPeripheral::SdCard(ref mut card) => {
                // Check if cards supports CMD8 (with pattern)
                Self::cmd(sd_cmd::send_if_cond(1, 0xAA), false)?;
                let cic = CIC::from(regs.respr(0).read().cardstatus());

                if cic.pattern() != 0xAA {
                    return Err(Error::UnsupportedCardVersion);
                }

                if cic.voltage_accepted() & 1 == 0 {
                    return Err(Error::UnsupportedVoltage);
                }

                let ocr = loop {
                    // Signal that next command is a app command
                    Self::cmd(common_cmd::app_cmd(0), false)?; // CMD55

                    // 3.2-3.3V
                    let voltage_window = 1 << 5;
                    // Initialize card
                    match Self::cmd(sd_cmd::sd_send_op_cond(true, false, true, voltage_window), false) {
                        // ACMD41
                        Ok(_) => (),
                        Err(Error::Crc) => (),
                        Err(err) => return Err(err),
                    }
                    let ocr: OCR<SD> = regs.respr(0).read().cardstatus().into();
                    if !ocr.is_busy() {
                        // Power up done
                        break ocr;
                    }
                };

                if ocr.high_capacity() {
                    // Card is SDHC or SDXC or SDUC
                    card.card_type = CardCapacity::HighCapacity;
                } else {
                    card.card_type = CardCapacity::StandardCapacity;
                }
                card.ocr = ocr;
            }
            SdmmcPeripheral::Emmc(ref mut emmc) => {
                let ocr = loop {
                    let high_voltage = 0b0 << 7;
                    let access_mode = 0b10 << 29;
                    let op_cond = high_voltage | access_mode | 0b1_1111_1111 << 15;
                    // Initialize card
                    match Self::cmd(emmc_cmd::send_op_cond(op_cond), false) {
                        Ok(_) => (),
                        Err(Error::Crc) => (),
                        Err(err) => return Err(err),
                    }
                    let ocr: OCR<EMMC> = regs.respr(0).read().cardstatus().into();
                    if !ocr.is_busy() {
                        // Power up done
                        break ocr;
                    }
                };

                emmc.capacity = if ocr.access_mode() == 0b10 {
                    // Card is SDHC or SDXC or SDUC
                    CardCapacity::HighCapacity
                } else {
                    CardCapacity::StandardCapacity
                };
                emmc.ocr = ocr;
            }
        }

        Self::cmd(common_cmd::all_send_cid(), false)?; // CMD2
        let cid0 = regs.respr(0).read().cardstatus() as u128;
        let cid1 = regs.respr(1).read().cardstatus() as u128;
        let cid2 = regs.respr(2).read().cardstatus() as u128;
        let cid3 = regs.respr(3).read().cardstatus() as u128;
        let cid = (cid0 << 96) | (cid1 << 64) | (cid2 << 32) | (cid3);

        match card {
            SdmmcPeripheral::SdCard(ref mut card) => {
                card.cid = cid.into();

                Self::cmd(sd_cmd::send_relative_address(), false)?;
                let rca = RCA::<SD>::from(regs.respr(0).read().cardstatus());
                card.rca = rca.address();
            }
            SdmmcPeripheral::Emmc(ref mut emmc) => {
                emmc.cid = cid.into();

                emmc.rca = 1u16.into();
                Self::cmd(emmc_cmd::assign_relative_address(emmc.rca), false)?;
            }
        }

        Self::cmd(common_cmd::send_csd(card.get_address()), false)?;
        let csd0 = regs.respr(0).read().cardstatus() as u128;
        let csd1 = regs.respr(1).read().cardstatus() as u128;
        let csd2 = regs.respr(2).read().cardstatus() as u128;
        let csd3 = regs.respr(3).read().cardstatus() as u128;
        let csd = (csd0 << 96) | (csd1 << 64) | (csd2 << 32) | (csd3);

        self.select_card(Some(card.get_address()))?;

        let bus_width = match card {
            SdmmcPeripheral::SdCard(ref mut card) => {
                card.csd = csd.into();

                self.get_scr(card).await?;

                if !card.scr.bus_width_four() {
                    BusWidth::One
                } else {
                    BusWidth::Four
                }
            }
            SdmmcPeripheral::Emmc(ref mut emmc) => {
                emmc.csd = csd.into();

                bus_width
            }
        };

        // Set bus width
        let widbus = match bus_width {
            BusWidth::Eight => 2,
            BusWidth::Four => 1,
            BusWidth::One => 0,
            _ => unreachable!(),
        };

        match card {
            SdmmcPeripheral::SdCard(ref mut card) => {
                let acmd_arg = match bus_width {
                    BusWidth::Four if card.scr.bus_width_four() => 2,
                    _ => 0,
                };
                Self::cmd(common_cmd::app_cmd(card.rca), false)?;
                Self::cmd(sd_cmd::cmd6(acmd_arg), false)?;
            }
            SdmmcPeripheral::Emmc(_) => {
                // Write bus width to ExtCSD byte 183
                Self::cmd(
                    emmc_cmd::modify_ext_csd(emmc_cmd::AccessMode::WriteByte, 183, widbus),
                    false,
                )?;

                // Wait for ready after R1b response
                loop {
                    let status = self.read_status::<EMMC>(&card)?;

                    if status.ready_for_data() {
                        break;
                    }
                }
            }
        }

        // CPSMACT and DPSMACT must be 0 to set WIDBUS
        Self::wait_idle();

        regs.clkcr().modify(|w| w.set_widbus(widbus));

        // Set Clock
        if freq.0 <= 25_000_000 {
            // Final clock frequency
            self.clkcr_set_clkdiv(freq.0, bus_width)?;
        } else {
            // Switch to max clock for SDR12
            self.clkcr_set_clkdiv(25_000_000, bus_width)?;
        }

        self.card = Some(card);

        match card {
            SdmmcPeripheral::SdCard(_) => {
                // Read status
                self.read_sd_status().await?;

                if freq.0 > 25_000_000 {
                    // Switch to SDR25
                    self.signalling = self.switch_signalling_mode(Signalling::SDR25).await?;

                    if self.signalling == Signalling::SDR25 {
                        // Set final clock frequency
                        self.clkcr_set_clkdiv(freq.0, bus_width)?;

                        if self.read_status::<SD>(self.card.as_ref().unwrap())?.state() != CurrentState::Transfer {
                            return Err(Error::SignalingSwitchFailed);
                        }
                    }
                }

                // Read status after signalling change
                self.read_sd_status().await?;
            }
            SdmmcPeripheral::Emmc(_) => {
                self.read_ext_csd().await?;
            }
        }

        Ok(())
    }

    /// Initializes card (if present) and sets the bus at the specified frequency.
    ///
    /// SD only.
    pub async fn init_sd_card(&mut self, freq: Hertz) -> Result<(), Error> {
        self.init_internal(freq, SdmmcPeripheral::SdCard(Card::default())).await
    }

    /// Switch mode using CMD6.
    ///
    /// Attempt to set a new signalling mode. The selected
    /// signalling mode is returned. Expects the current clock
    /// frequency to be > 12.5MHz.
    ///
    /// SD only.
    async fn switch_signalling_mode(&mut self, signalling: Signalling) -> Result<Signalling, Error> {
        let _ = self.card.as_mut().ok_or(Error::NoCard)?.get_sd_card();
        // NB PLSS v7_10 4.3.10.4: "the use of SET_BLK_LEN command is not
        // necessary"

        let set_function = 0x8000_0000
            | match signalling {
                // See PLSS v7_10 Table 4-11
                Signalling::DDR50 => 0xFF_FF04,
                Signalling::SDR104 => 0xFF_1F03,
                Signalling::SDR50 => 0xFF_1F02,
                Signalling::SDR25 => 0xFF_FF01,
                Signalling::SDR12 => 0xFF_FF00,
            };

        let status = match self.cmd_block.as_deref_mut() {
            Some(x) => x,
            None => &mut CmdBlock::new(),
        };

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            status.as_mut(),
            64,
            6,
        );
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(sd_cmd::cmd6(set_function), true)?; // CMD6

        let res = Self::complete_datapath_transfer().await;

        // Host is allowed to use the new functions at least 8
        // clocks after the end of the switch command
        // transaction. We know the current clock period is < 80ns,
        // so a total delay of 640ns is required here
        for _ in 0..300 {
            cortex_m::asm::nop();
        }

        match res {
            Ok(_) => {
                on_drop.defuse();
                Self::stop_datapath();
                drop(transfer);

                // Function Selection of Function Group 1
                let selection = (u32::from_be(status[4]) >> 24) & 0xF;

                match selection {
                    0 => Ok(Signalling::SDR12),
                    1 => Ok(Signalling::SDR25),
                    2 => Ok(Signalling::SDR50),
                    3 => Ok(Signalling::SDR104),
                    4 => Ok(Signalling::DDR50),
                    _ => Err(Error::UnsupportedCardType),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Reads the SCR register.
    ///
    /// SD only.
    async fn get_scr(&mut self, card: &mut Card) -> Result<(), Error> {
        // Read the 64-bit SCR register
        Self::cmd(common_cmd::set_block_length(8), false)?; // CMD16
        Self::cmd(common_cmd::app_cmd(card.rca), false)?;

        let cmd_block = match self.cmd_block.as_deref_mut() {
            Some(x) => x,
            None => &mut CmdBlock::new(),
        };
        let scr = &mut cmd_block.0[..2];

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            scr,
            8,
            3,
        );
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(sd_cmd::send_scr(), true)?;

        let res = Self::complete_datapath_transfer().await;

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);

            unsafe {
                let scr_bytes = &*(&scr as *const _ as *const [u8; 8]);
                card.scr = SCR(u64::from_be_bytes(*scr_bytes));
            }
        }
        res
    }

    /// Reads the SD Status (ACMD13)
    ///
    /// SD only.
    async fn read_sd_status(&mut self) -> Result<(), Error> {
        let card = self.card.as_mut().ok_or(Error::NoCard)?.get_sd_card();
        let rca = card.rca;

        let cmd_block = match self.cmd_block.as_deref_mut() {
            Some(x) => x,
            None => &mut CmdBlock::new(),
        };

        Self::cmd(common_cmd::set_block_length(64), false)?; // CMD16
        Self::cmd(common_cmd::app_cmd(rca), false)?; // APP

        let status = cmd_block;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            status.as_mut(),
            64,
            6,
        );
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(sd_cmd::sd_status(), true)?;

        let res = Self::complete_datapath_transfer().await;

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);

            for byte in status.iter_mut() {
                *byte = u32::from_be(*byte);
            }
            card.status = status.0.into();
        }
        res
    }

    /// Initializes eMMC and sets the bus at the specified frequency.
    ///
    /// eMMC only.
    pub async fn init_emmc(&mut self, freq: Hertz) -> Result<(), Error> {
        self.init_internal(freq, SdmmcPeripheral::Emmc(Emmc::default())).await
    }

    /// Gets the EXT_CSD register.
    ///
    /// eMMC only.
    async fn read_ext_csd(&mut self) -> Result<(), Error> {
        let card = self.card.as_mut().ok_or(Error::NoCard)?.get_emmc();

        // Note: cmd_block can't be used because ExtCSD is too long to fit.
        let mut data_block = DataBlock([0u8; 512]);

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &mut *((&mut data_block.0) as *mut [u8; 512] as *mut [u32; 128]) };

        Self::cmd(common_cmd::set_block_length(512), false).unwrap(); // CMD16

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = Self::prepare_datapath_read(
            &self.config,
            #[cfg(sdmmc_v1)]
            &mut self.dma,
            buffer,
            512,
            9,
        );
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(emmc_cmd::send_ext_csd(), true)?;

        let res = Self::complete_datapath_transfer().await;

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);

            card.ext_csd = unsafe { core::mem::transmute::<_, [u32; 128]>(data_block.0) }.into();
        }
        res
    }
}

impl<'d, T: Instance> Drop for Sdmmc<'d, T> {
    fn drop(&mut self) {
        T::Interrupt::disable();
        Self::on_drop();

        critical_section::with(|_| {
            self.clk.set_as_disconnected();
            self.cmd.set_as_disconnected();
            self.d0.set_as_disconnected();
            if let Some(x) = &mut self.d1 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d2 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d3 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d4 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d5 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d6 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d7 {
                x.set_as_disconnected();
            }
        });
    }
}

//////////////////////////////////////////////////////

trait SealedInstance {
    fn regs() -> RegBlock;
    fn state() -> &'static AtomicWaker;
}

/// SDMMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(CkPin, Instance);
pin_trait!(CmdPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);

#[cfg(sdmmc_v1)]
dma_trait!(SdmmcDma, Instance);

foreach_peripheral!(
    (sdmmc, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn regs() -> RegBlock {
                crate::pac::$inst
            }

            fn state() -> &'static ::embassy_sync::waitqueue::AtomicWaker {
                static WAKER: ::embassy_sync::waitqueue::AtomicWaker = ::embassy_sync::waitqueue::AtomicWaker::new();
                &WAKER
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$inst;
        }
    };
);

impl<'d, T: Instance> block_device_driver::BlockDevice<512> for Sdmmc<'d, T> {
    type Error = Error;
    type Align = aligned::A4;

    async fn read(
        &mut self,
        block_address: u32,
        buf: &mut [aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        // TODO: I think block_address needs to be adjusted by the partition start offset
        if buf.len() == 1 {
            let block = unsafe { &mut *(&mut buf[0] as *mut _ as *mut crate::sdmmc::DataBlock) };
            self.read_block(block_address, block).await?;
        } else {
            let blocks: &mut [DataBlock] =
                unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut DataBlock, buf.len()) };
            self.read_blocks(block_address, blocks).await?;
        }
        Ok(())
    }

    async fn write(
        &mut self,
        block_address: u32,
        buf: &[aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        // TODO: I think block_address needs to be adjusted by the partition start offset
        if buf.len() == 1 {
            let block = unsafe { &*(&buf[0] as *const _ as *const crate::sdmmc::DataBlock) };
            self.write_block(block_address, block).await?;
        } else {
            let blocks: &[DataBlock] =
                unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const DataBlock, buf.len()) };
            self.write_blocks(block_address, blocks).await?;
        }
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.card()?.size())
    }
}
