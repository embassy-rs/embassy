use core::marker::PhantomData;
use core::task::{Context, Poll};

use embassy::util::{AtomicWaker, OnDrop, Unborrow};
use embassy_extras::unborrow;
use futures::future::poll_fn;
use sdio_host::{BusWidth, CardCapacity, CardStatus, CurrentState, SDStatus, CID, CSD, OCR, SCR};

use crate::fmt::*;
use crate::gpio::AnyPin;
use crate::interrupt;
use crate::pac;
use crate::pac::gpio::Gpio;
use crate::pac::interrupt::{Interrupt, InterruptEnum};
use crate::pac::sdmmc::Sdmmc as RegBlock;
use crate::time::Hertz;

/// The signalling scheme used on the SDMMC bus
#[non_exhaustive]
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

/// Errors
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Timeout,
    SoftwareTimeout,
    UnsupportedCardVersion,
    UnsupportedCardType,
    Crc,
    DataCrcFail,
    RxOverFlow,
    NoCard,
    BadClock,
    SignalingSwitchFailed,
    PeripheralBusy,
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

/// Indicates transfer direction
enum Dir {
    CardToHost,
    HostToCard,
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
/// Returns `(clk_div, clk_f)`, where `clk_div` is the divisor register
/// value and `clk_f` is the resulting new clock frequency.
fn clk_div(ker_ck: Hertz, sdmmc_ck: u32) -> Result<(u16, Hertz), Error> {
    match (ker_ck.0 + sdmmc_ck - 1) / sdmmc_ck {
        0 | 1 => Ok((0, ker_ck)),
        x @ 2..=2046 => {
            let clk_div = ((x + 1) / 2) as u16;
            let clk = Hertz(ker_ck.0 / (clk_div as u32 * 2));

            Ok((clk_div, clk))
        }
        _ => Err(Error::BadClock),
    }
}

struct SdmmcPins {
    clk: AnyPin,
    cmd: AnyPin,
    d0: AnyPin,
    d1: Option<AnyPin>,
    d2: Option<AnyPin>,
    d3: Option<AnyPin>,
}

impl SdmmcPins {
    /// # Safety
    ///
    /// Must have exclusive access to the gpio(s)' registers
    unsafe fn deconfigure(&mut self) {
        use crate::gpio::sealed::Pin as _;
        use crate::gpio::Pin;
        use pac::gpio::vals::Moder;

        let n = self.clk.pin().into();
        self.clk
            .block()
            .moder()
            .modify(|w| w.set_moder(n, Moder::ANALOG));

        let n = self.cmd.pin().into();
        self.cmd
            .block()
            .moder()
            .modify(|w| w.set_moder(n, Moder::ANALOG));

        let n = self.d0.pin().into();
        self.d0
            .block()
            .moder()
            .modify(|w| w.set_moder(n, Moder::ANALOG));

        if let Some(pin) = &self.d1 {
            let n = pin.pin().into();
            pin.block()
                .moder()
                .modify(|w| w.set_moder(n, Moder::ANALOG));
        }

        if let Some(pin) = &self.d2 {
            let n = pin.pin().into();
            pin.block()
                .moder()
                .modify(|w| w.set_moder(n, Moder::ANALOG));
        }

        if let Some(pin) = &self.d3 {
            let n = pin.pin().into();
            pin.block()
                .moder()
                .modify(|w| w.set_moder(n, Moder::ANALOG));
        }
    }
}

#[repr(transparent)]
pub struct DataBlock {
    pub buf: [u32; 128],
}

/// Sdmmc device
pub struct Sdmmc<'d> {
    sdmmc: PhantomData<&'d mut ()>,
    regs_addr: u32,
    /// SDMMC kernel clock
    ker_ck: Hertz,
    /// AHB clock
    hclk: Hertz,
    /// Data bus width
    bus_width: BusWidth,
    /// Current clock to card
    clock: Hertz,
    /// Current signalling scheme to card
    signalling: Signalling,
    /// Card
    card: Option<Card>,
    pins: SdmmcPins,
    interrupt_sdmmc1: bool,
}

