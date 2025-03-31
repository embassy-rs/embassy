//! PIO driver.
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use atomic_polyfill::{AtomicU64, AtomicU8};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use fixed::types::extra::U8;
use fixed::FixedU32;
use pio::{Program, SideSet, Wrap};

use crate::dma::{Channel, Transfer, Word};
use crate::gpio::{self, AnyPin, Drive, Level, Pull, SealedPin, SlewRate};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::relocate::RelocatedProgram;
use crate::{pac, peripherals, RegExt};

mod instr;

#[doc(inline)]
pub use pio as program;

/// Wakers for interrupts and FIFOs.
pub struct Wakers([AtomicWaker; 12]);

impl Wakers {
    #[inline(always)]
    fn fifo_in(&self) -> &[AtomicWaker] {
        &self.0[0..4]
    }
    #[inline(always)]
    fn fifo_out(&self) -> &[AtomicWaker] {
        &self.0[4..8]
    }
    #[inline(always)]
    fn irq(&self) -> &[AtomicWaker] {
        &self.0[8..12]
    }
}

/// FIFO config.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FifoJoin {
    /// Both TX and RX fifo is enabled
    #[default]
    Duplex,
    /// Rx fifo twice as deep. TX fifo disabled
    RxOnly,
    /// Tx fifo twice as deep. RX fifo disabled
    TxOnly,
    /// Enable random writes (`FJOIN_RX_PUT`) from the state machine (through ISR),
    /// and random reads from the system (using [`StateMachine::get_rxf_entry`]).
    #[cfg(feature = "_rp235x")]
    RxAsStatus,
    /// Enable random reads (`FJOIN_RX_GET`) from the state machine (through OSR),
    /// and random writes from the system (using [`StateMachine::set_rxf_entry`]).
    #[cfg(feature = "_rp235x")]
    RxAsControl,
    /// FJOIN_RX_PUT | FJOIN_RX_GET: RX can be used as a scratch register,
    /// not accesible from the CPU
    #[cfg(feature = "_rp235x")]
    PioScratch,
}

/// Shift direction.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ShiftDirection {
    #[default]
    Right = 1,
    Left = 0,
}

/// Pin direction.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Direction {
    In = 0,
    Out = 1,
}

/// Which fifo level to use in status check.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum StatusSource {
    #[default]
    /// All-ones if TX FIFO level < N, otherwise all-zeroes.
    TxFifoLevel = 0,
    /// All-ones if RX FIFO level < N, otherwise all-zeroes.
    RxFifoLevel = 1,
}

const RXNEMPTY_MASK: u32 = 1 << 0;
const TXNFULL_MASK: u32 = 1 << 4;
const SMIRQ_MASK: u32 = 1 << 8;

/// Interrupt handler for PIO.
pub struct InterruptHandler<PIO: Instance> {
    _pio: PhantomData<PIO>,
}

impl<PIO: Instance> Handler<PIO::Interrupt> for InterruptHandler<PIO> {
    unsafe fn on_interrupt() {
        let ints = PIO::PIO.irqs(0).ints().read().0;
        for bit in 0..12 {
            if ints & (1 << bit) != 0 {
                PIO::wakers().0[bit].wake();
            }
        }
        PIO::PIO.irqs(0).inte().write_clear(|m| m.0 = ints);
    }
}

/// Future that waits for TX-FIFO to become writable
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoOutFuture<'a, 'd, PIO: Instance, const SM: usize> {
    sm_tx: &'a mut StateMachineTx<'d, PIO, SM>,
    value: u32,
}

impl<'a, 'd, PIO: Instance, const SM: usize> FifoOutFuture<'a, 'd, PIO, SM> {
    /// Create a new future waiting for TX-FIFO to become writable.
    pub fn new(sm: &'a mut StateMachineTx<'d, PIO, SM>, value: u32) -> Self {
        FifoOutFuture { sm_tx: sm, value }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Future for FifoOutFuture<'a, 'd, PIO, SM> {
    type Output = ();
    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);
        let value = self.value;
        if self.get_mut().sm_tx.try_push(value) {
            Poll::Ready(())
        } else {
            PIO::wakers().fifo_out()[SM].register(cx.waker());
            PIO::PIO.irqs(0).inte().write_set(|m| {
                m.0 = TXNFULL_MASK << SM;
            });
            // debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Drop for FifoOutFuture<'a, 'd, PIO, SM> {
    fn drop(&mut self) {
        PIO::PIO.irqs(0).inte().write_clear(|m| {
            m.0 = TXNFULL_MASK << SM;
        });
    }
}

/// Future that waits for RX-FIFO to become readable.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoInFuture<'a, 'd, PIO: Instance, const SM: usize> {
    sm_rx: &'a mut StateMachineRx<'d, PIO, SM>,
}

impl<'a, 'd, PIO: Instance, const SM: usize> FifoInFuture<'a, 'd, PIO, SM> {
    /// Create future that waits for RX-FIFO to become readable.
    pub fn new(sm: &'a mut StateMachineRx<'d, PIO, SM>) -> Self {
        FifoInFuture { sm_rx: sm }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Future for FifoInFuture<'a, 'd, PIO, SM> {
    type Output = u32;
    fn poll(mut self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);
        if let Some(v) = self.sm_rx.try_pull() {
            Poll::Ready(v)
        } else {
            PIO::wakers().fifo_in()[SM].register(cx.waker());
            PIO::PIO.irqs(0).inte().write_set(|m| {
                m.0 = RXNEMPTY_MASK << SM;
            });
            //debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Drop for FifoInFuture<'a, 'd, PIO, SM> {
    fn drop(&mut self) {
        PIO::PIO.irqs(0).inte().write_clear(|m| {
            m.0 = RXNEMPTY_MASK << SM;
        });
    }
}

/// Future that waits for IRQ
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct IrqFuture<'a, 'd, PIO: Instance> {
    pio: PhantomData<&'a mut Irq<'d, PIO, 0>>,
    irq_no: u8,
}

impl<'a, 'd, PIO: Instance> Future for IrqFuture<'a, 'd, PIO> {
    type Output = ();
    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);

        // Check if IRQ flag is already set
        if PIO::PIO.irq().read().0 & (1 << self.irq_no) != 0 {
            PIO::PIO.irq().write(|m| m.0 = 1 << self.irq_no);
            return Poll::Ready(());
        }

        PIO::wakers().irq()[self.irq_no as usize].register(cx.waker());
        PIO::PIO.irqs(0).inte().write_set(|m| {
            m.0 = SMIRQ_MASK << self.irq_no;
        });
        Poll::Pending
    }
}

impl<'a, 'd, PIO: Instance> Drop for IrqFuture<'a, 'd, PIO> {
    fn drop(&mut self) {
        PIO::PIO.irqs(0).inte().write_clear(|m| {
            m.0 = SMIRQ_MASK << self.irq_no;
        });
    }
}

/// Type representing a PIO pin.
pub struct Pin<'l, PIO: Instance> {
    pin: Peri<'l, AnyPin>,
    pio: PhantomData<PIO>,
}

impl<'l, PIO: Instance> Pin<'l, PIO> {
    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_drive(match strength {
                Drive::_2mA => pac::pads::vals::Drive::_2M_A,
                Drive::_4mA => pac::pads::vals::Drive::_4M_A,
                Drive::_8mA => pac::pads::vals::Drive::_8M_A,
                Drive::_12mA => pac::pads::vals::Drive::_12M_A,
            });
        });
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_slewfast(slew_rate == SlewRate::Fast);
        });
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_pue(pull == Pull::Up);
            w.set_pde(pull == Pull::Down);
        });
    }

    /// Set the pin's schmitt trigger.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_schmitt(enable);
        });
    }

    /// Set the pin's input sync bypass.
    pub fn set_input_sync_bypass(&mut self, bypass: bool) {
        let mask = 1 << self.pin();
        if bypass {
            PIO::PIO.input_sync_bypass().write_set(|w| *w = mask);
        } else {
            PIO::PIO.input_sync_bypass().write_clear(|w| *w = mask);
        }
    }

    /// Get the underlying pin number.
    pub fn pin(&self) -> u8 {
        self.pin._pin()
    }
}

