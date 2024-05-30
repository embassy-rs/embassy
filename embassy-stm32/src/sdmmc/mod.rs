//! Secure Digital / MultiMedia Card (SDMMC)
#![macro_use]

use core::default::Default;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use sdio_host::{BusWidth, CardCapacity, CardStatus, CurrentState, SDStatus, CID, CSD, OCR, SCR};

use crate::dma::NoDma;
use crate::gpio::{AFType, AnyPin, Pull, SealedPin, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::sdmmc::Sdmmc as RegBlock;
use crate::rcc::{self, RccPeripheral};
use crate::time::Hertz;
use crate::{interrupt, peripherals, Peripheral};

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
    /// CRC error.
    Crc,
    /// No card inserted.
    NoCard,
    /// Bad clock supplied to the SDMMC peripheral.
    BadClock,
    /// Signaling switch failed.
    SignalingSwitchFailed,
    /// ST bit error.
    #[cfg(sdmmc_v1)]
    StBitErr,
}

/// A SD command
struct Cmd {
    cmd: u8,
    arg: u32,
    resp: Response,
}

#[derive(Clone, Copy, Debug, Default)]
/// SD Card
pub struct Card {
    /// The type of this card
    pub card_type: CardCapacity,
    /// Operation Conditions Register
    pub ocr: OCR,
    /// Relative Card Address
    pub rca: u32,
    /// Card ID
    pub cid: CID,
    /// Card Specific Data
    pub csd: CSD,
    /// SD CARD Configuration Register
    pub scr: SCR,
    /// SD Status
    pub status: SDStatus,
}

impl Card {
    /// Size in bytes
    pub fn size(&self) -> u64 {
        // SDHC / SDXC / SDUC
        u64::from(self.csd.block_count()) * 512
    }
}

#[repr(u8)]
enum PowerCtrl {
    Off = 0b00,
    On = 0b11,
}

#[repr(u32)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum CmdAppOper {
    VOLTAGE_WINDOW_SD = 0x8010_0000,
    HIGH_CAPACITY = 0x4000_0000,
    SDMMC_STD_CAPACITY = 0x0000_0000,
    SDMMC_CHECK_PATTERN = 0x0000_01AA,
    SD_SWITCH_1_8V_CAPACITY = 0x0100_0000,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Response {
    None = 0,
    Short = 1,
    Long = 3,
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

/// Sdmmc device
pub struct Sdmmc<'d, T: Instance, Dma: SdmmcDma<T> = NoDma> {
    _peri: PeripheralRef<'d, T>,
    #[allow(unused)]
    dma: PeripheralRef<'d, Dma>,

    clk: PeripheralRef<'d, AnyPin>,
    cmd: PeripheralRef<'d, AnyPin>,
    d0: PeripheralRef<'d, AnyPin>,
    d1: Option<PeripheralRef<'d, AnyPin>>,
    d2: Option<PeripheralRef<'d, AnyPin>>,
    d3: Option<PeripheralRef<'d, AnyPin>>,

    config: Config,
    /// Current clock to card
    clock: Hertz,
    /// Current signalling scheme to card
    signalling: Signalling,
    /// Card
    card: Option<Card>,
}

