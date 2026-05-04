//! Secure Digital / MultiMedia Card (SDMMC)
#![macro_use]

use core::default::Default;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{Ordering, fence};
use core::task::Poll;

use aligned::{A4, Aligned};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use sdio_host::Cmd;
use sdio_host::common_cmd::{self, R1, R2, Resp, ResponseLen, Rz};
use sdio_host::sd::{BusWidth, CID, CSD, CardStatus};
#[cfg(sdmmc_uhs)]
use sdio_host::sd_cmd;

#[cfg(sdmmc_v1)]
use crate::dma::ChannelAndRequest;
#[cfg(sdmmc_uhs)]
use crate::gpio::Output;
#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::sdmmc::Sdmmc as RegBlock;
use crate::rcc::{self, RccInfo, RccPeripheral, SealedRccPeripheral};
use crate::time::Hertz;
use crate::{block_for_us, interrupt, peripherals};

/// Module for SD and EMMC cards
pub mod sd;

/// Module for SDIO interface
pub mod sdio;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().tx_waker.wake();
        let status = T::info().regs.star().read();
        #[cfg(sdmmc_v1)]
        if status.dcrcfail() || status.dtimeout() || status.dataend() || status.dbckend() || status.stbiterr() {
            T::state().tx_waker.wake();
        }

        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        if status.dcrcfail() || status.dtimeout() || status.dataend() || status.dbckend() || status.dabort() {
            T::state().tx_waker.wake();
        }

        if status.sdioit() {
            T::state().it_waker.wake();
        }

        T::info().regs.maskr().modify(|w| {
            if status.sdioit() {
                w.set_sdioitie(false);
            }
            if status.dcrcfail() {
                w.set_dcrcfailie(false);
            }
            if status.dtimeout() {
                w.set_dtimeoutie(false);
            }
            if status.dataend() {
                w.set_dataendie(false);
            }
            if status.dbckend() {
                w.set_dbckendie(false);
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                w.set_stbiterre(false);
            }
            #[cfg(any(sdmmc_v2, sdmmc_v3))]
            if status.dabort() {
                w.set_dabortie(false);
            }
        });
    }
}

struct U128(pub u128);

struct CommandResponse<R: TypedResp>(R::Word);

trait TypedResp: Resp {
    type Word: From<U128>;
}

impl From<U128> for () {
    fn from(value: U128) -> Self {
        match value.0 {
            0 => (),
            _ => unreachable!(),
        }
    }
}

impl From<U128> for u32 {
    fn from(value: U128) -> Self {
        value.0.try_into().unwrap()
    }
}

impl From<U128> for u128 {
    fn from(value: U128) -> Self {
        value.0
    }
}

impl TypedResp for Rz {
    type Word = ();
}

impl TypedResp for R1 {
    type Word = u32;
}

impl<E> From<CommandResponse<R1>> for CardStatus<E> {
    fn from(value: CommandResponse<R1>) -> Self {
        CardStatus::<E>::from(value.0)
    }
}

impl TypedResp for R2 {
    type Word = u128;
}

impl<E> From<CommandResponse<R2>> for CID<E> {
    fn from(value: CommandResponse<R2>) -> Self {
        CID::<E>::from(value.0)
    }
}

impl<E> From<CommandResponse<R2>> for CSD<E> {
    fn from(value: CommandResponse<R2>) -> Self {
        CSD::<E>::from(value.0)
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

const fn aligned_mut(x: &mut [u32]) -> &mut Aligned<A4, [u8]> {
    let len = x.len() * 4;
    unsafe { core::mem::transmute(slice::from_raw_parts_mut(x.as_mut_ptr() as *mut u8, len)) }
}

const fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as *mut u8, len) }
}

#[allow(unused)]
const fn slice32_mut(x: &mut Aligned<A4, [u8]>) -> &mut [u32] {
    let len = (size_of_val(x) + 4 - 1) / 4;
    unsafe { slice::from_raw_parts_mut(x as *mut Aligned<A4, [u8]> as *mut u32, len) }
}

const fn aligned_ref(x: &[u32]) -> &Aligned<A4, [u8]> {
    let len = x.len() * 4;
    unsafe { core::mem::transmute(slice::from_raw_parts(x.as_ptr() as *const u8, len)) }
}

const fn slice8_ref(x: &[u32]) -> &[u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts(x.as_ptr() as *const u8, len) }
}

#[allow(unused)]
const fn slice32_ref(x: &Aligned<A4, [u8]>) -> &[u32] {
    let len = (size_of_val(x) + 4 - 1) / 4;
    unsafe { slice::from_raw_parts(x as *const Aligned<A4, [u8]> as *const u32, len) }
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
    /// UHS-I voltage switch (CMD11) failed — the card refused 1.8V or
    /// the peripheral didn't complete the handshake. The driver leaves
    /// the bus at 3.3V and the caller can retry without the vswitch
    /// pin to fall back to HS mode.
    #[cfg(sdmmc_uhs)]
    VoltageSwitchFailed,
    /// Underrun error
    Underrun,
    /// ST bit error.
    #[cfg(sdmmc_v1)]
    StBitErr,
}