/// Type representing a state machine RX FIFO.
pub struct StateMachineRx<'d, PIO: Instance, const SM: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const SM: usize> StateMachineRx<'d, PIO, SM> {
    /// Check if RX FIFO is empty.
    pub fn empty(&self) -> bool {
        PIO::PIO.fstat().read().rxempty() & (1u8 << SM) != 0
    }

    /// Check if RX FIFO is full.
    pub fn full(&self) -> bool {
        PIO::PIO.fstat().read().rxfull() & (1u8 << SM) != 0
    }

    /// Check RX FIFO level.
    pub fn level(&self) -> u8 {
        (PIO::PIO.flevel().read().0 >> (SM * 8 + 4)) as u8 & 0x0f
    }

    /// Check if state machine has stalled on full RX FIFO.
    pub fn stalled(&self) -> bool {
        let fdebug = PIO::PIO.fdebug();
        let ret = fdebug.read().rxstall() & (1 << SM) != 0;
        if ret {
            fdebug.write(|w| w.set_rxstall(1 << SM));
        }
        ret
    }

    /// Check if RX FIFO underflow (i.e. read-on-empty by the system) has occurred.
    pub fn underflowed(&self) -> bool {
        let fdebug = PIO::PIO.fdebug();
        let ret = fdebug.read().rxunder() & (1 << SM) != 0;
        if ret {
            fdebug.write(|w| w.set_rxunder(1 << SM));
        }
        ret
    }

    /// Pull data from RX FIFO.
    ///
    /// This function doesn't check if there is data available to be read.
    /// If the rx FIFO is empty, an undefined value is returned. If you only
    /// want to pull if data is available, use `try_pull` instead.
    pub fn pull(&mut self) -> u32 {
        PIO::PIO.rxf(SM).read()
    }

    /// Attempt pulling data from RX FIFO.
    pub fn try_pull(&mut self) -> Option<u32> {
        if self.empty() {
            return None;
        }
        Some(self.pull())
    }

    /// Wait for RX FIFO readable.
    pub fn wait_pull<'a>(&'a mut self) -> FifoInFuture<'a, 'd, PIO, SM> {
        FifoInFuture::new(self)
    }

    /// Prepare DMA transfer from RX FIFO.
    pub fn dma_pull<'a, C: Channel, W: Word>(
        &'a mut self,
        ch: Peri<'a, C>,
        data: &'a mut [W],
        bswap: bool,
    ) -> Transfer<'a, C> {
        let pio_no = PIO::PIO_NO;
        let p = ch.regs();
        p.write_addr().write_value(data.as_ptr() as u32);
        p.read_addr().write_value(PIO::PIO.rxf(SM).as_ptr() as u32);
        #[cfg(feature = "rp2040")]
        p.trans_count().write(|w| *w = data.len() as u32);
        #[cfg(feature = "_rp235x")]
        p.trans_count().write(|w| w.set_count(data.len() as u32));
        compiler_fence(Ordering::SeqCst);
        p.ctrl_trig().write(|w| {
            // Set RX DREQ for this statemachine
            w.set_treq_sel(crate::pac::dma::vals::TreqSel::from(pio_no * 8 + SM as u8 + 4));
            w.set_data_size(W::size());
            w.set_chain_to(ch.number());
            w.set_incr_read(false);
            w.set_incr_write(true);
            w.set_bswap(bswap);
            w.set_en(true);
        });
        compiler_fence(Ordering::SeqCst);
        Transfer::new(ch)
    }
}

/// Type representing a state machine TX FIFO.
pub struct StateMachineTx<'d, PIO: Instance, const SM: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const SM: usize> StateMachineTx<'d, PIO, SM> {
    /// Check if TX FIFO is empty.
    pub fn empty(&self) -> bool {
        PIO::PIO.fstat().read().txempty() & (1u8 << SM) != 0
    }

    /// Check if TX FIFO is full.
    pub fn full(&self) -> bool {
        PIO::PIO.fstat().read().txfull() & (1u8 << SM) != 0
    }

    /// Check TX FIFO level.
    pub fn level(&self) -> u8 {
        (PIO::PIO.flevel().read().0 >> (SM * 8)) as u8 & 0x0f
    }

    /// Check state machine has stalled on empty TX FIFO.
    pub fn stalled(&self) -> bool {
        let fdebug = PIO::PIO.fdebug();
        let ret = fdebug.read().txstall() & (1 << SM) != 0;
        if ret {
            fdebug.write(|w| w.set_txstall(1 << SM));
        }
        ret
    }

    /// Check if FIFO overflowed.
    pub fn overflowed(&self) -> bool {
        let fdebug = PIO::PIO.fdebug();
        let ret = fdebug.read().txover() & (1 << SM) != 0;
        if ret {
            fdebug.write(|w| w.set_txover(1 << SM));
        }
        ret
    }