impl<'d> Sdmmc<'d> {
    #[inline(always)]
    fn regs(&mut self) -> RegBlock {
        RegBlock(self.regs_addr as _)
    }

    /// # Safety
    ///
    /// Access to `block` registers should be exclusive
    unsafe fn configure_pin(block: Gpio, n: usize, afr_num: u8, pup: bool) {
        use pac::gpio::vals::{Afr, Moder, Ospeedr, Pupdr};

        let (afr, n_af) = if n < 8 { (0, n) } else { (1, n - 8) };
        block.afr(afr).modify(|w| w.set_afr(n_af, Afr(afr_num)));
        block.moder().modify(|w| w.set_moder(n, Moder::ALTERNATE));
        if pup {
            block.pupdr().modify(|w| w.set_pupdr(n, Pupdr::PULLUP));
        }
        block
            .ospeedr()
            .modify(|w| w.set_ospeedr(n, Ospeedr::VERYHIGHSPEED));
    }

    /// # Safety
    ///
    /// Access to `regs` registers should be exclusive
    unsafe fn new_inner(regs: RegBlock, kernel_clk: Hertz) -> Hertz {
        // While the SD/SDIO card or eMMC is in identification mode,
        // the SDMMC_CK frequency must be less than 400 kHz.
        let (clkdiv, clock) = unwrap!(clk_div(kernel_clk, 400_000));

        regs.clkcr().write(|w| {
            w.set_widbus(0);
            w.set_clkdiv(clkdiv);
            w.set_pwrsav(false);
            w.set_negedge(false);
            w.set_hwfc_en(true);
        });

        // Power off, writen 00: Clock to the card is stopped;
        // D[7:0], CMD, and CK are driven high.
        regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::Off as u8));

        clock
    }

    /// # Safety
    ///
    /// Futures that borrow this type can't be leaked
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn new_four_width<T, CLK, CMD, D0, D1, D2, D3>(
        _peripheral: impl Unborrow<Target = T> + 'd,
        interrupt: u16,
        clk_pin: impl Unborrow<Target = CLK> + 'd,
        cmd_pin: impl Unborrow<Target = CMD> + 'd,
        d0_pin: impl Unborrow<Target = D0> + 'd,
        d1_pin: impl Unborrow<Target = D1> + 'd,
        d2_pin: impl Unborrow<Target = D2> + 'd,
        d3_pin: impl Unborrow<Target = D3> + 'd,
        hclk: Hertz,
        kernel_clk: Hertz,
    ) -> Self
    where
        T: Instance,
        CLK: CkPin<T>,
        CMD: CmdPin<T>,
        D0: D0Pin<T>,
        D1: D1Pin<T>,
        D2: D2Pin<T>,
        D3: D3Pin<T>,
    {
        unborrow!(clk_pin, cmd_pin, d0_pin, d1_pin, d2_pin, d3_pin);

        // Configure Pins
        cortex_m::interrupt::free(|_| {
            // clk
            let block = clk_pin.block();
            let n = clk_pin.pin() as usize;
            let afr_num = CLK::AF_NUM;
            Self::configure_pin(block, n, afr_num, false);

            // cmd
            let block = cmd_pin.block();
            let n = cmd_pin.pin() as usize;
            let afr_num = CMD::AF_NUM;
            Self::configure_pin(block, n, afr_num, true);

            // d0
            let block = d0_pin.block();
            let n = d0_pin.pin() as usize;
            let afr_num = D0::AF_NUM;
            Self::configure_pin(block, n, afr_num, true);

            // d1
            let block = d1_pin.block();
            let n = d1_pin.pin() as usize;
            let afr_num = D1::AF_NUM;
            Self::configure_pin(block, n, afr_num, true);

            // d2
            let block = d2_pin.block();
            let n = d2_pin.pin() as usize;
            let afr_num = D2::AF_NUM;
            Self::configure_pin(block, n, afr_num, true);

            // d3
            let block = d3_pin.block();
            let n = d3_pin.pin() as usize;
            let afr_num = D3::AF_NUM;
            Self::configure_pin(block, n, afr_num, true);
        });

        let regs = RegBlock(T::ADDR as _);
        let clock = Self::new_inner(regs, kernel_clk);

        let pins = SdmmcPins {
            cmd: cmd_pin.degrade(),
            clk: clk_pin.degrade(),
            d0: d0_pin.degrade(),
            d1: Some(d1_pin.degrade()),
            d2: Some(d2_pin.degrade()),
            d3: Some(d3_pin.degrade()),
        };

        Self {
            sdmmc: PhantomData,
            regs_addr: T::ADDR,
            bus_width: BusWidth::Four,
            ker_ck: kernel_clk,
            hclk,
            clock,
            signalling: Default::default(),
            card: None,
            pins,
            interrupt_sdmmc1: interrupt == SDMMC1_INR,
        }
    }

    /// Initializes card (if present) and sets the bus at the
    /// specified frequency.
    pub async fn init_card(&mut self, freq: impl Into<Hertz>) -> Result<(), Error> {
        let freq = freq.into();
        let regs = self.regs();

        // NOTE(unsafe) We have exclusive access to the peripheral
        unsafe {
            regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::On as u8));
            self.cmd(Cmd::idle(), false)?;

            // Check if cards supports CMD8 (with pattern)
            self.cmd(Cmd::hs_send_ext_csd(0x1AA), false)?;
            let r1 = regs.respr(0).read().cardstatus1();

            let mut card = if r1 == 0x1AA {
                // Card echoed back the pattern. Must be at least v2
                Card::default()
            } else {
                return Err(Error::UnsupportedCardVersion);
            };

            let ocr = loop {
                // Signal that next command is a app command
                self.cmd(Cmd::app_cmd(0), false)?; // CMD55

                let arg = CmdAppOper::VOLTAGE_WINDOW_SD as u32
                    | CmdAppOper::HIGH_CAPACITY as u32
                    | CmdAppOper::SD_SWITCH_1_8V_CAPACITY as u32;

                // Initialize card
                match self.cmd(Cmd::app_op_cmd(arg), false) {
                    // ACMD41
                    Ok(_) => (),
                    Err(Error::Crc) => (),
                    Err(err) => return Err(err),
                }
                let ocr: OCR = regs.respr(0).read().cardstatus1().into();
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

            self.cmd(Cmd::all_send_cid(), false)?; // CMD2
            let cid0 = regs.respr(0).read().cardstatus1() as u128;
            let cid1 = regs.respr(1).read().cardstatus1() as u128;
            let cid2 = regs.respr(2).read().cardstatus1() as u128;
            let cid3 = regs.respr(3).read().cardstatus1() as u128;
            let cid = (cid0 << 96) | (cid1 << 64) | (cid2 << 32) | (cid3);
            card.cid = cid.into();

            self.cmd(Cmd::send_rel_addr(), false)?;
            card.rca = regs.respr(0).read().cardstatus1() >> 16;

            self.cmd(Cmd::send_csd(card.rca << 16), false)?;
            let csd0 = regs.respr(0).read().cardstatus1() as u128;
            let csd1 = regs.respr(1).read().cardstatus1() as u128;
            let csd2 = regs.respr(2).read().cardstatus1() as u128;
            let csd3 = regs.respr(3).read().cardstatus1() as u128;
            let csd = (csd0 << 96) | (csd1 << 64) | (csd2 << 32) | (csd3);
            card.csd = csd.into();

            self.select_card(Some(&card))?;
            self.get_scr(&mut card).await?;

            // Set bus width
            let (width, acmd_arg) = match self.bus_width {
                BusWidth::Eight => unimplemented!(),
                BusWidth::Four if card.scr.bus_width_four() => (BusWidth::Four, 2),
                _ => (BusWidth::One, 0),
            };
            self.cmd(Cmd::app_cmd(card.rca << 16), false)?;
            self.cmd(Cmd::cmd6(acmd_arg), false)?;

            // CPSMACT and DPSMACT must be 0 to set WIDBUS
            self.wait_idle();

            regs.clkcr().modify(|w| {
                w.set_widbus(match width {
                    BusWidth::One => 0,
                    BusWidth::Four => 1,
                    BusWidth::Eight => 2,
                    _ => self::panic!("Invalid Bus Width"),
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

            // Read status
            self.card.replace(card);
            self.read_sd_status().await?;

            if freq.0 > 25_000_000 {
                // Switch to SDR25
                self.signalling = self.switch_signalling_mode(Signalling::SDR25).await?;

                if self.signalling == Signalling::SDR25 {
                    // Set final clock frequency
                    self.clkcr_set_clkdiv(freq.0, width)?;

                    if self.read_status()?.state() != CurrentState::Transfer {
                        return Err(Error::SignalingSwitchFailed);
                    }
                }
            }
            // Read status after signalling change
            self.read_sd_status().await?;
        }
        Ok(())
    }

    /// Get a reference to the initialized card
    ///
    /// # Errors
    ///
    /// Returns Error::NoCard if [`init_card`](#method.init_card)
    /// has not previously succeeded
    pub fn card(&self) -> Result<&Card, Error> {
        self.card.as_ref().ok_or(Error::NoCard)
    }

    /// Get a mutable reference to the initialized card
    ///
    /// # Errors
    ///
    /// Returns Error::NoCard if [`init_card`](#method.init_card)
    /// has not previously succeeded
    pub fn card_mut(&mut self) -> Result<&mut Card, Error> {
        self.card.as_mut().ok_or(Error::NoCard)
    }

    pub async fn read_block(
        &mut self,
        block_idx: u32,
        buffer: &mut DataBlock,
    ) -> Result<(), Error> {
        self::todo!()
    }

    /// Get the current SDMMC bus clock
    pub fn clock(&self) -> Hertz {
        self.clock
    }

    /// Wait idle on DOSNACT and CPSMACT
    #[inline(always)]
    fn wait_idle(&mut self) {
        let regs = self.regs();

        // NOTE(unsafe) Atomic read with no side-effects
        unsafe {
            while {
                let status = regs.star().read();
                status.dpsmact() || status.cpsmact()
            } {}
        }
    }

    /// # Safety
    ///
    /// `buffer_addr` must be valid for the whole transfer and word aligned
    unsafe fn prepare_datapath_transfer(
        &mut self,
        buffer_addr: u32,
        length_bytes: u32,
        block_size: u8,
        direction: Dir,
    ) {
        self::assert!(block_size <= 14, "Block size up to 2^14 bytes");
        let regs = self.regs();

        let dtdir = match direction {
            Dir::CardToHost => true,
            Dir::HostToCard => false,
        };

        // Command AND Data state machines must be idle
        self.wait_idle();
        Self::clear_interrupt_flags(regs);

        // NOTE(unsafe) We have exclusive access to the regisers

        // TODO: Make this configurable
        regs.dtimer().write(|w| w.set_datatime(5_000_000));
        regs.dlenr().write(|w| w.set_datalength(length_bytes));

        regs.idmabase0r().write(|w| w.set_idmabase0(buffer_addr));
        regs.idmactrlr().modify(|w| w.set_idmaen(true));
        regs.dctrl().modify(|w| {
            w.set_dblocksize(block_size);
            w.set_dtdir(dtdir);
        });
    }

    /// Sets the CLKDIV field in CLKCR. Updates clock field in self
    fn clkcr_set_clkdiv(&mut self, freq: u32, width: BusWidth) -> Result<(), Error> {
        let regs = self.regs();

        let (clkdiv, new_clock) = clk_div(self.ker_ck, freq)?;
        // Enforce AHB and SDMMC_CK clock relation. See RM0433 Rev 7
        // Section 55.5.8
        let sdmmc_bus_bandwidth = new_clock.0 * (width as u32);
        self::assert!(self.hclk.0 > 3 * sdmmc_bus_bandwidth / 32);
        self.clock = new_clock;

        // NOTE(unsafe) We have exclusive access to the regblock
        unsafe {
            // CPSMACT and DPSMACT must be 0 to set CLKDIV
            self.wait_idle();
            regs.clkcr().modify(|w| w.set_clkdiv(clkdiv));
        }

        Ok(())
    }

    /// Switch mode using CMD6.
    ///
    /// Attempt to set a new signalling mode. The selected
    /// signalling mode is returned. Expects the current clock
    /// frequency to be > 12.5MHz.
    async fn switch_signalling_mode(
        &mut self,
        signalling: Signalling,
    ) -> Result<Signalling, Error> {
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
        let status_addr = &mut status as *mut [u32; 16] as u32;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = self.regs();
        let on_drop = OnDrop::new(move || unsafe { Self::on_drop(regs) });

        unsafe {
            self.prepare_datapath_transfer(status_addr, 64, 6, Dir::CardToHost);
            Self::data_interrupts(regs, true);
        }
        self.cmd(Cmd::cmd6(set_function), true)?; // CMD6

        let res = poll_fn(|cx| {
            self.store_waker_and_unmask(&cx);
            let status = unsafe { regs.star().read() };

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            } else if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            } else if status.dataend() {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        unsafe {
            Self::data_interrupts(regs, false);
        }
        Self::clear_interrupt_flags(regs);

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
                unsafe {
                    regs.idmactrlr().modify(|w| w.set_idmaen(false));
                }
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
    ///
    fn read_status(&mut self) -> Result<CardStatus, Error> {
        let regs = self.regs();
        let rca = self.card()?.rca;

        self.cmd(Cmd::card_status(rca << 16), false)?; // CMD13

        // NOTE(unsafe) Atomic read with no side-effects
        let r1 = unsafe { regs.respr(0).read().cardstatus1() };
        Ok(r1.into())
    }

    /// Reads the SD Status (ACMD13)
    async fn read_sd_status(&mut self) -> Result<(), Error> {
        let card = self.card()?;
        let rca = card.rca;
        self.cmd(Cmd::set_block_length(64), false)?; // CMD16
        self.cmd(Cmd::app_cmd(rca << 16), false)?; // APP

        let mut status = [0u32; 16];
        let status_addr = &mut status as *mut [u32; 16] as u32;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = self.regs();
        let on_drop = OnDrop::new(move || unsafe { Self::on_drop(regs) });

        unsafe {
            self.prepare_datapath_transfer(status_addr, 64, 6, Dir::CardToHost);
            Self::data_interrupts(regs, true);
        }
        self.cmd(Cmd::card_status(0), true)?;

        let res = poll_fn(|cx| {
            self.store_waker_and_unmask(&cx);
            let status = unsafe { regs.star().read() };

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            } else if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            } else if status.dataend() {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        unsafe {
            Self::data_interrupts(regs, false);
        }
        Self::clear_interrupt_flags(regs);

        if let Ok(_) = &res {
            on_drop.defuse();
            unsafe {
                regs.idmactrlr().modify(|w| w.set_idmaen(false));
            }
            for byte in status.iter_mut() {
                *byte = u32::from_be(*byte);
            }
            self.card_mut()?.status = status.into();
        }
        res
    }

    /// Select one card and place it into the _Tranfer State_
    ///
    /// If `None` is specifed for `card`, all cards are put back into
    /// _Stand-by State_
    fn select_card(&mut self, card: Option<&Card>) -> Result<(), Error> {
        // Determine Relative Card Address (RCA) of given card
        let rca = card.map(|c| c.rca << 16).unwrap_or(0);

        let r = self.cmd(Cmd::sel_desel_card(rca), false);
        match (r, rca) {
            (Err(Error::Timeout), 0) => Ok(()),
            _ => r,
        }
    }

    /// Clear flags in interrupt clear register
    #[inline(always)]
    fn clear_interrupt_flags(regs: RegBlock) {
        // NOTE(unsafe) Atomic write
        unsafe {
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
                w.set_dholdc(true);
                w.set_dbckendc(true);
                w.set_dabortc(true);
                w.set_busyd0endc(true);
                w.set_sdioitc(true);
                w.set_ackfailc(true);
                w.set_acktimeoutc(true);
                w.set_vswendc(true);
                w.set_ckstopc(true);
                w.set_idmatec(true);
                w.set_idmabtcc(true);
            });
        }
    }

    /// Enables the interrupts for data transfer
    ///
    /// # Safety
    ///
    /// Access to `regs` must be exclusive
    #[inline(always)]
    unsafe fn data_interrupts(regs: RegBlock, enable: bool) {
        regs.maskr().modify(|w| {
            w.set_dcrcfailie(enable);
            w.set_dtimeoutie(enable);
            w.set_dataendie(enable);
            w.set_dabortie(enable);
        });
    }

    async fn get_scr(&mut self, card: &mut Card) -> Result<(), Error> {
        // Read the the 64-bit SCR register
        self.cmd(Cmd::set_block_length(8), false)?; // CMD16
        self.cmd(Cmd::app_cmd(card.rca << 16), false)?;

        let mut scr = [0u32; 2];
        let scr_addr = &mut scr as *mut u32 as u32;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let regs = self.regs();
        let on_drop = OnDrop::new(move || unsafe { Self::on_drop(regs) });

        unsafe {
            self.prepare_datapath_transfer(scr_addr, 8, 3, Dir::CardToHost);
            Self::data_interrupts(regs, true);
        }
        self.cmd(Cmd::cmd51(), true)?;

        let res = poll_fn(|cx| {
            self.store_waker_and_unmask(&cx);
            let status = unsafe { regs.star().read() };

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            } else if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            } else if status.dataend() {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        unsafe {
            Self::data_interrupts(regs, false);
        }
        Self::clear_interrupt_flags(regs);

        if let Ok(_) = &res {
            on_drop.defuse();

            unsafe {
                regs.idmactrlr().modify(|w| w.set_idmaen(false));
                let scr_bytes = &*(&scr as *const [u32; 2] as *const [u8; 8]);
                card.scr = SCR(u64::from_be_bytes(*scr_bytes));
            }
        }
        res
    }

    /// Send command to card
    fn cmd(&mut self, cmd: Cmd, data: bool) -> Result<(), Error> {
        let regs = self.regs();

        Self::clear_interrupt_flags(regs);
        // NOTE(safety) Atomic operations
        unsafe {
            // CP state machine must be idle
            while regs.star().read().cpsmact() {}

            // Command arg
            regs.argr().write(|w| w.set_cmdarg(cmd.arg));

            // Special mode in CP State Machine
            // CMD12: Stop Transmission
            let cpsm_stop_transmission = cmd.cmd == 12;

            // Command index and start CP State Machine
            regs.cmdr().write(|w| {
                w.set_waitint(false);
                w.set_waitresp(cmd.resp as u8);
                w.set_cmdstop(cpsm_stop_transmission);
                w.set_cmdindex(cmd.cmd);
                w.set_cpsmen(true);
                w.set_cmdtrans(data);
            });

            // TODO: Check if this timeout is necessary
            let mut timeout: u32 = 0xFFFF_FFFF;

            let mut status;
            if cmd.resp == Response::None {
                // Wait for CMDSENT or a timeout
                while {
                    status = regs.star().read();
                    !(status.ctimeout() || status.cmdsent()) && timeout > 0
                } {
                    timeout -= 1;
                }
            } else {
                // Wait for CMDREND or CCRCFAIL or a timeout
                while {
                    status = regs.star().read();
                    !(status.ctimeout() || status.cmdrend() || status.ccrcfail()) && timeout > 0
                } {
                    timeout -= 1;
                }
            }

            if status.ctimeout() {
                return Err(Error::Timeout);
            } else if timeout == 0 {
                return Err(Error::SoftwareTimeout);
            } else if status.ccrcfail() {
                return Err(Error::Crc);
            }

            Ok(())
        }
    }

    fn store_waker_and_unmask(&self, cx: &Context) {
        use cortex_m::peripheral::NVIC;

        // NOTE(unsafe) We own the interrupt and can unmask it, it won't cause unsoundness
        unsafe {
            if self.interrupt_sdmmc1 {
                WAKER_1.register(cx.waker());
                NVIC::unmask(InterruptEnum::SDMMC1);
            } else {
                WAKER_2.register(cx.waker());
                NVIC::unmask(InterruptEnum::SDMMC2);
            }
        }
    }

    /// # Safety
    ///
    /// Ensure that `regs` has exclusive access to the regblocks
    unsafe fn on_drop(regs: RegBlock) {
        if regs.star().read().dpsmact() {
            // Send abort
            // CP state machine must be idle
            while regs.star().read().cpsmact() {}

            // Command arg
            regs.argr().write(|w| w.set_cmdarg(0));

            // Command index and start CP State Machine
            regs.cmdr().write(|w| {
                w.set_waitint(false);
                w.set_waitresp(Response::Short as u8);
                w.set_cmdstop(true);
                w.set_cmdindex(12);
                w.set_cpsmen(true);
                w.set_cmdtrans(false);
            });

            // Wait for the abort
            while regs.star().read().dpsmact() {}
        }
        Self::data_interrupts(regs, false);
        Self::clear_interrupt_flags(regs);
        regs.idmactrlr().modify(|w| w.set_idmaen(false));
    }
}

impl<'d> Drop for Sdmmc<'d> {
    fn drop(&mut self) {
        unsafe {
            Self::on_drop(self.regs());
        }

        // NOTE(unsafe) With `free` we will have exclusive access to the registers
        cortex_m::interrupt::free(|_| unsafe {
            self.pins.deconfigure();
        })
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
    const fn stop_transmission() -> Cmd {
        Cmd::new(12, 0, Response::Short)
    }

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
    const fn read_multiple_blocks(addr: u32) -> Cmd {
        Cmd::new(18, addr, Response::Short)
    }

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

const SDMMC1_INR: u16 = 49;
static WAKER_1: AtomicWaker = AtomicWaker::new();
static WAKER_2: AtomicWaker = AtomicWaker::new();

#[interrupt]
unsafe fn SDMMC1() {
    cortex_m::peripheral::NVIC::mask(InterruptEnum::SDMMC1);
    WAKER_1.wake();
}

#[cfg(feature = "2sdmmc")]
#[interrupt]
unsafe fn SDMMC2() {
    cortex_m::peripheral::NVIC::mask(InterruptEnum::SDMMC2);
    WAKER_2.wake();
}

pub(crate) mod sealed {
    use super::*;
    use crate::gpio::Pin as GpioPin;

    pub trait Instance {
        const ADDR: u32;
        type Interrupt: Interrupt;
    }
    pub trait CkPin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait CmdPin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D0Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D1Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D2Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D3Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D4Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D5Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D6Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
    pub trait D7Pin<T: Instance>: GpioPin {
        const AF_NUM: u8;
    }
}

pub trait Instance: sealed::Instance {}
pub trait CkPin<T: Instance>: sealed::CkPin<T> {}
pub trait CmdPin<T: Instance>: sealed::CmdPin<T> {}
pub trait D0Pin<T: Instance>: sealed::D0Pin<T> {}
pub trait D1Pin<T: Instance>: sealed::D1Pin<T> {}
pub trait D2Pin<T: Instance>: sealed::D2Pin<T> {}
pub trait D3Pin<T: Instance>: sealed::D3Pin<T> {}
pub trait D4Pin<T: Instance>: sealed::D4Pin<T> {}
pub trait D5Pin<T: Instance>: sealed::D5Pin<T> {}
pub trait D6Pin<T: Instance>: sealed::D6Pin<T> {}
pub trait D7Pin<T: Instance>: sealed::D7Pin<T> {}

macro_rules! impl_sdmmc {
    ($inst:ident, $addr:expr) => {
        impl crate::sdmmc_v2::sealed::Instance for peripherals::$inst {
            const ADDR: u32 = $addr;
            type Interrupt = interrupt::$inst;
        }

        impl crate::sdmmc_v2::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_sdmmc_pin {
    ($inst:ident, $func:ident, $pin:ident, $num:expr) => {
        impl crate::sdmmc_v2::sealed::$func<peripherals::$inst> for peripherals::$pin {
            const AF_NUM: u8 = $num;
        }

        impl crate::sdmmc_v2::$func<peripherals::$inst> for peripherals::$pin {}
    };
}