#[cfg(sdmmc_v1)]
impl<'d, T: Instance, Dma: SdmmcDma<T>> Sdmmc<'d, T, Dma> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit(
        sdmmc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        clk: impl Peripheral<P = impl CkPin<T>> + 'd,
        cmd: impl Peripheral<P = impl CmdPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, cmd, d0);

        critical_section::with(|_| {
            clk.set_as_af_pull(clk.af_num(), AFType::OutputPushPull, Pull::None);
            cmd.set_as_af_pull(cmd.af_num(), AFType::OutputPushPull, Pull::Up);
            d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::Up);

            clk.set_speed(Speed::VeryHigh);
            cmd.set_speed(Speed::VeryHigh);
            d0.set_speed(Speed::VeryHigh);
        });

        Self::new_inner(
            sdmmc,
            dma,
            clk.map_into(),
            cmd.map_into(),
            d0.map_into(),
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit(
        sdmmc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        clk: impl Peripheral<P = impl CkPin<T>> + 'd,
        cmd: impl Peripheral<P = impl CmdPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, cmd, d0, d1, d2, d3);

        critical_section::with(|_| {
            clk.set_as_af_pull(clk.af_num(), AFType::OutputPushPull, Pull::None);
            cmd.set_as_af_pull(cmd.af_num(), AFType::OutputPushPull, Pull::Up);
            d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::Up);
            d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::Up);
            d2.set_as_af_pull(d2.af_num(), AFType::OutputPushPull, Pull::Up);
            d3.set_as_af_pull(d3.af_num(), AFType::OutputPushPull, Pull::Up);

            clk.set_speed(Speed::VeryHigh);
            cmd.set_speed(Speed::VeryHigh);
            d0.set_speed(Speed::VeryHigh);
            d1.set_speed(Speed::VeryHigh);
            d2.set_speed(Speed::VeryHigh);
            d3.set_speed(Speed::VeryHigh);
        });

        Self::new_inner(
            sdmmc,
            dma,
            clk.map_into(),
            cmd.map_into(),
            d0.map_into(),
            Some(d1.map_into()),
            Some(d2.map_into()),
            Some(d3.map_into()),
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
impl<'d, T: Instance> Sdmmc<'d, T, NoDma> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit(
        sdmmc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: impl Peripheral<P = impl CkPin<T>> + 'd,
        cmd: impl Peripheral<P = impl CmdPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, cmd, d0);

        critical_section::with(|_| {
            clk.set_as_af_pull(clk.af_num(), AFType::OutputPushPull, Pull::None);
            cmd.set_as_af_pull(cmd.af_num(), AFType::OutputPushPull, Pull::Up);
            d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::Up);

            clk.set_speed(Speed::VeryHigh);
            cmd.set_speed(Speed::VeryHigh);
            d0.set_speed(Speed::VeryHigh);
        });

        Self::new_inner(
            sdmmc,
            NoDma.into_ref(),
            clk.map_into(),
            cmd.map_into(),
            d0.map_into(),
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit(
        sdmmc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: impl Peripheral<P = impl CkPin<T>> + 'd,
        cmd: impl Peripheral<P = impl CmdPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(clk, cmd, d0, d1, d2, d3);

        critical_section::with(|_| {
            clk.set_as_af_pull(clk.af_num(), AFType::OutputPushPull, Pull::None);
            cmd.set_as_af_pull(cmd.af_num(), AFType::OutputPushPull, Pull::Up);
            d0.set_as_af_pull(d0.af_num(), AFType::OutputPushPull, Pull::Up);
            d1.set_as_af_pull(d1.af_num(), AFType::OutputPushPull, Pull::Up);
            d2.set_as_af_pull(d2.af_num(), AFType::OutputPushPull, Pull::Up);
            d3.set_as_af_pull(d3.af_num(), AFType::OutputPushPull, Pull::Up);

            clk.set_speed(Speed::VeryHigh);
            cmd.set_speed(Speed::VeryHigh);
            d0.set_speed(Speed::VeryHigh);
            d1.set_speed(Speed::VeryHigh);
            d2.set_speed(Speed::VeryHigh);
            d3.set_speed(Speed::VeryHigh);
        });

        Self::new_inner(
            sdmmc,
            NoDma.into_ref(),
            clk.map_into(),
            cmd.map_into(),
            d0.map_into(),
            Some(d1.map_into()),
            Some(d2.map_into()),
            Some(d3.map_into()),
            config,
        )
    }
}