    /// Force push data to TX FIFO.
    pub fn push(&mut self, v: u32) {
        PIO::PIO.txf(SM).write_value(v);
    }

    /// Attempt to push data to TX FIFO.
    pub fn try_push(&mut self, v: u32) -> bool {
        if self.full() {
            return false;
        }
        self.push(v);
        true
    }

    /// Wait until FIFO is ready for writing.
    pub fn wait_push<'a>(&'a mut self, value: u32) -> FifoOutFuture<'a, 'd, PIO, SM> {
        FifoOutFuture::new(self, value)
    }

    /// Prepare a DMA transfer to TX FIFO.
    pub fn dma_push<'a, C: Channel, W: Word>(
        &'a mut self,
        ch: Peri<'a, C>,
        data: &'a [W],
        bswap: bool,
    ) -> Transfer<'a, C> {
        let pio_no = PIO::PIO_NO;
        let p = ch.regs();
        p.read_addr().write_value(data.as_ptr() as u32);
        p.write_addr().write_value(PIO::PIO.txf(SM).as_ptr() as u32);
        #[cfg(feature = "rp2040")]
        p.trans_count().write(|w| *w = data.len() as u32);
        #[cfg(feature = "_rp235x")]
        p.trans_count().write(|w| w.set_count(data.len() as u32));
        compiler_fence(Ordering::SeqCst);
        p.ctrl_trig().write(|w| {
            // Set TX DREQ for this statemachine
            w.set_treq_sel(crate::pac::dma::vals::TreqSel::from(pio_no * 8 + SM as u8));
            w.set_data_size(W::size());
            w.set_chain_to(ch.number());
            w.set_incr_read(true);
            w.set_incr_write(false);
            w.set_bswap(bswap);
            w.set_en(true);
        });
        compiler_fence(Ordering::SeqCst);
        Transfer::new(ch)
    }
}

/// A type representing a single PIO state machine.
pub struct StateMachine<'d, PIO: Instance, const SM: usize> {
    rx: StateMachineRx<'d, PIO, SM>,
    tx: StateMachineTx<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> Drop for StateMachine<'d, PIO, SM> {
    fn drop(&mut self) {
        PIO::PIO.ctrl().write_clear(|w| w.set_sm_enable(1 << SM));
        on_pio_drop::<PIO>();
    }
}

fn assert_consecutive<PIO: Instance>(pins: &[&Pin<PIO>]) {
    for (p1, p2) in pins.iter().zip(pins.iter().skip(1)) {
        // purposely does not allow wrap-around because we can't claim pins 30 and 31.
        assert!(p1.pin() + 1 == p2.pin(), "pins must be consecutive");
    }
}

/// PIO Execution config.
#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ExecConfig {
    /// If true, the MSB of the Delay/Side-set instruction field is used as side-set enable, rather than a side-set data bit.
    pub side_en: bool,
    /// If true, side-set data is asserted to pin directions, instead of pin values.
    pub side_pindir: bool,
    /// Pin to trigger jump.
    pub jmp_pin: u8,
    /// After reaching this address, execution is wrapped to wrap_bottom.
    pub wrap_top: u8,
    /// After reaching wrap_top, execution is wrapped to this address.
    pub wrap_bottom: u8,
}

/// PIO shift register config for input or output.
#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ShiftConfig {
    /// Number of bits shifted out of OSR before autopull.
    pub threshold: u8,
    /// Shift direction.
    pub direction: ShiftDirection,
    /// For output: Pull automatically output shift register is emptied.
    /// For input: Push automatically when the input shift register is filled.
    pub auto_fill: bool,
}

/// PIO pin config.
#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PinConfig {
    /// The number of MSBs of the Delay/Side-set instruction field which are used for side-set.
    pub sideset_count: u8,
    /// The number of pins asserted by a SET. In the range 0 to 5 inclusive.
    pub set_count: u8,
    /// The number of pins asserted by an OUT PINS, OUT PINDIRS or MOV PINS instruction. In the range 0 to 32 inclusive.
    pub out_count: u8,
    /// The pin which is mapped to the least-significant bit of a state machine's IN data bus.
    pub in_base: u8,
    /// The lowest-numbered pin that will be affected by a side-set operation.
    pub sideset_base: u8,
    /// The lowest-numbered pin that will be affected by a SET PINS or SET PINDIRS instruction.
    pub set_base: u8,
    /// The lowest-numbered pin that will be affected by an OUT PINS, OUT PINDIRS or MOV PINS instruction.
    pub out_base: u8,
}

/// Comparison level or IRQ index for the MOV x, STATUS instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "_rp235x")]
pub enum StatusN {
    /// IRQ flag in this PIO block
    This(u8),
    /// IRQ flag in the next lower PIO block
    Lower(u8),
    /// IRQ flag in the next higher PIO block
    Higher(u8),
}

#[cfg(feature = "_rp235x")]
impl Default for StatusN {
    fn default() -> Self {
        Self::This(0)
    }
}

#[cfg(feature = "_rp235x")]
impl Into<crate::pac::pio::vals::ExecctrlStatusN> for StatusN {
    fn into(self) -> crate::pac::pio::vals::ExecctrlStatusN {
        let x = match self {
            StatusN::This(n) => n,
            StatusN::Lower(n) => n + 0x08,
            StatusN::Higher(n) => n + 0x10,
        };

        crate::pac::pio::vals::ExecctrlStatusN(x)
    }
}

/// PIO config.
#[derive(Clone, Copy, Debug)]
pub struct Config<'d, PIO: Instance> {
    /// Clock divisor register for state machines.
    pub clock_divider: FixedU32<U8>,
    /// Which data bit to use for inline OUT enable.
    pub out_en_sel: u8,
    /// Use a bit of OUT data as an auxiliary write enable When used in conjunction with OUT_STICKY.
    pub inline_out_en: bool,
    /// Continuously assert the most recent OUT/SET to the pins.
    pub out_sticky: bool,
    /// Which source to use for checking status.
    pub status_sel: StatusSource,
    /// Status comparison level.
    #[cfg(feature = "rp2040")]
    pub status_n: u8,
    // This cfg probably shouldn't be required, but the SVD for the 2040 doesn't have the enum
    #[cfg(feature = "_rp235x")]
    /// Status comparison level.
    pub status_n: StatusN,
    exec: ExecConfig,
    origin: Option<u8>,
    /// Configure FIFO allocation.
    pub fifo_join: FifoJoin,
    /// Input shifting config.
    pub shift_in: ShiftConfig,
    /// Output shifting config.
    pub shift_out: ShiftConfig,
    // PINCTRL
    pins: PinConfig,
    in_count: u8,
    _pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> Default for Config<'d, PIO> {
    fn default() -> Self {
        Self {
            clock_divider: 1u8.into(),
            out_en_sel: Default::default(),
            inline_out_en: Default::default(),
            out_sticky: Default::default(),
            status_sel: Default::default(),
            status_n: Default::default(),
            exec: Default::default(),
            origin: Default::default(),
            fifo_join: Default::default(),
            shift_in: Default::default(),
            shift_out: Default::default(),
            pins: Default::default(),
            in_count: Default::default(),
            _pio: Default::default(),
        }
    }
}