#[repr(u8)]
enum PowerCtrl {
    Off = 0b00,
    On = 0b11,
}

enum DatapathMode {
    Block(BlockSize),
    Byte,
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
fn clk_div(ker_ck: Hertz, sdmmc_ck: Hertz) -> Result<(bool, u8, Hertz), Error> {
    // sdmmc_v1 maximum clock is 50 MHz
    if sdmmc_ck.0 > 50_000_000 {
        return Err(Error::BadClock);
    }

    // bypass divisor
    if ker_ck.0 <= sdmmc_ck.0 {
        return Ok((true, 0, ker_ck));
    }

    let clk_div = match ker_ck.0.div_ceil(sdmmc_ck.0) {
        0 | 1 => Ok(0),
        x @ 2..=258 => Ok((x - 2) as u8),
        _ => Err(Error::BadClock),
    }?;

    // SDIO_CK frequency = SDIOCLK / [CLKDIV + 2]
    let clk_f = Hertz(ker_ck.0 / (clk_div as u32 + 2));
    Ok((false, clk_div, clk_f))
}

fn bus_width_vals(bus_width: BusWidth) -> (u8, u32) {
    match bus_width {
        BusWidth::One => (0, 1u32),
        BusWidth::Four => (1, 4u32),
        BusWidth::Eight => (2, 8u32),
        _ => panic!("Invalid Bus Width"),
    }
}

#[repr(u8)]
enum BlockSize {
    Size1 = 0b0000,
    Size2 = 0b0001,
    Size4 = 0b0010,
    Size8 = 0b0011,
    Size16 = 0b0100,
    Size32 = 0b0101,
    Size64 = 0b0110,
    Size128 = 0b0111,
    Size256 = 0b1000,
    Size512 = 0b1001,
    Size1024 = 0b1010,
    Size2048 = 0b1011,
    Size4096 = 0b1100,
    Size8192 = 0b1101,
    Size16384 = 0b1110,
}

const fn block_size(bytes: usize) -> BlockSize {
    match bytes {
        1 => BlockSize::Size1,
        2 => BlockSize::Size2,
        4 => BlockSize::Size4,
        8 => BlockSize::Size8,
        16 => BlockSize::Size16,
        32 => BlockSize::Size32,
        64 => BlockSize::Size64,
        128 => BlockSize::Size128,
        256 => BlockSize::Size256,
        512 => BlockSize::Size512,
        1024 => BlockSize::Size1024,
        2048 => BlockSize::Size2048,
        4096 => BlockSize::Size4096,
        8192 => BlockSize::Size8192,
        16384 => BlockSize::Size16384,
        _ => core::unreachable!(),
    }
}

/// Calculate clock divisor. Returns a SDMMC_CK less than or equal to
/// `sdmmc_ck` in Hertz.
///
/// Returns `(bypass, clk_div, clk_f)`, where `bypass` enables clock divisor bypass (only sdmmc_v1),
/// `clk_div` is the divisor register value and `clk_f` is the resulting new clock frequency.
#[cfg(any(sdmmc_v2, sdmmc_v3))]
fn clk_div(ker_ck: Hertz, sdmmc_ck: Hertz) -> Result<(bool, u16, Hertz), Error> {
    match ker_ck.0.div_ceil(sdmmc_ck.0) {
        0 | 1 => Ok((false, 0, ker_ck)),
        x @ 2..=2046 => {
            // SDMMC_CK frequency = SDMMCCLK / [CLKDIV * 2]
            let clk_div = x.div_ceil(2) as u16;
            let clk = Hertz(ker_ck.0 / (clk_div as u32 * 2));

            Ok((false, clk_div, clk))
        }
        _ => Err(Error::BadClock),
    }
}

#[cfg(sdmmc_v1)]
type Transfer<'a> = crate::dma::Transfer<'a>;
#[cfg(any(sdmmc_v2, sdmmc_v3))]
struct Transfer<'a> {
    _dummy: PhantomData<&'a ()>,
}

struct WrappedTransfer<'a> {
    _transfer: Transfer<'a>,
    sdmmc: &'a Sdmmc<'a>,
    defused: bool,
}

impl<'a> WrappedTransfer<'a> {
    pub const fn new(_transfer: Transfer<'a>, sdmmc: &'a Sdmmc) -> Self {
        Self {
            _transfer,
            sdmmc,
            defused: false,
        }
    }

    pub fn defuse(&mut self) {
        self.defused = true;
    }
}

impl<'a> Drop for WrappedTransfer<'a> {
    fn drop(&mut self) {
        if !self.defused {
            self.sdmmc.on_drop();
        }
    }
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

/// Sdmmc device
pub struct Sdmmc<'d> {
    info: &'static Info,
    state: &'static State,
    ker_clk: Hertz,
    #[cfg(sdmmc_v1)]
    dma: ChannelAndRequest<'d>,

    _clk: Flex<'d>,
    _cmd: Flex<'d>,
    _d0: Flex<'d>,
    _d1: Option<Flex<'d>>,
    _d2: Option<Flex<'d>>,
    d3: Option<Flex<'d>>,
    _d4: Option<Flex<'d>>,
    _d5: Option<Flex<'d>>,
    _d6: Option<Flex<'d>>,
    d7: Option<Flex<'d>>,