impl<'d, T: Instance, Dma: SdmmcDma<T> + 'd> Sdmmc<'d, T, Dma> {
    fn new_inner(
        sdmmc: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        clk: PeripheralRef<'d, AnyPin>,
        cmd: PeripheralRef<'d, AnyPin>,
        d0: PeripheralRef<'d, AnyPin>,
        d1: Option<PeripheralRef<'d, AnyPin>>,
        d2: Option<PeripheralRef<'d, AnyPin>>,
        d3: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(sdmmc, dma);

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
            dma,

            clk,
            cmd,
            d0,
            d1,
            d2,
            d3,

            config,
            clock: SD_INIT_FREQ,
            signalling: Default::default(),
            card: None,
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
    fn prepare_datapath_read<'a>(
        &'a mut self,
        buffer: &'a mut [u32],
        length_bytes: u32,
        block_size: u8,
    ) -> Transfer<'a> {
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
            let request = self.dma.request();
            Transfer::new_read(
                &mut self.dma,
                request,
                regs.fifor().as_ptr() as *mut u32,
                buffer,
                DMA_TRANSFER_OPTIONS,
            )
        };
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
            let request = self.dma.request();
            Transfer::new_write(
                &mut self.dma,
                request,
                buffer,
                regs.fifor().as_ptr() as *mut u32,
                DMA_TRANSFER_OPTIONS,
            )
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

    /// Switch mode using CMD6.
    ///
    /// Attempt to set a new signalling mode. The selected
    /// signalling mode is returned. Expects the current clock
    /// frequency to be > 12.5MHz.
    async fn switch_signalling_mode(&mut self, signalling: Signalling) -> Result<Signalling, Error> {
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

        let mut status = [0u32; 16];

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = self.prepare_datapath_read(&mut status, 64, 6);
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(Cmd::cmd6(set_function), true)?; // CMD6

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
        Self::clear_interrupt_flags();

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

    /// Query the card status (CMD13, returns R1)
    fn read_status(&self, card: &Card) -> Result<CardStatus, Error> {
        let regs = T::regs();
        let rca = card.rca;

        Self::cmd(Cmd::card_status(rca << 16), false)?; // CMD13

        let r1 = regs.respr(0).read().cardstatus();
        Ok(r1.into())
    }

    /// Reads the SD Status (ACMD13)
    async fn read_sd_status(&mut self) -> Result<(), Error> {
        let card = self.card.as_mut().ok_or(Error::NoCard)?;
        let rca = card.rca;

        Self::cmd(Cmd::set_block_length(64), false)?; // CMD16
        Self::cmd(Cmd::app_cmd(rca << 16), false)?; // APP

        let mut status = [0u32; 16];

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = self.prepare_datapath_read(&mut status, 64, 6);
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(Cmd::card_status(0), true)?;

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
        Self::clear_interrupt_flags();

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);

            for byte in status.iter_mut() {
                *byte = u32::from_be(*byte);
            }
            self.card.as_mut().unwrap().status = status.into();
        }
        res
    }

    /// Select one card and place it into the _Tranfer State_
    ///
    /// If `None` is specifed for `card`, all cards are put back into
    /// _Stand-by State_
    fn select_card(&self, card: Option<&Card>) -> Result<(), Error> {
        // Determine Relative Card Address (RCA) of given card
        let rca = card.map(|c| c.rca << 16).unwrap_or(0);

        let r = Self::cmd(Cmd::sel_desel_card(rca), false);
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

    async fn get_scr(&mut self, card: &mut Card) -> Result<(), Error> {
        // Read the the 64-bit SCR register
        Self::cmd(Cmd::set_block_length(8), false)?; // CMD16
        Self::cmd(Cmd::app_cmd(card.rca << 16), false)?;

        let mut scr = [0u32; 2];

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = self.prepare_datapath_read(&mut scr[..], 8, 3);
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(Cmd::cmd51(), true)?;

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
        Self::clear_interrupt_flags();

        if res.is_ok() {
            on_drop.defuse();
            Self::stop_datapath();
            drop(transfer);

            unsafe {
                let scr_bytes = &*(&scr as *const [u32; 2] as *const [u8; 8]);
                card.scr = SCR(u64::from_be_bytes(*scr_bytes));
            }
        }
        res
    }

    /// Send command to card
    #[allow(unused_variables)]
    fn cmd(cmd: Cmd, data: bool) -> Result<(), Error> {
        let regs = T::regs();

        Self::clear_interrupt_flags();
        // CP state machine must be idle
        while Self::cmd_active() {}

        // Command arg
        regs.argr().write(|w| w.set_cmdarg(cmd.arg));

        // Command index and start CP State Machine
        regs.cmdr().write(|w| {
            w.set_waitint(false);
            w.set_waitresp(cmd.resp as u8);
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
        if cmd.resp == Response::None {
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
                w.set_waitresp(Response::Short as u8);
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

    /// Initializes card (if present) and sets the bus at the
    /// specified frequency.
    pub async fn init_card(&mut self, freq: Hertz) -> Result<(), Error> {
        let regs = T::regs();
        let ker_ck = T::frequency();

        let bus_width = match self.d3.is_some() {
            true => BusWidth::Four,
            false => BusWidth::One,
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
        Self::cmd(Cmd::idle(), false)?;

        // Check if cards supports CMD8 (with pattern)
        Self::cmd(Cmd::hs_send_ext_csd(0x1AA), false)?;
        let r1 = regs.respr(0).read().cardstatus();

        let mut card = if r1 == 0x1AA {
            // Card echoed back the pattern. Must be at least v2
            Card::default()
        } else {
            return Err(Error::UnsupportedCardVersion);
        };

        let ocr = loop {
            // Signal that next command is a app command
            Self::cmd(Cmd::app_cmd(0), false)?; // CMD55

            let arg = CmdAppOper::VOLTAGE_WINDOW_SD as u32
                | CmdAppOper::HIGH_CAPACITY as u32
                | CmdAppOper::SD_SWITCH_1_8V_CAPACITY as u32;

            // Initialize card
            match Self::cmd(Cmd::app_op_cmd(arg), false) {
                // ACMD41
                Ok(_) => (),
                Err(Error::Crc) => (),
                Err(err) => return Err(err),
            }
            let ocr: OCR = regs.respr(0).read().cardstatus().into();
            if !ocr.is_busy() {
                // Power up done
                break ocr;
            }
        };

        if ocr.high_capacity() {
            // Card is SDHC or SDXC or SDUC
            card.card_type = CardCapacity::SDHC;
        } else {
            card.card_type = CardCapacity::SDSC;
        }
        card.ocr = ocr;

        Self::cmd(Cmd::all_send_cid(), false)?; // CMD2
        let cid0 = regs.respr(0).read().cardstatus() as u128;
        let cid1 = regs.respr(1).read().cardstatus() as u128;
        let cid2 = regs.respr(2).read().cardstatus() as u128;
        let cid3 = regs.respr(3).read().cardstatus() as u128;
        let cid = (cid0 << 96) | (cid1 << 64) | (cid2 << 32) | (cid3);
        card.cid = cid.into();

        Self::cmd(Cmd::send_rel_addr(), false)?;
        card.rca = regs.respr(0).read().cardstatus() >> 16;

        Self::cmd(Cmd::send_csd(card.rca << 16), false)?;
        let csd0 = regs.respr(0).read().cardstatus() as u128;
        let csd1 = regs.respr(1).read().cardstatus() as u128;
        let csd2 = regs.respr(2).read().cardstatus() as u128;
        let csd3 = regs.respr(3).read().cardstatus() as u128;
        let csd = (csd0 << 96) | (csd1 << 64) | (csd2 << 32) | (csd3);
        card.csd = csd.into();

        self.select_card(Some(&card))?;

        self.get_scr(&mut card).await?;

        // Set bus width
        let (width, acmd_arg) = match bus_width {
            BusWidth::Eight => unimplemented!(),
            BusWidth::Four if card.scr.bus_width_four() => (BusWidth::Four, 2),
            _ => (BusWidth::One, 0),
        };
        Self::cmd(Cmd::app_cmd(card.rca << 16), false)?;
        Self::cmd(Cmd::cmd6(acmd_arg), false)?;

        // CPSMACT and DPSMACT must be 0 to set WIDBUS
        Self::wait_idle();

        regs.clkcr().modify(|w| {
            w.set_widbus(match width {
                BusWidth::One => 0,
                BusWidth::Four => 1,
                BusWidth::Eight => 2,
                _ => panic!("Invalid Bus Width"),
            })
        });

        // Set Clock
        if freq.0 <= 25_000_000 {
            // Final clock frequency
            self.clkcr_set_clkdiv(freq.0, width)?;
        } else {
            // Switch to max clock for SDR12
            self.clkcr_set_clkdiv(25_000_000, width)?;
        }

        self.card = Some(card);

        // Read status
        self.read_sd_status().await?;

        if freq.0 > 25_000_000 {
            // Switch to SDR25
            self.signalling = self.switch_signalling_mode(Signalling::SDR25).await?;

            if self.signalling == Signalling::SDR25 {
                // Set final clock frequency
                self.clkcr_set_clkdiv(freq.0, width)?;

                if self.read_status(&card)?.state() != CurrentState::Transfer {
                    return Err(Error::SignalingSwitchFailed);
                }
            }
        }
        // Read status after signalling change
        self.read_sd_status().await?;

        Ok(())
    }

    /// Read a data block.
    #[inline]
    pub async fn read_block(&mut self, block_idx: u32, buffer: &mut DataBlock) -> Result<(), Error> {
        let card_capacity = self.card()?.card_type;

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &mut *((&mut buffer.0) as *mut [u8; 512] as *mut [u32; 128]) };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::SDSC => block_idx * 512,
            _ => block_idx,
        };
        Self::cmd(Cmd::set_block_length(512), false)?; // CMD16

        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        let transfer = self.prepare_datapath_read(buffer, 512, 9);
        InterruptHandler::<T>::data_interrupts(true);
        Self::cmd(Cmd::read_single_block(address), true)?;

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
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card.card_type {
            CardCapacity::SDSC => block_idx * 512,
            _ => block_idx,
        };
        Self::cmd(Cmd::set_block_length(512), false)?; // CMD16

        let regs = T::regs();
        let on_drop = OnDrop::new(|| Self::on_drop());

        // sdmmc_v1 uses different cmd/dma order than v2, but only for writes
        #[cfg(sdmmc_v1)]
        Self::cmd(Cmd::write_single_block(address), true)?;

        let transfer = self.prepare_datapath_write(buffer, 512, 9);
        InterruptHandler::<T>::data_interrupts(true);

        #[cfg(sdmmc_v2)]
        Self::cmd(Cmd::write_single_block(address), true)?;

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
    /// Returns Error::NoCard if [`init_card`](#method.init_card)
    /// has not previously succeeded
    #[inline]
    pub fn card(&self) -> Result<&Card, Error> {
        self.card.as_ref().ok_or(Error::NoCard)
    }

    /// Get the current SDMMC bus clock
    pub fn clock(&self) -> Hertz {
        self.clock
    }
}

impl<'d, T: Instance, Dma: SdmmcDma<T> + 'd> Drop for Sdmmc<'d, T, Dma> {
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
        });
    }
}