impl<'d, PIO: Instance> Config<'d, PIO> {
    /// Get execution configuration.
    pub fn get_exec(&self) -> ExecConfig {
        self.exec
    }

    /// Update execution configuration.
    pub unsafe fn set_exec(&mut self, e: ExecConfig) {
        self.exec = e;
    }

    /// Get pin configuration.
    pub fn get_pins(&self) -> PinConfig {
        self.pins
    }

    /// Update pin configuration.
    pub unsafe fn set_pins(&mut self, p: PinConfig) {
        self.pins = p;
    }

    /// Configures this state machine to use the given program, including jumping to the origin
    /// of the program. The state machine is not started.
    ///
    /// `side_set` sets the range of pins affected by side-sets. The range must be consecutive.
    /// Sideset pins must configured as outputs using [`StateMachine::set_pin_dirs`] to be
    /// effective.
    pub fn use_program(&mut self, prog: &LoadedProgram<'d, PIO>, side_set: &[&Pin<'d, PIO>]) {
        assert!((prog.side_set.bits() - prog.side_set.optional() as u8) as usize == side_set.len());
        assert_consecutive(side_set);
        self.exec.side_en = prog.side_set.optional();
        self.exec.side_pindir = prog.side_set.pindirs();
        self.exec.wrap_bottom = prog.wrap.target;
        self.exec.wrap_top = prog.wrap.source;
        self.pins.sideset_count = prog.side_set.bits();
        self.pins.sideset_base = side_set.first().map_or(0, |p| p.pin());
        self.origin = Some(prog.origin);
    }

    /// Set pin used to signal jump.
    pub fn set_jmp_pin(&mut self, pin: &Pin<'d, PIO>) {
        self.exec.jmp_pin = pin.pin();
    }

    /// Sets the range of pins affected by SET instructions. The range must be consecutive.
    /// Set pins must configured as outputs using [`StateMachine::set_pin_dirs`] to be
    /// effective.
    pub fn set_set_pins(&mut self, pins: &[&Pin<'d, PIO>]) {
        assert!(pins.len() <= 5);
        assert_consecutive(pins);
        self.pins.set_base = pins.first().map_or(0, |p| p.pin());
        self.pins.set_count = pins.len() as u8;
    }

    /// Sets the range of pins affected by OUT instructions. The range must be consecutive.
    /// Out pins must configured as outputs using [`StateMachine::set_pin_dirs`] to be
    /// effective.
    pub fn set_out_pins(&mut self, pins: &[&Pin<'d, PIO>]) {
        assert_consecutive(pins);
        self.pins.out_base = pins.first().map_or(0, |p| p.pin());
        self.pins.out_count = pins.len() as u8;
    }

    /// Sets the range of pins used by IN instructions. The range must be consecutive.
    /// In pins must configured as inputs using [`StateMachine::set_pin_dirs`] to be
    /// effective.
    pub fn set_in_pins(&mut self, pins: &[&Pin<'d, PIO>]) {
        assert_consecutive(pins);
        self.pins.in_base = pins.first().map_or(0, |p| p.pin());
        self.in_count = pins.len() as u8;
    }
}

impl<'d, PIO: Instance + 'd, const SM: usize> StateMachine<'d, PIO, SM> {
    /// Set the config for a given PIO state machine.
    pub fn set_config(&mut self, config: &Config<'d, PIO>) {
        // sm expects 0 for 65536, truncation makes that happen
        assert!(config.clock_divider <= 65536, "clkdiv must be <= 65536");
        assert!(config.clock_divider >= 1, "clkdiv must be >= 1");
        assert!(config.out_en_sel < 32, "out_en_sel must be < 32");
        //assert!(config.status_n < 32, "status_n must be < 32");
        // sm expects 0 for 32, truncation makes that happen
        assert!(config.shift_in.threshold <= 32, "shift_in.threshold must be <= 32");
        assert!(config.shift_out.threshold <= 32, "shift_out.threshold must be <= 32");
        let sm = Self::this_sm();
        sm.clkdiv().write(|w| w.0 = config.clock_divider.to_bits() << 8);
        sm.execctrl().write(|w| {
            w.set_side_en(config.exec.side_en);
            w.set_side_pindir(config.exec.side_pindir);
            w.set_jmp_pin(config.exec.jmp_pin);
            w.set_out_en_sel(config.out_en_sel);
            w.set_inline_out_en(config.inline_out_en);
            w.set_out_sticky(config.out_sticky);
            w.set_wrap_top(config.exec.wrap_top);
            w.set_wrap_bottom(config.exec.wrap_bottom);
            #[cfg(feature = "_rp235x")]
            w.set_status_sel(match config.status_sel {
                StatusSource::TxFifoLevel => pac::pio::vals::ExecctrlStatusSel::TXLEVEL,
                StatusSource::RxFifoLevel => pac::pio::vals::ExecctrlStatusSel::RXLEVEL,
            });
            #[cfg(feature = "rp2040")]
            w.set_status_sel(match config.status_sel {
                StatusSource::TxFifoLevel => pac::pio::vals::SmExecctrlStatusSel::TXLEVEL,
                StatusSource::RxFifoLevel => pac::pio::vals::SmExecctrlStatusSel::RXLEVEL,
            });
            w.set_status_n(config.status_n.into());
        });
        sm.shiftctrl().write(|w| {
            w.set_fjoin_rx(config.fifo_join == FifoJoin::RxOnly);
            w.set_fjoin_tx(config.fifo_join == FifoJoin::TxOnly);
            w.set_pull_thresh(config.shift_out.threshold);
            w.set_push_thresh(config.shift_in.threshold);
            w.set_out_shiftdir(config.shift_out.direction == ShiftDirection::Right);
            w.set_in_shiftdir(config.shift_in.direction == ShiftDirection::Right);
            w.set_autopull(config.shift_out.auto_fill);
            w.set_autopush(config.shift_in.auto_fill);

            #[cfg(feature = "_rp235x")]
            {
                w.set_fjoin_rx_get(
                    config.fifo_join == FifoJoin::RxAsControl || config.fifo_join == FifoJoin::PioScratch,
                );
                w.set_fjoin_rx_put(
                    config.fifo_join == FifoJoin::RxAsStatus || config.fifo_join == FifoJoin::PioScratch,
                );
                w.set_in_count(config.in_count);
            }
        });

        #[cfg(feature = "rp2040")]
        sm.pinctrl().write(|w| {
            w.set_sideset_count(config.pins.sideset_count);
            w.set_set_count(config.pins.set_count);
            w.set_out_count(config.pins.out_count);
            w.set_in_base(config.pins.in_base);
            w.set_sideset_base(config.pins.sideset_base);
            w.set_set_base(config.pins.set_base);
            w.set_out_base(config.pins.out_base);
        });

        #[cfg(feature = "_rp235x")]
        {
            let mut low_ok = true;
            let mut high_ok = true;

            let in_pins = config.pins.in_base..config.pins.in_base + config.in_count;
            let side_pins = config.pins.sideset_base..config.pins.sideset_base + config.pins.sideset_count;
            let set_pins = config.pins.set_base..config.pins.set_base + config.pins.set_count;
            let out_pins = config.pins.out_base..config.pins.out_base + config.pins.out_count;

            for pin_range in [in_pins, side_pins, set_pins, out_pins] {
                for pin in pin_range {
                    low_ok &= pin < 32;
                    high_ok &= pin >= 16;
                }
            }

            if !low_ok && !high_ok {
                panic!(
                    "All pins must either be <32 or >=16, in:{:?}-{:?}, side:{:?}-{:?}, set:{:?}-{:?}, out:{:?}-{:?}",
                    config.pins.in_base,
                    config.pins.in_base + config.in_count - 1,
                    config.pins.sideset_base,
                    config.pins.sideset_base + config.pins.sideset_count - 1,
                    config.pins.set_base,
                    config.pins.set_base + config.pins.set_count - 1,
                    config.pins.out_base,
                    config.pins.out_base + config.pins.out_count - 1,
                )
            }
            let shift = if low_ok { 0 } else { 16 };

            sm.pinctrl().write(|w| {
                w.set_sideset_count(config.pins.sideset_count);
                w.set_set_count(config.pins.set_count);
                w.set_out_count(config.pins.out_count);
                w.set_in_base(config.pins.in_base.checked_sub(shift).unwrap_or_default());
                w.set_sideset_base(config.pins.sideset_base.checked_sub(shift).unwrap_or_default());
                w.set_set_base(config.pins.set_base.checked_sub(shift).unwrap_or_default());
                w.set_out_base(config.pins.out_base.checked_sub(shift).unwrap_or_default());
            });

            PIO::PIO.gpiobase().write(|w| w.set_gpiobase(shift == 16));
        }

        if let Some(origin) = config.origin {
            unsafe { self.exec_jmp(origin) }
        }
    }

    /// Read current instruction address for this state machine
    pub fn get_addr(&self) -> u8 {
        let addr = Self::this_sm().addr();
        addr.read().addr()
    }

    /// Read TX FIFO threshold for this state machine.
    pub fn get_tx_threshold(&self) -> u8 {
        let shiftctrl = Self::this_sm().shiftctrl();
        shiftctrl.read().pull_thresh()
    }

    /// Set/change the TX FIFO threshold for this state machine.
    pub fn set_tx_threshold(&mut self, threshold: u8) {
        assert!(threshold <= 31);
        let shiftctrl = Self::this_sm().shiftctrl();
        shiftctrl.modify(|w| {
            w.set_pull_thresh(threshold);
        });
    }

    /// Read TX FIFO threshold for this state machine.
    pub fn get_rx_threshold(&self) -> u8 {
        Self::this_sm().shiftctrl().read().push_thresh()
    }

    /// Set/change the RX FIFO threshold for this state machine.
    pub fn set_rx_threshold(&mut self, threshold: u8) {
        assert!(threshold <= 31);
        let shiftctrl = Self::this_sm().shiftctrl();
        shiftctrl.modify(|w| {
            w.set_push_thresh(threshold);
        });
    }

    /// Set/change both TX and RX FIFO thresholds for this state machine.
    pub fn set_thresholds(&mut self, threshold: u8) {
        assert!(threshold <= 31);
        let shiftctrl = Self::this_sm().shiftctrl();
        shiftctrl.modify(|w| {
            w.set_push_thresh(threshold);
            w.set_pull_thresh(threshold);
        });
    }

    /// Set the clock divider for this state machine.
    pub fn set_clock_divider(&mut self, clock_divider: FixedU32<U8>) {
        let sm = Self::this_sm();
        sm.clkdiv().write(|w| w.0 = clock_divider.to_bits() << 8);
    }

    #[inline(always)]
    fn this_sm() -> crate::pac::pio::StateMachine {
        PIO::PIO.sm(SM)
    }

    /// Restart this state machine.
    pub fn restart(&mut self) {
        let mask = 1u8 << SM;
        PIO::PIO.ctrl().write_set(|w| w.set_sm_restart(mask));
    }

    /// Enable state machine.
    pub fn set_enable(&mut self, enable: bool) {
        let mask = 1u8 << SM;
        if enable {
            PIO::PIO.ctrl().write_set(|w| w.set_sm_enable(mask));
        } else {
            PIO::PIO.ctrl().write_clear(|w| w.set_sm_enable(mask));
        }
    }

    /// Check if state machine is enabled.
    pub fn is_enabled(&self) -> bool {
        PIO::PIO.ctrl().read().sm_enable() & (1u8 << SM) != 0
    }

    /// Restart a state machine's clock divider from an initial phase of 0.
    pub fn clkdiv_restart(&mut self) {
        let mask = 1u8 << SM;
        PIO::PIO.ctrl().write_set(|w| w.set_clkdiv_restart(mask));
    }

    fn with_paused(&mut self, f: impl FnOnce(&mut Self)) {
        let enabled = self.is_enabled();
        self.set_enable(false);
        let pincfg = Self::this_sm().pinctrl().read();
        let execcfg = Self::this_sm().execctrl().read();
        Self::this_sm().execctrl().write_clear(|w| w.set_out_sticky(true));
        f(self);
        Self::this_sm().pinctrl().write_value(pincfg);
        Self::this_sm().execctrl().write_value(execcfg);
        self.set_enable(enabled);
    }

    /// Sets pin directions. This pauses the current state machine to run `SET` commands
    /// and temporarily unsets the `OUT_STICKY` bit.
    pub fn set_pin_dirs(&mut self, dir: Direction, pins: &[&Pin<'d, PIO>]) {
        self.with_paused(|sm| {
            for pin in pins {
                Self::this_sm().pinctrl().write(|w| {
                    w.set_set_base(pin.pin());
                    w.set_set_count(1);
                });
                // SET PINDIRS, (dir)
                unsafe { sm.exec_instr(0b111_00000_100_00000 | dir as u16) };
            }
        });
    }

    /// Sets pin output values. This pauses the current state machine to run
    /// `SET` commands and temporarily unsets the `OUT_STICKY` bit.
    pub fn set_pins(&mut self, level: Level, pins: &[&Pin<'d, PIO>]) {
        self.with_paused(|sm| {
            for pin in pins {
                Self::this_sm().pinctrl().write(|w| {
                    w.set_set_base(pin.pin());
                    w.set_set_count(1);
                });
                // SET PINS, (dir)
                unsafe { sm.exec_instr(0b11100_000_000_00000 | level as u16) };
            }
        });
    }

    /// Flush FIFOs for state machine.
    pub fn clear_fifos(&mut self) {
        // Toggle FJOIN_RX to flush FIFOs
        let shiftctrl = Self::this_sm().shiftctrl();
        shiftctrl.modify(|w| {
            w.set_fjoin_rx(!w.fjoin_rx());
        });
        shiftctrl.modify(|w| {
            w.set_fjoin_rx(!w.fjoin_rx());
        });
    }

    /// Instruct state machine to execute a given instructions
    ///
    /// SAFETY: The state machine must be in a state where executing
    /// an arbitrary instruction does not crash it.
    pub unsafe fn exec_instr(&mut self, instr: u16) {
        Self::this_sm().instr().write(|w| w.set_instr(instr));
    }

    /// Return a read handle for reading state machine outputs.
    pub fn rx(&mut self) -> &mut StateMachineRx<'d, PIO, SM> {
        &mut self.rx
    }

    /// Return a handle for writing to inputs.
    pub fn tx(&mut self) -> &mut StateMachineTx<'d, PIO, SM> {
        &mut self.tx
    }

    /// Return both read and write handles for the state machine.
    pub fn rx_tx(&mut self) -> (&mut StateMachineRx<'d, PIO, SM>, &mut StateMachineTx<'d, PIO, SM>) {
        (&mut self.rx, &mut self.tx)
    }

    /// Return the contents of the nth entry of the RX FIFO
    /// (should be used only when the FIFO config is set to [`FifoJoin::RxAsStatus`])
    #[cfg(feature = "_rp235x")]
    pub fn get_rxf_entry(&self, n: usize) -> u32 {
        PIO::PIO.rxf_putget(SM).putget(n).read()
    }

    /// Set the contents of the nth entry of the RX FIFO
    /// (should be used only when the FIFO config is set to [`FifoJoin::RxAsControl`])
    #[cfg(feature = "_rp235x")]
    pub fn set_rxf_entry(&self, n: usize, val: u32) {
        PIO::PIO.rxf_putget(SM).putget(n).write_value(val)
    }
}

/// PIO handle.
pub struct Common<'d, PIO: Instance> {
    instructions_used: u32,
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> Drop for Common<'d, PIO> {
    fn drop(&mut self) {
        on_pio_drop::<PIO>();
    }
}