    /// Optional level-shifter select pin for UHS-I 1.8V signalling.
    /// Driver toggles it during the CMD11 (VOLTAGE_SWITCH) handshake.
    /// The caller picks the initial level (3.3V) so polarity is per-board.
    /// Only present on `sdmmc_v3` (v1 lacks the matching POWER bits and
    /// v2 isn't validated yet) AND with `feature = "time"` (the
    /// voltage-switch state machine waits on hardware flags via
    /// `embassy_time::with_timeout`).
    #[cfg(sdmmc_uhs)]
    vswitch_pin: Option<Output<'d>>,

    /// `true` after a successful CMD11 voltage switch. `clkcr_set_clkdiv`
    /// reads this to decide whether to set `CLKCR.busspeed = 1` (1.8V
    /// UHS timing) on subsequent clock-divider updates.
    #[cfg(sdmmc_uhs)]
    uhs_active: bool,

    /// `true` after a successful UHS-SDR50 negotiation. SDR50 runs the
    /// bus at up to 100 MHz / 1.8V — at that speed the SDMMC peripheral
    /// must sample receive data on the CKIN feedback clock rather than
    /// the launch clock to recover the right timing margin. We set
    /// `CLKCR.selclkrx = 1` whenever this flag is on.
    #[cfg(sdmmc_uhs)]
    feedback_clk: bool,

    /// Optional CKIN feedback-clock input pin. `Some` only when the user
    /// constructed via `*_with_vswitch_ckin`. `acquire()` checks this
    /// at the SDR25→SDR50 decision: if absent, SDR50 isn't attempted
    /// (and `CLKCR.SELCLKRX` is left at 0 selecting `sdmmc_io_in_ck`).
    /// On chips/instances where the silicon doesn't expose CKIN (e.g.
    /// STM32N6 SDMMC2) no `CkinPin<T>` impl exists, so the
    /// `*_with_vswitch_ckin` constructor is uncallable and this field
    /// stays `None`.
    #[cfg(sdmmc_uhs)]
    ckin_pin: Option<Flex<'d>>,

    config: Config,
}

const CLK_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v1)]
const CMD_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v2)]
const CMD_AF: AfType = AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up);
const DATA_AF: AfType = CMD_AF;
/// CKIN is a feedback clock INPUT into the SDMMC peripheral (the chip
/// samples this signal). Configured as a high-speed input on the
/// peripheral's CKIN AF.
#[cfg(sdmmc_uhs)]
const CKIN_AF: AfType = AfType::input(Pull::None);

#[cfg(sdmmc_v1)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit<T: Instance, D: SdmmcDma<T>>(
        sdmmc: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma, _irq),
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit<T: Instance, D: SdmmcDma<T>>(
        sdmmc: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma, _irq),
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            None,
            None,
            None,
            None,
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }
}

#[cfg(sdmmc_v1)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit<T: Instance, D: SdmmcDma<T>>(
        sdmmc: Peri<'d, T>,
        dma: Peri<'d, D>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
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
        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma, _irq),
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            new_pin!(d4, DATA_AF),
            new_pin!(d5, DATA_AF),
            new_pin!(d6, DATA_AF),
            new_pin!(d7, DATA_AF),
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }
}

#[cfg(any(sdmmc_v2, sdmmc_v3))]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit<T: Instance>(
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
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            None,
            None,
            None,
            None,
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }
}

#[cfg(any(sdmmc_v2, sdmmc_v3))]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit<T: Instance>(
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
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            new_pin!(d4, DATA_AF),
            new_pin!(d5, DATA_AF),
            new_pin!(d6, DATA_AF),
            new_pin!(d7, DATA_AF),
            #[cfg(sdmmc_uhs)]
            None,
            #[cfg(sdmmc_uhs)]
            None,
            config,
        )
    }
}