/// SD card Commands
impl Cmd {
    const fn new(cmd: u8, arg: u32, resp: Response) -> Cmd {
        Cmd { cmd, arg, resp }
    }

    /// CMD0: Idle
    const fn idle() -> Cmd {
        Cmd::new(0, 0, Response::None)
    }

    /// CMD2: Send CID
    const fn all_send_cid() -> Cmd {
        Cmd::new(2, 0, Response::Long)
    }

    /// CMD3: Send Relative Address
    const fn send_rel_addr() -> Cmd {
        Cmd::new(3, 0, Response::Short)
    }

    /// CMD6: Switch Function Command
    /// ACMD6: Bus Width
    const fn cmd6(arg: u32) -> Cmd {
        Cmd::new(6, arg, Response::Short)
    }

    /// CMD7: Select one card and put it into the _Tranfer State_
    const fn sel_desel_card(rca: u32) -> Cmd {
        Cmd::new(7, rca, Response::Short)
    }

    /// CMD8:
    const fn hs_send_ext_csd(arg: u32) -> Cmd {
        Cmd::new(8, arg, Response::Short)
    }

    /// CMD9:
    const fn send_csd(rca: u32) -> Cmd {
        Cmd::new(9, rca, Response::Long)
    }

    /// CMD12:
    //const fn stop_transmission() -> Cmd {
    //    Cmd::new(12, 0, Response::Short)
    //}