/// Memory of PIO instance.
pub struct InstanceMemory<'d, PIO: Instance> {
    used_mask: u32,
    pio: PhantomData<&'d mut PIO>,
}

/// A loaded PIO program.
pub struct LoadedProgram<'d, PIO: Instance> {
    /// Memory used by program.
    pub used_memory: InstanceMemory<'d, PIO>,
    /// Program origin for loading.
    pub origin: u8,
    /// Wrap controls what to do once program is done executing.
    pub wrap: Wrap,
    /// Data for 'side' set instruction parameters.
    pub side_set: SideSet,
}

/// Errors loading a PIO program.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoadError {
    /// Insufficient consecutive free instruction space to load program.
    InsufficientSpace,
    /// Loading the program would overwrite an instruction address already
    /// used by another program.
    AddressInUse(usize),
}

impl<'d, PIO: Instance> Common<'d, PIO> {
    /// Load a PIO program. This will automatically relocate the program to
    /// an available chunk of free instruction memory if the program origin
    /// was not explicitly specified, otherwise it will attempt to load the
    /// program only at its origin.
    pub fn load_program<const SIZE: usize>(&mut self, prog: &Program<SIZE>) -> LoadedProgram<'d, PIO> {
        match self.try_load_program(prog) {
            Ok(r) => r,
            Err(e) => panic!("Failed to load PIO program: {:?}", e),
        }
    }

    /// Load a PIO program. This will automatically relocate the program to
    /// an available chunk of free instruction memory if the program origin
    /// was not explicitly specified, otherwise it will attempt to load the
    /// program only at its origin.
    pub fn try_load_program<const SIZE: usize>(
        &mut self,
        prog: &Program<SIZE>,
    ) -> Result<LoadedProgram<'d, PIO>, LoadError> {
        match prog.origin {
            Some(origin) => self.try_load_program_at(prog, origin).map_err(LoadError::AddressInUse),
            None => {
                // naively search for free space, allowing wraparound since
                // PIO does support that. with only 32 instruction slots it
                // doesn't make much sense to do anything more fancy.
                let mut origin = 0;
                while origin < 32 {
                    match self.try_load_program_at(prog, origin as _) {
                        Ok(r) => return Ok(r),
                        Err(a) => origin = a + 1,
                    }
                }
                Err(LoadError::InsufficientSpace)
            }
        }
    }

    fn try_load_program_at<const SIZE: usize>(
        &mut self,
        prog: &Program<SIZE>,
        origin: u8,
    ) -> Result<LoadedProgram<'d, PIO>, usize> {
        #[cfg(not(feature = "_rp235x"))]
        assert!(prog.version == pio::PioVersion::V0);

        let prog = RelocatedProgram::new_with_origin(prog, origin);
        let used_memory = self.try_write_instr(prog.origin() as _, prog.code())?;
        Ok(LoadedProgram {
            used_memory,
            origin: prog.origin(),
            wrap: prog.wrap(),
            side_set: prog.side_set(),
        })
    }

    fn try_write_instr<I>(&mut self, start: usize, instrs: I) -> Result<InstanceMemory<'d, PIO>, usize>
    where
        I: Iterator<Item = u16>,
    {
        let mut used_mask = 0;
        for (i, instr) in instrs.enumerate() {
            // wrapping around the end of program memory is valid, let's make use of that.
            let addr = (i + start) % 32;
            let mask = 1 << addr;
            if (self.instructions_used | used_mask) & mask != 0 {
                return Err(addr);
            }
            PIO::PIO.instr_mem(addr).write(|w| {
                w.set_instr_mem(instr);
            });
            used_mask |= mask;
        }
        self.instructions_used |= used_mask;
        Ok(InstanceMemory {
            used_mask,
            pio: PhantomData,
        })
    }

    /// Free instruction memory. This is always possible but unsafe if any
    /// state machine is still using this bit of memory.
    pub unsafe fn free_instr(&mut self, instrs: InstanceMemory<PIO>) {
        self.instructions_used &= !instrs.used_mask;
    }

    /// Bypass flipflop synchronizer on GPIO inputs.
    pub fn set_input_sync_bypass<'a>(&'a mut self, bypass: u32, mask: u32) {
        // this can interfere with per-pin bypass functions. splitting the
        // modification is going to be fine since nothing that relies on
        // it can reasonably run before we finish.
        PIO::PIO.input_sync_bypass().write_set(|w| *w = mask & bypass);
        PIO::PIO.input_sync_bypass().write_clear(|w| *w = mask & !bypass);
    }

    /// Get bypass configuration.
    pub fn get_input_sync_bypass(&self) -> u32 {
        PIO::PIO.input_sync_bypass().read()
    }

    /// Register a pin for PIO usage. Pins will be released from the PIO block
    /// (i.e., have their `FUNCSEL` reset to `NULL`) when the [`Common`] *and*
    /// all [`StateMachine`]s for this block have been dropped. **Other members
    /// of [`Pio`] do not keep pin registrations alive.**
    pub fn make_pio_pin(&mut self, pin: Peri<'d, impl PioPin + 'd>) -> Pin<'d, PIO> {
        // enable the outputs
        pin.pad_ctrl().write(|w| w.set_od(false));
        // especially important on the 235x, where IE defaults to 0
        pin.pad_ctrl().write(|w| w.set_ie(true));

        pin.gpio().ctrl().write(|w| w.set_funcsel(PIO::FUNCSEL as _));
        pin.pad_ctrl().write(|w| {
            #[cfg(feature = "_rp235x")]
            w.set_iso(false);
            w.set_schmitt(true);
            w.set_slewfast(false);
            // TODO rp235x errata E9 recommends to not enable IE if we're not
            // going to use input. Maybe add an API for the user to enable/disable this?
            w.set_ie(true);
            w.set_od(false);
            w.set_pue(false);
            w.set_pde(false);
        });
        // we can be relaxed about this because we're &mut here and nothing is cached
        PIO::state().used_pins.fetch_or(1 << pin.pin_bank(), Ordering::Relaxed);
        Pin {
            pin: pin.into(),
            pio: PhantomData::default(),
        }
    }

    /// Apply changes to all state machines in a batch.
    pub fn apply_sm_batch(&mut self, f: impl FnOnce(&mut PioBatch<'d, PIO>)) {
        let mut batch = PioBatch {
            clkdiv_restart: 0,
            sm_restart: 0,
            sm_enable_mask: 0,
            sm_enable: 0,
            _pio: PhantomData,
        };
        f(&mut batch);
        PIO::PIO.ctrl().modify(|w| {
            w.set_clkdiv_restart(batch.clkdiv_restart);
            w.set_sm_restart(batch.sm_restart);
            w.set_sm_enable((w.sm_enable() & !batch.sm_enable_mask) | batch.sm_enable);
        });
    }
}