// UHS-I (1.8V signalling) constructors. Only on `sdmmc_v3` with
// `feature = "time"` — see `new_1bit_with_vswitch` for the rationale.
#[cfg(sdmmc_uhs)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 1 data lane and a UHS-I level-
    /// shifter select pin.
    ///
    /// Only available on `sdmmc_v3` (STM32N6) with `feature = "time"`:
    /// other peripheral versions lack the required POWER/VSWITCH bits
    /// (v1) or are unvalidated (v2), and the switch handshake uses
    /// `embassy_time::with_timeout` to wait on hardware status flags.
    ///
    /// The caller pre-constructs an `Output` for the board's level-
    /// shifter select pin at its 3.3V level (polarity is per-board);
    /// the driver toggles it during the CMD11 handshake whenever the
    /// card accepts the S18A request on ACMD41.
    pub fn new_1bit_with_vswitch<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        vswitch: Output<'d>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vswitch),
            None,
            config,
        )
    }

    /// 4 data lanes; see [`Self::new_1bit_with_vswitch`].
    pub fn new_4bit_with_vswitch<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        vswitch: Output<'d>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            None,
            None,
            None,
            None,
            Some(vswitch),
            None,
            config,
        )
    }

    /// 1 data lane, plus UHS-I vswitch and a CKIN feedback-clock pin.
    ///
    /// Only callable on SDMMC instances whose silicon exposes a CKIN
    /// signal — a `CkinPin<T>` trait impl must exist for the passed-in
    /// pin. With this constructor the driver is allowed to negotiate
    /// UHS-SDR50 (CMD6 function group 1 ID 2, `CLKCR.SELCLKRX = 1`);
    /// without it, `acquire()` caps at SDR25.
    pub fn new_1bit_with_vswitch_ckin<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        vswitch: Output<'d>,
        ckin: Peri<'d, impl CkinPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vswitch),
            new_pin!(ckin, CKIN_AF),
            config,
        )
    }

    /// 4 data lanes; see [`Self::new_1bit_with_vswitch_ckin`].
    pub fn new_4bit_with_vswitch_ckin<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        vswitch: Output<'d>,
        ckin: Peri<'d, impl CkinPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            sdmmc,
            new_pin!(clk, CLK_AF).unwrap(),
            new_pin!(cmd, CMD_AF).unwrap(),
            new_pin!(d0, DATA_AF).unwrap(),
            new_pin!(d1, DATA_AF),
            new_pin!(d2, DATA_AF),
            new_pin!(d3, DATA_AF),
            None,
            None,
            None,
            None,
            Some(vswitch),
            new_pin!(ckin, CKIN_AF),
            config,
        )
    }
}

/// Tear-down guard for `voltage_switch`. On Drop with `armed = true`,
/// flips the level shifter back to 3.3V (when `pin_toggled = true`)
/// and clears the vswitch trigger bits, restoring the peripheral so
/// the caller can retry without UHS.
#[cfg(sdmmc_uhs)]
struct VswitchGuard<'a, 'd> {
    sdmmc: &'a mut Sdmmc<'d>,
    pin_toggled: bool,
    armed: bool,
}

#[cfg(sdmmc_uhs)]
impl Drop for VswitchGuard<'_, '_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.pin_toggled {
            if let Some(pin) = self.sdmmc.vswitch_pin.as_mut() {
                pin.toggle();
            }
        }
        self.sdmmc.info.regs.power().modify(|w| {
            w.set_vswitch(false);
            w.set_vswitchen(false);
        });
    }
}

impl<'d> Sdmmc<'d> {
    /// True if the driver has switched to UHS-I 1.8V signalling. Always
    /// `false` outside `cfg(sdmmc_uhs)`.
    pub(crate) fn uhs_active(&self) -> bool {
        #[cfg(sdmmc_uhs)]
        return self.uhs_active;
        #[cfg(not(sdmmc_uhs))]
        return false;
    }

    /// Set the CKIN feedback-clock sampling flag (`CLKCR.SELCLKRX`).
    /// Caller must follow with a `clkcr_set_clkdiv` to write the bit.
    /// No-op outside `cfg(sdmmc_uhs)`.
    pub(crate) fn set_feedback_clk(&mut self, on: bool) {
        #[cfg(sdmmc_uhs)]
        {
            self.feedback_clk = on;
        }
        #[cfg(not(sdmmc_uhs))]
        let _ = on;
    }

    /// True if this driver owns a UHS-I level-shifter pin. Always
    /// `false` outside `cfg(sdmmc_uhs)`.
    pub(crate) fn has_vswitch(&self) -> bool {
        #[cfg(sdmmc_uhs)]
        return self.vswitch_pin.is_some();
        #[cfg(not(sdmmc_uhs))]
        return false;
    }

    /// True if this driver owns a CKIN feedback-clock pin (gate for the
    /// SDR50 negotiation path). Always `false` outside `cfg(sdmmc_uhs)`.
    pub(crate) fn has_ckin(&self) -> bool {
        #[cfg(sdmmc_uhs)]
        return self.ckin_pin.is_some();
        #[cfg(not(sdmmc_uhs))]
        return false;
    }

    /// Restore the level-shifter pin to 3.3V and clear the UHS-related
    /// `CLKCR` bits. Called by `acquire()` so that re-init after a card
    /// swap starts from a known 3.3V / SDR12 state. No-op outside
    /// `cfg(sdmmc_uhs)`.
    pub(crate) fn reset_uhs_state(&mut self) {
        #[cfg(sdmmc_uhs)]
        {
            if self.uhs_active {
                if let Some(pin) = self.vswitch_pin.as_mut() {
                    pin.toggle();
                }
            }
            self.info.regs.clkcr().modify(|w| {
                w.set_busspeed(false);
                w.set_selclkrx(0);
            });
            self.uhs_active = false;
            self.feedback_clk = false;
        }
    }

    fn enable_interrupts(&self) {
        let regs = self.info.regs;
        critical_section::with(|_| {
            regs.maskr().modify(|w| {
                w.set_dcrcfailie(true);
                w.set_dtimeoutie(true);
                w.set_dataendie(true);
                w.set_dbckendie(true);

                #[cfg(sdmmc_v1)]
                w.set_stbiterre(true);
                #[cfg(any(sdmmc_v2, sdmmc_v3))]
                w.set_dabortie(true);
            });
        });
    }

