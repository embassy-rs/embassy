//! Secure Digital / MultiMedia Card (SDMMC)
#![macro_use]

use core::default::Default;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::slice;
use core::task::Poll;

use aligned::{A4, Aligned};
use cortex_m::asm::dsb;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use sdio_host::Cmd;
use sdio_host::common_cmd::{self, R1, R2, R3, Resp, ResponseLen, Rz};
use sdio_host::sd::{BusWidth, CID, CSD, CardStatus, OCR, RCA};
use sdio_host::sd_cmd::{R6, R7};

#[cfg(sdmmc_v1)]
use crate::dma::ChannelAndRequest;
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

        #[cfg(sdmmc_v2)]
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
            #[cfg(sdmmc_v2)]
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

impl TypedResp for R3 {
    type Word = u32;
}

impl<E> From<CommandResponse<R3>> for OCR<E> {
    fn from(value: CommandResponse<R3>) -> Self {
        OCR::<E>::from(value.0)
    }
}

impl TypedResp for R6 {
    type Word = u32;
}

impl<E> From<CommandResponse<R6>> for RCA<E> {
    fn from(value: CommandResponse<R6>) -> Self {
        RCA::<E>::from(value.0)
    }
}

impl TypedResp for R7 {
    type Word = u32;
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
#[cfg(sdmmc_v2)]
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
#[cfg(sdmmc_v2)]
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

    config: Config,
}

const CLK_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v1)]
const CMD_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v2)]
const CMD_AF: AfType = AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up);
const DATA_AF: AfType = CMD_AF;

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
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
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
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
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
            config,
        )
    }
}

impl<'d> Sdmmc<'d> {
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
                #[cfg(sdmmc_v2)]
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
        config: Config,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<T>();

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
        #[cfg(sdmmc_v2)]
        return status.dpsmact();
    }

    /// Coammand transfer is in progress
    #[inline]
    fn cmd_active(&self) -> bool {
        let regs = self.info.regs;

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.cmdact();
        #[cfg(sdmmc_v2)]
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

        regs.dlenr().write(|w| w.set_datalength(size_of_val(buffer) as u32));

        // Memory barrier before DMA setup to ensure any pending memory writes complete
        dsb();

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

        #[cfg(sdmmc_v2)]
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
        dsb();

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

        regs.dlenr().write(|w| w.set_datalength(size_of_val(buffer) as u32));

        // Memory barrier before DMA setup to ensure buffer data is visible to DMA
        dsb();

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

        #[cfg(sdmmc_v2)]
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
        dsb();

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
        #[cfg(sdmmc_v2)]
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
        regs.clkcr().modify(|w| {
            w.set_clkdiv(clkdiv);
            #[cfg(sdmmc_v1)]
            w.set_bypass(_bypass);
            w.set_widbus(widbus);
        });

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

                #[cfg(sdmmc_v2)]
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
            #[cfg(sdmmc_v2)]
            let done = status.dataend();
            if done {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        // Memory barrier after DMA completion to ensure CPU sees DMA-written data
        dsb();

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
        self.info.rcc.disable_without_stop();
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