/// Represents multiple state machines in a single type.
pub struct PioBatch<'a, PIO: Instance> {
    clkdiv_restart: u8,
    sm_restart: u8,
    sm_enable_mask: u8,
    sm_enable: u8,
    _pio: PhantomData<&'a PIO>,
}

impl<'a, PIO: Instance> PioBatch<'a, PIO> {
    /// Restart a state machine's clock divider from an initial phase of 0.
    pub fn restart<const SM: usize>(&mut self, _sm: &mut StateMachine<'a, PIO, SM>) {
        self.clkdiv_restart |= 1 << SM;
    }

    /// Enable a specific state machine.
    pub fn set_enable<const SM: usize>(&mut self, _sm: &mut StateMachine<'a, PIO, SM>, enable: bool) {
        self.sm_enable_mask |= 1 << SM;
        self.sm_enable |= (enable as u8) << SM;
    }
}

/// Type representing a PIO interrupt.
pub struct Irq<'d, PIO: Instance, const N: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const N: usize> Irq<'d, PIO, N> {
    /// Wait for an IRQ to fire.
    pub fn wait<'a>(&'a mut self) -> IrqFuture<'a, 'd, PIO> {
        IrqFuture {
            pio: PhantomData,
            irq_no: N as u8,
        }
    }
}