    fn new_inner<T: Instance>(
        _sdmmc: Peri<'d, T>,
        #[cfg(sdmmc_v1)] dma: ChannelAndRequest<'d>,
        clk: Flex<'d>,
        cmd: Flex<'d>,
        d0: Flex<'d>,
        d1: Option<Flex<'d>>,
        d2: Option<Flex<'d>>,
        d3: Option<Flex<'d>>,
        d4: Option<Flex<'d>>,
        d5: Option<Flex<'d>>,
        d6: Option<Flex<'d>>,
        d7: Option<Flex<'d>>,
        #[cfg(sdmmc_uhs)] vswitch_pin: Option<Output<'d>>,
        #[cfg(sdmmc_uhs)] ckin_pin: Option<Flex<'d>>,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let info = T::info();
        let state = T::state();
        let ker_clk = T::frequency();

        info.regs.clkcr().write(|w| {
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
        info.regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::Off as u8));

        Self {
            info,
            state,
            ker_clk,
            #[cfg(sdmmc_v1)]
            dma,

            _clk: clk,
            _cmd: cmd,
            _d0: d0,
            _d1: d1,
            _d2: d2,
            d3,
            _d4: d4,
            _d5: d5,
            _d6: d6,
            d7,

            #[cfg(sdmmc_uhs)]
            vswitch_pin,
            #[cfg(sdmmc_uhs)]
            uhs_active: false,
            #[cfg(sdmmc_uhs)]
            feedback_clk: false,
            #[cfg(sdmmc_uhs)]
            ckin_pin,

            config,
        }
    }

    /// Data transfer is in progress
    #[inline]
    fn data_active(&self) -> bool {
        let regs = self.info.regs;

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.rxact() || status.txact();
        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        return status.dpsmact();
    }

    /// Coammand transfer is in progress
    #[inline]
    fn cmd_active(&self) -> bool {
        let regs = self.info.regs;

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.cmdact();
        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        return status.cpsmact();
    }

    /// Wait idle on CMDACT, RXACT and TXACT (v1) or DOSNACT and CPSMACT (v2)
    #[inline]
    fn wait_idle(&self) {
        while self.data_active() || self.cmd_active() {}
    }

    fn bus_width(&self) -> BusWidth {
        match (self.d3.is_some(), self.d7.is_some()) {
            (true, true) => BusWidth::Eight,
            (true, false) => BusWidth::Four,
            _ => BusWidth::One,
        }
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    #[allow(unused_variables)]
    fn prepare_datapath_read<'a>(
        &'a self,
        buffer: &'a mut Aligned<A4, [u8]>,
        mode: DatapathMode,
    ) -> WrappedTransfer<'a> {
        let regs = self.info.regs;

        let (byte_mode, block_size) = match mode {
            DatapathMode::Block(block_size) => (false, block_size as u8),
            DatapathMode::Byte => (true, 0),
        };

        // Command AND Data state machines must be idle
        self.wait_idle();
        self.clear_interrupt_flags();

        // Use buffer.len() not size_of_val: Aligned<A4, [u8]> is #[repr(C)]
        // with alignment 4, so size_of_val rounds up to a multiple of 4.
        regs.dlenr().write(|w| w.set_datalength(buffer.len() as u32));