    /// CMD13: Ask card to send status register
    /// ACMD13: SD Status
    const fn card_status(rca: u32) -> Cmd {
        Cmd::new(13, rca, Response::Short)
    }

    /// CMD16:
    const fn set_block_length(blocklen: u32) -> Cmd {
        Cmd::new(16, blocklen, Response::Short)
    }

    /// CMD17: Block Read
    const fn read_single_block(addr: u32) -> Cmd {
        Cmd::new(17, addr, Response::Short)
    }

    /// CMD18: Multiple Block Read
    //const fn read_multiple_blocks(addr: u32) -> Cmd {
    //    Cmd::new(18, addr, Response::Short)
    //}

    /// CMD24: Block Write
    const fn write_single_block(addr: u32) -> Cmd {
        Cmd::new(24, addr, Response::Short)
    }

    const fn app_op_cmd(arg: u32) -> Cmd {
        Cmd::new(41, arg, Response::Short)
    }

    const fn cmd51() -> Cmd {
        Cmd::new(51, 0, Response::Short)
    }

    /// App Command. Indicates that next command will be a app command
    const fn app_cmd(rca: u32) -> Cmd {
        Cmd::new(55, rca, Response::Short)
    }
}

//////////////////////////////////////////////////////

trait SealedInstance {
    fn regs() -> RegBlock;
    fn state() -> &'static AtomicWaker;
}

/// SDMMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
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

/// DMA instance trait.
///
/// This is only implemented for `NoDma`, since SDMMCv2 has DMA built-in, instead of
/// using ST's system-wide DMA peripheral.
#[cfg(sdmmc_v2)]
pub trait SdmmcDma<T: Instance> {}
#[cfg(sdmmc_v2)]
impl<T: Instance> SdmmcDma<T> for NoDma {}

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