/// Interrupt flags for a PIO instance.
#[derive(Clone)]
pub struct IrqFlags<'d, PIO: Instance> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> IrqFlags<'d, PIO> {
    /// Check if interrupt fired.
    pub fn check(&self, irq_no: u8) -> bool {
        assert!(irq_no < 8);
        self.check_any(1 << irq_no)
    }

    /// Check if any of the interrupts in the bitmap fired.
    pub fn check_any(&self, irqs: u8) -> bool {
        PIO::PIO.irq().read().irq() & irqs != 0
    }

    /// Check if all interrupts have fired.
    pub fn check_all(&self, irqs: u8) -> bool {
        PIO::PIO.irq().read().irq() & irqs == irqs
    }

    /// Clear interrupt for interrupt number.
    pub fn clear(&self, irq_no: usize) {
        assert!(irq_no < 8);
        self.clear_all(1 << irq_no);
    }

    /// Clear all interrupts set in the bitmap.
    pub fn clear_all(&self, irqs: u8) {
        PIO::PIO.irq().write(|w| w.set_irq(irqs))
    }

    /// Fire a given interrupt.
    pub fn set(&self, irq_no: usize) {
        assert!(irq_no < 8);
        self.set_all(1 << irq_no);
    }

    /// Fire all interrupts.
    pub fn set_all(&self, irqs: u8) {
        PIO::PIO.irq_force().write(|w| w.set_irq_force(irqs))
    }
}