        // SAFETY: No other functions use the dma
        #[cfg(sdmmc_v1)]
        let transfer = unsafe {
            self.dma
                .clone_unchecked()
                .read(
                    regs.fifor().as_ptr() as *mut u32,
                    slice32_mut(buffer),
                    DMA_TRANSFER_OPTIONS,
                )
                .unchecked_extend_lifetime()
        };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_mut_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };
        #[cfg(sdmmc_v3)]
        let transfer = {
            // v3 uses single-buffer IDMA: program base + size, then enable.
            regs.idmabaser().write(|w| w.set_idmabase(buffer.as_mut_ptr() as u32));
            regs.idmabsizer().write(|w| w.set_idmabndt(buffer.len() as u16));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        let byte_mode = byte_mode as u8;

        regs.dctrl().modify(|w| {
            w.set_dtmode(byte_mode);
            w.set_dblocksize(block_size as u8);
            w.set_dtdir(true);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        // Memory barrier after DMA setup to ensure register writes complete before command
        fence(Ordering::SeqCst);

        self.enable_interrupts();

        WrappedTransfer::new(transfer, &self)
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    fn prepare_datapath_write<'a>(&'a self, buffer: &'a Aligned<A4, [u8]>, mode: DatapathMode) -> WrappedTransfer<'a> {
        let regs = self.info.regs;

        let (byte_mode, block_size) = match mode {
            DatapathMode::Block(block_size) => (false, block_size as u8),
            DatapathMode::Byte => (true, 0),
        };

        // Command AND Data state machines must be idle
        self.wait_idle();
        self.clear_interrupt_flags();

        // Use buffer.len() not size_of_val: Aligned<A4, [u8]> is #[repr(C)]
        // with alignment 4, so size_of_val rounds up to a multiple of 4.
        regs.dlenr().write(|w| w.set_datalength(buffer.len() as u32));

        // SAFETY: No other functions use the dma
        #[cfg(sdmmc_v1)]
        let transfer = unsafe {
            self.dma
                .clone_unchecked()
                .write(
                    slice32_ref(buffer),
                    regs.fifor().as_ptr() as *mut u32,
                    DMA_TRANSFER_OPTIONS,
                )
                .unchecked_extend_lifetime()
        };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };
        #[cfg(sdmmc_v3)]
        let transfer = {
            regs.idmabaser().write(|w| w.set_idmabase(buffer.as_ptr() as u32));
            regs.idmabsizer().write(|w| w.set_idmabndt(buffer.len() as u16));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        let byte_mode = byte_mode as u8;

        regs.dctrl().modify(|w| {
            w.set_dtmode(byte_mode);
            w.set_dblocksize(block_size as u8);
            w.set_dtdir(false);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        // Memory barrier after DMA setup to ensure register writes complete before command
        fence(Ordering::SeqCst);

        self.enable_interrupts();

        WrappedTransfer::new(transfer, &self)
    }

    /// Stops the DMA datapath
    fn stop_datapath(&self) {
        let regs = self.info.regs;

        #[cfg(sdmmc_v1)]
        regs.dctrl().modify(|w| {
            w.set_dmaen(false);
            w.set_dten(false);
        });
        #[cfg(any(sdmmc_v2, sdmmc_v3))]
        regs.idmactrlr().modify(|w| w.set_idmaen(false));
    }

    fn init_idle(&mut self) -> Result<CommandResponse<Rz>, Error> {
        let regs = self.info.regs;

        self.clkcr_set_clkdiv(SD_INIT_FREQ, BusWidth::One)?;
        regs.dtimer()
            .write(|w| w.set_datatime(self.config.data_transfer_timeout));

        regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::On as u8));

        // Wait 74 cycles
        block_for_us((74_000_000 / SD_INIT_FREQ.0) as u64);

        self.cmd(common_cmd::idle(), true, false)
    }

    /// Sets the CLKDIV field in CLKCR. Updates clock field in self
    fn clkcr_set_clkdiv(&mut self, freq: Hertz, width: BusWidth) -> Result<(), Error> {
        let regs = self.info.regs;

        let (widbus, width_u32) = bus_width_vals(width);
        let (_bypass, clkdiv, new_clock) = clk_div(self.ker_clk, freq)?;

        trace!("sdmmc: set clock to {}", new_clock);

        // Enforce AHB and SDMMC_CK clock relation. See RM0433 Rev 7
        // Section 55.5.8
        let sdmmc_bus_bandwidth = new_clock.0 * width_u32;
        assert!(self.ker_clk.0 > 3 * sdmmc_bus_bandwidth / 32);

        // CPSMACT and DPSMACT must be 0 to set CLKDIV or WIDBUS
        self.wait_idle();
        #[cfg(sdmmc_uhs)]
        let self_uhs_active = self.uhs_active;
        #[cfg(sdmmc_uhs)]
        let self_feedback_clk = self.feedback_clk;
        regs.clkcr().modify(|w| {
            w.set_clkdiv(clkdiv);
            #[cfg(sdmmc_v1)]
            w.set_bypass(_bypass);
            w.set_widbus(widbus);
            // Re-assert busspeed only when UHS is currently active —
            // a clkcr_set_clkdiv call after the voltage-switch must
            // preserve busspeed=1 across the bus-speed bump. When UHS
            // is not active we leave busspeed completely alone (i.e.
            // do not write 0) to keep the non-UHS path 100 % bit-for-
            // bit identical to the pre-UHS HAL.
            #[cfg(sdmmc_uhs)]
            if self_uhs_active {
                w.set_busspeed(true);
            }
            // selclkrx = 1 selects CKIN feedback-clock sampling, which
            // is required for SDR50 (>50 MHz at 1.8V). For lower modes
            // we leave selclkrx at 0 (use SDMMC_CK directly).
            #[cfg(sdmmc_uhs)]
            if self_feedback_clk {
                w.set_selclkrx(1);
            }
        });

        Ok(())
    }

    /// Poll a status-register flag until it goes high or the timeout
    /// elapses. Used by `voltage_switch()` to wait on `CKSTOP` and
    /// `VSWEND` without busy-spinning the CPU.
    #[cfg(sdmmc_uhs)]
    async fn wait_status_flag(
        &self,
        timeout: embassy_time::Duration,
        check: impl Fn(crate::pac::sdmmc::regs::Star) -> bool,
    ) -> Result<(), Error> {
        let regs = self.info.regs;
        embassy_time::with_timeout(timeout, async {
            while !check(regs.star().read()) {
                embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;
            }
        })
        .await
        .map_err(|_| Error::VoltageSwitchFailed)
    }

    /// Run the UHS-I voltage-switch (CMD11) handshake against the card,
    /// drive the level-shifter pin to its 1.8V level, and mark the
    /// peripheral as UHS-active so subsequent `clkcr_set_clkdiv` calls
    /// set `CLKCR.busspeed = 1`.
    ///
    /// Caller MUST have verified the card responded to ACMD41 with
    /// `S18A = 1` AND that `self.has_vswitch()` is true. This routine
    /// must be called while the bus is at the 400 kHz init clock; see
    /// SD Physical Layer Spec §3.7.5 / RM0486 §30.8.
    ///
    /// On success, `self.uhs_active` is set and the level shifter is
    /// at 1.8V; the bus clock has been auto-restarted by hardware. On
    /// failure, returns `Error::VoltageSwitchFailed` with the level
    /// shifter restored to 3.3V so the caller can retry without UHS.
    #[cfg(sdmmc_uhs)]
    async fn voltage_switch(&mut self) -> Result<(), Error> {
        use embassy_time::Duration;

        // CKSTOP fires within microseconds of the CMD11 R1 ack at
        // 400 kHz; a 50 ms ceiling is generous.
        const CKSTOP_TIMEOUT: Duration = Duration::from_millis(50);
        // VSWEND fires after the hardware-managed 5 ms clock-low hold
        // plus the 1 ms post-restart sampling window — ~6 ms typical.
        const VSWEND_TIMEOUT: Duration = Duration::from_millis(50);

        let regs = self.info.regs;

        // Clear stale CKSTOP / VSWEND flags from a previous attempt and
        // arm the voltage-switch state machine. The next CMD11 R1 will
        // trigger the clock-stop + timed-hold sequence.
        regs.icr().write(|w| {
            w.set_ckstopc(true);
            w.set_vswendc(true);
        });
        regs.power().modify(|w| w.set_vswitchen(true));

        // RAII guard: on any early return below, restore the level
        // shifter to 3.3V (if we'd already toggled to 1.8V) and clear
        // the vswitch trigger bits.
        let mut guard = VswitchGuard {
            sdmmc: &mut *self,
            pin_toggled: false,
            armed: true,
        };

        // 1. Send CMD11.
        guard
            .sdmmc
            .cmd(sd_cmd::voltage_switch(), true, false)
            .map_err(|_| Error::VoltageSwitchFailed)?;

        // 2. Wait for CKSTOP — peripheral parked the clock low.
        guard.sdmmc.wait_status_flag(CKSTOP_TIMEOUT, |s| s.ckstop()).await?;
        regs.icr().write(|w| w.set_ckstopc(true));

        // 3. Toggle level-shifter pin to 1.8V (caller pre-set 3.3V).
        if let Some(pin) = guard.sdmmc.vswitch_pin.as_mut() {
            pin.toggle();
        }
        guard.pin_toggled = true;

        // 4. Release the clock-low hold. Hardware holds SDMMC_CK low
        //    for ≥5 ms then auto-restarts.
        regs.power().modify(|w| w.set_vswitch(true));

        // 5. Wait for VSWEND — peripheral re-sampled D0 after restart.
        guard.sdmmc.wait_status_flag(VSWEND_TIMEOUT, |s| s.vswend()).await?;
        regs.icr().write(|w| w.set_vswendc(true));

        // 6. STAR.BUSYD0 is the INVERTED level of D0 (per the v3 metapac
        //    docstring "Inverted value of SDMMC_D0 line (Busy)"):
        //      BUSYD0 = 1 → D0 LOW  → card still busy / refused switch
        //      BUSYD0 = 0 → D0 HIGH → card released D0 / handshake ok
        if regs.star().read().busyd0() {
            return Err(Error::VoltageSwitchFailed);
        }

        // Success — disarm the abort guard and finalise.
        guard.armed = false;
        drop(guard);

        // 7. Disarm the voltage-switch state machine. VSWITCHEN/VSWITCH
        //    are one-shot triggers, not mode bits — leaving them set
        //    has been observed to wedge the next command (e.g. CMD2).
        regs.power().modify(|w| {
            w.set_vswitch(false);
            w.set_vswitchen(false);
        });

        // 8. Mark UHS active and assert busspeed=1 right away. The next
        //    CMDs (CMD2/CMD3/CMD9/CMD7/...) fire before the next
        //    clkcr_set_clkdiv, and they need UHS timing on the 1.8V bus.
        //    CLKDIV / WIDBUS stay at their init_idle values (400 kHz,
        //    1-bit) — only the busspeed bit flips.
        self.uhs_active = true;
        regs.clkcr().modify(|w| w.set_busspeed(true));

        // 9. Settle. The clock just restarted and we just flipped
        //    busspeed; empirically the very next CPSM command hangs
        //    without a short delay. 1 ms is well above the internal
        //    resync window.
        embassy_time::Timer::after(Duration::from_millis(1)).await;
        Ok(())
    }

    fn get_cid(&self) -> Result<CommandResponse<R2>, Error> {
        self.cmd(common_cmd::all_send_cid(), true, false) // CMD2
    }

    fn get_csd(&self, address: u16) -> Result<CommandResponse<R2>, Error> {
        self.cmd(common_cmd::send_csd(address), true, false)
    }

    /// Query the card status (CMD13, returns R1)
    fn read_status(&self, address: u16) -> Result<CommandResponse<R1>, Error> {
        self.cmd(common_cmd::card_status(address, false), true, false)
    }

    /// Select one card and place it into the _Tranfer State_
    ///
    /// If `None` is specifed for `card`, all cards are put back into
    /// _Stand-by State_
    fn select_card(&self, rca: Option<u16>) -> Result<(), Error> {
        match self.cmd(common_cmd::select_card(rca.unwrap_or(0)), true, false) {
            Err(Error::Timeout) if rca == None => Ok(()),
            result => result.map(|_| ()),
        }
    }

    /// Clear flags in interrupt clear register
    #[inline]
    fn clear_interrupt_flags(&self) {
        let regs = self.info.regs;
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
            #[cfg(sdmmc_v1)]
            w.set_stbiterrc(true);

            #[cfg(any(sdmmc_v2, sdmmc_v3))]
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
    fn cmd<R: TypedResp>(&self, cmd: Cmd<R>, check_crc: bool, data: bool) -> Result<CommandResponse<R>, Error> {
        let regs = self.info.regs;

        self.clear_interrupt_flags();
        // CP state machine must be idle
        while self.cmd_active() {}

        // Command arg
        regs.argr().write(|w| w.set_cmdarg(cmd.arg));

        // Command index and start CP State Machine
        regs.cmdr().write(|w| {
            w.set_waitint(false);
            w.set_waitresp(get_waitresp_val(cmd.response_len()));
            w.set_cmdindex(cmd.cmd);
            w.set_cpsmen(true);

            #[cfg(any(sdmmc_v2, sdmmc_v3))]
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
            trace!("ctimeout: {}", cmd.cmd);

            return Err(Error::Timeout);
        } else if check_crc && status.ccrcfail() {
            return Err(Error::Crc);
        }

        Ok(CommandResponse(
            match R::LENGTH {
                ResponseLen::Zero => U128(0u128),
                ResponseLen::R48 => U128(self.info.regs.respr(0).read().cardstatus() as u128),
                ResponseLen::R136 => {
                    let cid0 = self.info.regs.respr(0).read().cardstatus() as u128;
                    let cid1 = self.info.regs.respr(1).read().cardstatus() as u128;
                    let cid2 = self.info.regs.respr(2).read().cardstatus() as u128;
                    let cid3 = self.info.regs.respr(3).read().cardstatus() as u128;

                    U128((cid0 << 96) | (cid1 << 64) | (cid2 << 32) | (cid3))
                }
            }
            .into(),
        ))
    }

    fn on_drop(&self) {
        let regs = self.info.regs;
        if self.data_active() {
            self.clear_interrupt_flags();
            // Send abort
            // CP state machine must be idle
            while self.cmd_active() {}

            // Command arg
            regs.argr().write(|w| w.set_cmdarg(0));

            // Command index and start CP State Machine
            regs.cmdr().write(|w| {
                w.set_waitint(false);
                w.set_waitresp(get_waitresp_val(ResponseLen::R48));
                w.set_cmdindex(12);
                w.set_cpsmen(true);

                #[cfg(any(sdmmc_v2, sdmmc_v3))]
                {
                    w.set_cmdstop(true);
                    w.set_cmdtrans(false);
                }
            });

            // Wait for the abort
            while self.data_active() {}
        }
        regs.maskr().write(|_| ()); // disable irqs
        self.clear_interrupt_flags();
        self.stop_datapath();
    }

    /// Wait for a previously started datapath transfer to complete from an interrupt.
    #[inline]
    #[allow(unused)]
    async fn complete_datapath_transfer(&self, mut transfer: WrappedTransfer<'_>, block: bool) -> Result<(), Error> {
        let res = poll_fn(|cx| {
            // Compiler might not be sufficiently constrained here
            // https://github.com/embassy-rs/embassy/issues/4723
            self.state.tx_waker.register(cx.waker());
            let status = self.info.regs.star().read();

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
            #[cfg(sdmmc_v1)]
            let done = match block {
                true => status.dbckend(),
                false => status.dataend(),
            };
            #[cfg(any(sdmmc_v2, sdmmc_v3))]
            let done = status.dataend();
            if done {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        // Memory barrier after DMA completion to ensure CPU sees DMA-written data
        fence(Ordering::Acquire);

        self.clear_interrupt_flags();
        self.stop_datapath();

        if !res.is_err() {
            transfer.defuse();
        }
        drop(transfer);

        res
    }
}

impl<'d> Drop for Sdmmc<'d> {
    fn drop(&mut self) {
        // T::Interrupt::disable();
        self.on_drop();
        self.info.rcc.disable();
    }
}

//////////////////////////////////////////////////////

type Regs = RegBlock;

struct Info {
    regs: Regs,
    rcc: RccInfo,
}

struct State {
    tx_waker: AtomicWaker,
    it_waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            it_waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// SDMMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(CkPin, Instance);
pin_trait!(CkinPin, Instance);
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
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: unsafe { Regs::from_ptr(crate::pac::$inst.as_ptr()) },
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$inst;
        }
    };
);