/// An instance of the PIO driver.
pub struct Pio<'d, PIO: Instance> {
    /// PIO handle.
    pub common: Common<'d, PIO>,
    /// PIO IRQ flags.
    pub irq_flags: IrqFlags<'d, PIO>,
    /// IRQ0 configuration.
    pub irq0: Irq<'d, PIO, 0>,
    /// IRQ1 configuration.
    pub irq1: Irq<'d, PIO, 1>,
    /// IRQ2 configuration.
    pub irq2: Irq<'d, PIO, 2>,
    /// IRQ3 configuration.
    pub irq3: Irq<'d, PIO, 3>,
    /// State machine 0 handle.
    pub sm0: StateMachine<'d, PIO, 0>,
    /// State machine 1 handle.
    pub sm1: StateMachine<'d, PIO, 1>,
    /// State machine 2 handle.
    pub sm2: StateMachine<'d, PIO, 2>,
    /// State machine 3 handle.
    pub sm3: StateMachine<'d, PIO, 3>,
    _pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> Pio<'d, PIO> {
    /// Create a new instance of a PIO peripheral.
    pub fn new(_pio: Peri<'d, PIO>, _irq: impl Binding<PIO::Interrupt, InterruptHandler<PIO>>) -> Self {
        PIO::state().users.store(5, Ordering::Release);
        PIO::state().used_pins.store(0, Ordering::Release);
        PIO::Interrupt::unpend();
        unsafe { PIO::Interrupt::enable() };
        Self {
            common: Common {
                instructions_used: 0,
                pio: PhantomData,
            },
            irq_flags: IrqFlags { pio: PhantomData },
            irq0: Irq { pio: PhantomData },
            irq1: Irq { pio: PhantomData },
            irq2: Irq { pio: PhantomData },
            irq3: Irq { pio: PhantomData },
            sm0: StateMachine {
                rx: StateMachineRx { pio: PhantomData },
                tx: StateMachineTx { pio: PhantomData },
            },
            sm1: StateMachine {
                rx: StateMachineRx { pio: PhantomData },
                tx: StateMachineTx { pio: PhantomData },
            },
            sm2: StateMachine {
                rx: StateMachineRx { pio: PhantomData },
                tx: StateMachineTx { pio: PhantomData },
            },
            sm3: StateMachine {
                rx: StateMachineRx { pio: PhantomData },
                tx: StateMachineTx { pio: PhantomData },
            },
            _pio: PhantomData,
        }
    }
}

/// Representation of the PIO state keeping a record of which pins are assigned to
/// each PIO.
// make_pio_pin notionally takes ownership of the pin it is given, but the wrapped pin
// cannot be treated as an owned resource since dropping it would have to deconfigure
// the pin, breaking running state machines in the process. pins are also shared
// between all state machines, which makes ownership even messier to track any
// other way.
pub struct State {
    users: AtomicU8,
    used_pins: AtomicU64,
}

fn on_pio_drop<PIO: Instance>() {
    let state = PIO::state();
    if state.users.fetch_sub(1, Ordering::AcqRel) == 1 {
        let used_pins = state.used_pins.load(Ordering::Relaxed);
        let null = pac::io::vals::Gpio0ctrlFuncsel::NULL as _;
        for i in 0..crate::gpio::BANK0_PIN_COUNT {
            if used_pins & (1 << i) != 0 {
                pac::IO_BANK0.gpio(i).ctrl().write(|w| w.set_funcsel(null));
            }
        }
    }
}

trait SealedInstance {
    const PIO_NO: u8;
    const PIO: &'static crate::pac::pio::Pio;
    const FUNCSEL: crate::pac::io::vals::Gpio0ctrlFuncsel;

    #[inline]
    fn wakers() -> &'static Wakers {
        static WAKERS: Wakers = Wakers([const { AtomicWaker::new() }; 12]);
        &WAKERS
    }

    #[inline]
    fn state() -> &'static State {
        static STATE: State = State {
            users: AtomicU8::new(0),
            used_pins: AtomicU64::new(0),
        };

        &STATE
    }
}

/// PIO instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Sized + Unpin {
    /// Interrupt for this peripheral.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

macro_rules! impl_pio {
    ($name:ident, $pio:expr, $pac:ident, $funcsel:ident, $irq:ident) => {
        impl SealedInstance for peripherals::$name {
            const PIO_NO: u8 = $pio;
            const PIO: &'static pac::pio::Pio = &pac::$pac;
            const FUNCSEL: pac::io::vals::Gpio0ctrlFuncsel = pac::io::vals::Gpio0ctrlFuncsel::$funcsel;
        }
        impl Instance for peripherals::$name {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

impl_pio!(PIO0, 0, PIO0, PIO0_0, PIO0_IRQ_0);
impl_pio!(PIO1, 1, PIO1, PIO1_0, PIO1_IRQ_0);
#[cfg(feature = "_rp235x")]
impl_pio!(PIO2, 2, PIO2, PIO2_0, PIO2_IRQ_0);

/// PIO pin.
pub trait PioPin: gpio::Pin {}

macro_rules! impl_pio_pin {
    ($( $pin:ident, )*) => {
        $(
            impl PioPin for peripherals::$pin {}
        )*
    };
}

impl_pio_pin! {
    PIN_0,
    PIN_1,
    PIN_2,
    PIN_3,
    PIN_4,
    PIN_5,
    PIN_6,
    PIN_7,
    PIN_8,
    PIN_9,
    PIN_10,
    PIN_11,
    PIN_12,
    PIN_13,
    PIN_14,
    PIN_15,
    PIN_16,
    PIN_17,
    PIN_18,
    PIN_19,
    PIN_20,
    PIN_21,
    PIN_22,
    PIN_23,
    PIN_24,
    PIN_25,
    PIN_26,
    PIN_27,
    PIN_28,
    PIN_29,
}

#[cfg(feature = "rp235xb")]
impl_pio_pin! {
    PIN_30,
    PIN_31,
    PIN_32,
    PIN_33,
    PIN_34,
    PIN_35,
    PIN_36,
    PIN_37,
    PIN_38,
    PIN_39,
    PIN_40,
    PIN_41,
    PIN_42,
    PIN_43,
    PIN_44,
    PIN_45,
    PIN_46,
    PIN_47,
}
