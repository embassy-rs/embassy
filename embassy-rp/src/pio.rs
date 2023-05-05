use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use atomic_polyfill::{AtomicU32, AtomicU8};
use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::io::vals::Gpio0ctrlFuncsel;
use pio::{SideSet, Wrap};

use crate::dma::{Channel, Transfer, Word};
use crate::gpio::sealed::Pin as SealedPin;
use crate::gpio::{self, AnyPin, Drive, Pull, SlewRate};
use crate::pac::dma::vals::TreqSel;
use crate::relocate::RelocatedProgram;
use crate::{interrupt, pac, peripherals, pio_instr_util, RegExt};

struct Wakers([AtomicWaker; 12]);

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

const NEW_AW: AtomicWaker = AtomicWaker::new();
const PIO_WAKERS_INIT: Wakers = Wakers([NEW_AW; 12]);
static WAKERS: [Wakers; 2] = [PIO_WAKERS_INIT; 2];

pub enum FifoJoin {
    /// Both TX and RX fifo is enabled
    Duplex,
    /// Rx fifo twice as deep. TX fifo disabled
    RxOnly,
    /// Tx fifo twice as deep. RX fifo disabled
    TxOnly,
}

#[derive(PartialEq)]
pub enum ShiftDirection {
    Right = 1,
    Left = 0,
}

const RXNEMPTY_MASK: u32 = 1 << 0;
const TXNFULL_MASK: u32 = 1 << 4;
const SMIRQ_MASK: u32 = 1 << 8;

#[interrupt]
unsafe fn PIO0_IRQ_0() {
    use crate::pac;
    let ints = pac::PIO0.irqs(0).ints().read().0;
    for bit in 0..12 {
        if ints & (1 << bit) != 0 {
            WAKERS[0].0[bit].wake();
        }
    }
    pac::PIO0.irqs(0).inte().write_clear(|m| m.0 = ints);
}

#[interrupt]
unsafe fn PIO1_IRQ_0() {
    use crate::pac;
    let ints = pac::PIO1.irqs(0).ints().read().0;
    for bit in 0..12 {
        if ints & (1 << bit) != 0 {
            WAKERS[1].0[bit].wake();
        }
    }
    pac::PIO1.irqs(0).inte().write_clear(|m| m.0 = ints);
}

pub(crate) unsafe fn init() {
    let irq = interrupt::PIO0_IRQ_0::steal();
    irq.disable();
    irq.set_priority(interrupt::Priority::P3);
    pac::PIO0.irqs(0).inte().write(|m| m.0 = 0);
    irq.enable();

    let irq = interrupt::PIO1_IRQ_0::steal();
    irq.disable();
    irq.set_priority(interrupt::Priority::P3);
    pac::PIO1.irqs(0).inte().write(|m| m.0 = 0);
    irq.enable();
}

/// Future that waits for TX-FIFO to become writable
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoOutFuture<'a, 'd, PIO: Instance, const SM: usize> {
    sm_tx: &'a mut StateMachineTx<'d, PIO, SM>,
    value: u32,
}

impl<'a, 'd, PIO: Instance, const SM: usize> FifoOutFuture<'a, 'd, PIO, SM> {
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
            WAKERS[PIO::PIO_NO as usize].fifo_out()[SM].register(cx.waker());
            unsafe {
                PIO::PIO.irqs(0).inte().write_set(|m| {
                    m.0 = TXNFULL_MASK << SM;
                });
            }
            // debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Drop for FifoOutFuture<'a, 'd, PIO, SM> {
    fn drop(&mut self) {
        unsafe {
            PIO::PIO.irqs(0).inte().write_clear(|m| {
                m.0 = TXNFULL_MASK << SM;
            });
        }
    }
}

/// Future that waits for RX-FIFO to become readable
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoInFuture<'a, 'd, PIO: Instance, const SM: usize> {
    sm_rx: &'a mut StateMachineRx<'d, PIO, SM>,
}

impl<'a, 'd, PIO: Instance, const SM: usize> FifoInFuture<'a, 'd, PIO, SM> {
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
            WAKERS[PIO::PIO_NO as usize].fifo_in()[SM].register(cx.waker());
            unsafe {
                PIO::PIO.irqs(0).inte().write_set(|m| {
                    m.0 = RXNEMPTY_MASK << SM;
                });
            }
            //debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'a, 'd, PIO: Instance, const SM: usize> Drop for FifoInFuture<'a, 'd, PIO, SM> {
    fn drop(&mut self) {
        unsafe {
            PIO::PIO.irqs(0).inte().write_clear(|m| {
                m.0 = RXNEMPTY_MASK << SM;
            });
        }
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
        if unsafe { PIO::PIO.irq().read().0 & (1 << self.irq_no) != 0 } {
            unsafe {
                PIO::PIO.irq().write(|m| m.0 = 1 << self.irq_no);
            }
            return Poll::Ready(());
        }

        WAKERS[PIO::PIO_NO as usize].irq()[self.irq_no as usize].register(cx.waker());
        unsafe {
            PIO::PIO.irqs(0).inte().write_set(|m| {
                m.0 = SMIRQ_MASK << self.irq_no;
            });
        }
        Poll::Pending
    }
}

impl<'a, 'd, PIO: Instance> Drop for IrqFuture<'a, 'd, PIO> {
    fn drop(&mut self) {
        unsafe {
            PIO::PIO.irqs(0).inte().write_clear(|m| {
                m.0 = SMIRQ_MASK << self.irq_no;
            });
        }
    }
}

pub struct Pin<'l, PIO: Instance> {
    pin: PeripheralRef<'l, AnyPin>,
    pio: PhantomData<PIO>,
}

impl<'l, PIO: Instance> Pin<'l, PIO> {
    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        unsafe {
            self.pin.pad_ctrl().modify(|w| {
                w.set_drive(match strength {
                    Drive::_2mA => pac::pads::vals::Drive::_2MA,
                    Drive::_4mA => pac::pads::vals::Drive::_4MA,
                    Drive::_8mA => pac::pads::vals::Drive::_8MA,
                    Drive::_12mA => pac::pads::vals::Drive::_12MA,
                });
            });
        }
    }

    // Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        unsafe {
            self.pin.pad_ctrl().modify(|w| {
                w.set_slewfast(slew_rate == SlewRate::Fast);
            });
        }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        unsafe {
            self.pin.pad_ctrl().modify(|w| {
                w.set_pue(pull == Pull::Up);
                w.set_pde(pull == Pull::Down);
            });
        }
    }

    /// Set the pin's schmitt trigger.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        unsafe {
            self.pin.pad_ctrl().modify(|w| {
                w.set_schmitt(enable);
            });
        }
    }

    pub fn set_input_sync_bypass<'a>(&mut self, bypass: bool) {
        let mask = 1 << self.pin();
        unsafe {
            if bypass {
                PIO::PIO.input_sync_bypass().write_set(|w| *w = mask);
            } else {
                PIO::PIO.input_sync_bypass().write_clear(|w| *w = mask);
            }
        }
    }

    pub fn pin(&self) -> u8 {
        self.pin._pin()
    }
}

pub struct StateMachineRx<'d, PIO: Instance, const SM: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const SM: usize> StateMachineRx<'d, PIO, SM> {
    pub fn empty(&self) -> bool {
        unsafe { PIO::PIO.fstat().read().rxempty() & (1u8 << SM) != 0 }
    }

    pub fn full(&self) -> bool {
        unsafe { PIO::PIO.fstat().read().rxfull() & (1u8 << SM) != 0 }
    }

    pub fn level(&self) -> u8 {
        unsafe { (PIO::PIO.flevel().read().0 >> (SM * 8 + 4)) as u8 & 0x0f }
    }

    pub fn stalled(&self) -> bool {
        unsafe {
            let fdebug = PIO::PIO.fdebug();
            let ret = fdebug.read().rxstall() & (1 << SM) != 0;
            if ret {
                fdebug.write(|w| w.set_rxstall(1 << SM));
            }
            ret
        }
    }

    pub fn underflowed(&self) -> bool {
        unsafe {
            let fdebug = PIO::PIO.fdebug();
            let ret = fdebug.read().rxunder() & (1 << SM) != 0;
            if ret {
                fdebug.write(|w| w.set_rxunder(1 << SM));
            }
            ret
        }
    }

    pub fn pull(&mut self) -> u32 {
        unsafe { PIO::PIO.rxf(SM).read() }
    }

    pub fn try_pull(&mut self) -> Option<u32> {
        if self.empty() {
            return None;
        }
        Some(self.pull())
    }

    pub fn wait_pull<'a>(&'a mut self) -> FifoInFuture<'a, 'd, PIO, SM> {
        FifoInFuture::new(self)
    }

    pub fn dma_pull<'a, C: Channel, W: Word>(
        &'a mut self,
        ch: PeripheralRef<'a, C>,
        data: &'a mut [W],
    ) -> Transfer<'a, C> {
        unsafe {
            let pio_no = PIO::PIO_NO;
            let p = ch.regs();
            p.write_addr().write_value(data.as_ptr() as u32);
            p.read_addr().write_value(PIO::PIO.rxf(SM).ptr() as u32);
            p.trans_count().write_value(data.len() as u32);
            compiler_fence(Ordering::SeqCst);
            p.ctrl_trig().write(|w| {
                // Set RX DREQ for this statemachine
                w.set_treq_sel(TreqSel(pio_no * 8 + SM as u8 + 4));
                w.set_data_size(W::size());
                w.set_chain_to(ch.number());
                w.set_incr_read(false);
                w.set_incr_write(true);
                w.set_en(true);
            });
            compiler_fence(Ordering::SeqCst);
        }
        Transfer::new(ch)
    }
}

pub struct StateMachineTx<'d, PIO: Instance, const SM: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const SM: usize> StateMachineTx<'d, PIO, SM> {
    pub fn empty(&self) -> bool {
        unsafe { PIO::PIO.fstat().read().txempty() & (1u8 << SM) != 0 }
    }
    pub fn full(&self) -> bool {
        unsafe { PIO::PIO.fstat().read().txfull() & (1u8 << SM) != 0 }
    }

    pub fn level(&self) -> u8 {
        unsafe { (PIO::PIO.flevel().read().0 >> (SM * 8)) as u8 & 0x0f }
    }

    pub fn stalled(&self) -> bool {
        unsafe {
            let fdebug = PIO::PIO.fdebug();
            let ret = fdebug.read().txstall() & (1 << SM) != 0;
            if ret {
                fdebug.write(|w| w.set_txstall(1 << SM));
            }
            ret
        }
    }

    pub fn overflowed(&self) -> bool {
        unsafe {
            let fdebug = PIO::PIO.fdebug();
            let ret = fdebug.read().txover() & (1 << SM) != 0;
            if ret {
                fdebug.write(|w| w.set_txover(1 << SM));
            }
            ret
        }
    }

    pub fn push(&mut self, v: u32) {
        unsafe {
            PIO::PIO.txf(SM).write_value(v);
        }
    }

    pub fn try_push(&mut self, v: u32) -> bool {
        if self.full() {
            return false;
        }
        self.push(v);
        true
    }

    pub fn wait_push<'a>(&'a mut self, value: u32) -> FifoOutFuture<'a, 'd, PIO, SM> {
        FifoOutFuture::new(self, value)
    }

    pub fn dma_push<'a, C: Channel, W: Word>(&'a mut self, ch: PeripheralRef<'a, C>, data: &'a [W]) -> Transfer<'a, C> {
        unsafe {
            let pio_no = PIO::PIO_NO;
            let p = ch.regs();
            p.read_addr().write_value(data.as_ptr() as u32);
            p.write_addr().write_value(PIO::PIO.txf(SM).ptr() as u32);
            p.trans_count().write_value(data.len() as u32);
            compiler_fence(Ordering::SeqCst);
            p.ctrl_trig().write(|w| {
                // Set TX DREQ for this statemachine
                w.set_treq_sel(TreqSel(pio_no * 8 + SM as u8));
                w.set_data_size(W::size());
                w.set_chain_to(ch.number());
                w.set_incr_read(true);
                w.set_incr_write(false);
                w.set_en(true);
            });
            compiler_fence(Ordering::SeqCst);
        }
        Transfer::new(ch)
    }
}

pub struct StateMachine<'d, PIO: Instance, const SM: usize> {
    rx: StateMachineRx<'d, PIO, SM>,
    tx: StateMachineTx<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> Drop for StateMachine<'d, PIO, SM> {
    fn drop(&mut self) {
        unsafe {
            PIO::PIO.ctrl().write_clear(|w| w.set_sm_enable(1 << SM));
        }
        on_pio_drop::<PIO>();
    }
}

fn assert_consecutive<'d, PIO: Instance>(pins: &[&Pin<'d, PIO>]) {
    for (p1, p2) in pins.iter().zip(pins.iter().skip(1)) {
        // purposely does not allow wrap-around because we can't claim pins 30 and 31.
        assert!(p1.pin() + 1 == p2.pin(), "pins must be consecutive");
    }
}

impl<'d, PIO: Instance + 'd, const SM: usize> StateMachine<'d, PIO, SM> {
    #[inline(always)]
    fn this_sm() -> crate::pac::pio::StateMachine {
        PIO::PIO.sm(SM)
    }

    pub fn restart(&mut self) {
        let mask = 1u8 << SM;
        unsafe {
            PIO::PIO.ctrl().write_set(|w| w.set_sm_restart(mask));
        }
    }
    pub fn set_enable(&mut self, enable: bool) {
        let mask = 1u8 << SM;
        unsafe {
            if enable {
                PIO::PIO.ctrl().write_set(|w| w.set_sm_enable(mask));
            } else {
                PIO::PIO.ctrl().write_clear(|w| w.set_sm_enable(mask));
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { PIO::PIO.ctrl().read().sm_enable() & (1u8 << SM) != 0 }
    }

    pub fn set_clkdiv(&mut self, div_x_256: u32) {
        unsafe {
            Self::this_sm().clkdiv().write(|w| w.0 = div_x_256 << 8);
        }
    }

    pub fn get_clkdiv(&self) -> u32 {
        unsafe { Self::this_sm().clkdiv().read().0 >> 8 }
    }

    pub fn clkdiv_restart(&mut self) {
        let mask = 1u8 << SM;
        unsafe {
            PIO::PIO.ctrl().write_set(|w| w.set_clkdiv_restart(mask));
        }
    }

    /// Configures this state machine to use the given program, including jumping to the origin
    /// of the program. The state machine is not started.
    pub fn use_program(&mut self, prog: &LoadedProgram<'d, PIO>, side_set: &[&Pin<'d, PIO>]) {
        assert!((prog.side_set.bits() - prog.side_set.optional() as u8) as usize == side_set.len());
        assert_consecutive(side_set);
        unsafe {
            Self::this_sm().execctrl().modify(|w| {
                w.set_side_en(prog.side_set.optional());
                w.set_side_pindir(prog.side_set.pindirs());
                w.set_wrap_bottom(prog.wrap.target);
                w.set_wrap_top(prog.wrap.source);
            });
            Self::this_sm().pinctrl().modify(|w| {
                w.set_sideset_count(prog.side_set.bits());
                w.set_sideset_base(side_set.first().map_or(0, |p| p.pin()));
            });
            pio_instr_util::exec_jmp(self, prog.origin);
        }
    }

    pub fn set_jmp_pin(&mut self, pin: u8) {
        unsafe {
            Self::this_sm().execctrl().modify(|w| w.set_jmp_pin(pin));
        }
    }

    pub fn get_jmp_pin(&mut self) -> u8 {
        unsafe { Self::this_sm().execctrl().read().jmp_pin() }
    }

    pub fn set_fifo_join(&mut self, join: FifoJoin) {
        let (rx, tx) = match join {
            FifoJoin::Duplex => (false, false),
            FifoJoin::RxOnly => (true, false),
            FifoJoin::TxOnly => (false, true),
        };
        unsafe {
            Self::this_sm().shiftctrl().modify(|w| {
                w.set_fjoin_rx(rx);
                w.set_fjoin_tx(tx)
            });
        }
    }
    pub fn get_fifo_join(&self) -> FifoJoin {
        unsafe {
            let r = Self::this_sm().shiftctrl().read();
            // Ignores the invalid state when both bits are set
            if r.fjoin_rx() {
                FifoJoin::RxOnly
            } else if r.fjoin_tx() {
                FifoJoin::TxOnly
            } else {
                FifoJoin::Duplex
            }
        }
    }

    pub fn clear_fifos(&mut self) {
        // Toggle FJOIN_RX to flush FIFOs
        unsafe {
            let shiftctrl = Self::this_sm().shiftctrl();
            shiftctrl.modify(|w| {
                w.set_fjoin_rx(!w.fjoin_rx());
            });
            shiftctrl.modify(|w| {
                w.set_fjoin_rx(!w.fjoin_rx());
            });
        }
    }

    pub fn set_pull_threshold(&mut self, threshold: u8) {
        unsafe {
            Self::this_sm().shiftctrl().modify(|w| w.set_pull_thresh(threshold));
        }
    }

    pub fn get_pull_threshold(&self) -> u8 {
        unsafe { Self::this_sm().shiftctrl().read().pull_thresh() }
    }
    pub fn set_push_threshold(&mut self, threshold: u8) {
        unsafe {
            Self::this_sm().shiftctrl().modify(|w| w.set_push_thresh(threshold));
        }
    }

    pub fn get_push_threshold(&self) -> u8 {
        unsafe { Self::this_sm().shiftctrl().read().push_thresh() }
    }

    pub fn set_out_shift_dir(&mut self, dir: ShiftDirection) {
        unsafe {
            Self::this_sm()
                .shiftctrl()
                .modify(|w| w.set_out_shiftdir(dir == ShiftDirection::Right));
        }
    }
    pub fn get_out_shiftdir(&self) -> ShiftDirection {
        unsafe {
            if Self::this_sm().shiftctrl().read().out_shiftdir() {
                ShiftDirection::Right
            } else {
                ShiftDirection::Left
            }
        }
    }

    pub fn set_in_shift_dir(&mut self, dir: ShiftDirection) {
        unsafe {
            Self::this_sm()
                .shiftctrl()
                .modify(|w| w.set_in_shiftdir(dir == ShiftDirection::Right));
        }
    }
    pub fn get_in_shiftdir(&self) -> ShiftDirection {
        unsafe {
            if Self::this_sm().shiftctrl().read().in_shiftdir() {
                ShiftDirection::Right
            } else {
                ShiftDirection::Left
            }
        }
    }

    pub fn set_autopull(&mut self, auto: bool) {
        unsafe {
            Self::this_sm().shiftctrl().modify(|w| w.set_autopull(auto));
        }
    }

    pub fn is_autopull(&self) -> bool {
        unsafe { Self::this_sm().shiftctrl().read().autopull() }
    }

    pub fn set_autopush(&mut self, auto: bool) {
        unsafe {
            Self::this_sm().shiftctrl().modify(|w| w.set_autopush(auto));
        }
    }

    pub fn is_autopush(&self) -> bool {
        unsafe { Self::this_sm().shiftctrl().read().autopush() }
    }

    pub fn get_addr(&self) -> u8 {
        unsafe { Self::this_sm().addr().read().addr() }
    }

    /// Set the range of out pins affected by a set instruction.
    pub fn set_set_range(&mut self, base: u8, count: u8) {
        assert!(base + count < 32);
        unsafe {
            Self::this_sm().pinctrl().modify(|w| {
                w.set_set_base(base);
                w.set_set_count(count)
            });
        }
    }

    /// Get the range of out pins affected by a set instruction. Returns (base, count).
    pub fn get_set_range(&self) -> (u8, u8) {
        unsafe {
            let r = Self::this_sm().pinctrl().read();
            (r.set_base(), r.set_count())
        }
    }

    pub fn set_in_base_pin(&mut self, base: &Pin<PIO>) {
        unsafe {
            Self::this_sm().pinctrl().modify(|w| w.set_in_base(base.pin()));
        }
    }

    pub fn get_in_base(&self) -> u8 {
        unsafe {
            let r = Self::this_sm().pinctrl().read();
            r.in_base()
        }
    }

    pub fn set_out_range(&mut self, base: u8, count: u8) {
        assert!(base + count < 32);
        unsafe {
            Self::this_sm().pinctrl().modify(|w| {
                w.set_out_base(base);
                w.set_out_count(count)
            });
        }
    }

    /// Get the range of out pins affected by a set instruction. Returns (base, count).
    pub fn get_out_range(&self) -> (u8, u8) {
        unsafe {
            let r = Self::this_sm().pinctrl().read();
            (r.out_base(), r.out_count())
        }
    }

    pub fn set_out_pins<'a, 'b: 'a>(&'a mut self, pins: &'b [&Pin<PIO>]) {
        let count = pins.len();
        assert!(count >= 1);
        let start = pins[0].pin() as usize;
        assert!(start + pins.len() <= 32);
        for i in 0..count {
            assert!(pins[i].pin() as usize == start + i, "Pins must be sequential");
        }
        self.set_out_range(start as u8, count as u8);
    }

    pub fn set_set_pins<'a, 'b: 'a>(&'a mut self, pins: &'b [&Pin<PIO>]) {
        let count = pins.len();
        assert!(count >= 1);
        let start = pins[0].pin() as usize;
        assert!(start + pins.len() <= 32);
        for i in 0..count {
            assert!(pins[i].pin() as usize == start + i, "Pins must be sequential");
        }
        self.set_set_range(start as u8, count as u8);
    }

    pub fn get_current_instr() -> u32 {
        unsafe { Self::this_sm().instr().read().0 }
    }

    pub fn exec_instr(&mut self, instr: u16) {
        unsafe {
            Self::this_sm().instr().write(|w| w.set_instr(instr));
        }
    }

    pub fn rx(&mut self) -> &mut StateMachineRx<'d, PIO, SM> {
        &mut self.rx
    }
    pub fn tx(&mut self) -> &mut StateMachineTx<'d, PIO, SM> {
        &mut self.tx
    }
    pub fn rx_tx(&mut self) -> (&mut StateMachineRx<'d, PIO, SM>, &mut StateMachineTx<'d, PIO, SM>) {
        (&mut self.rx, &mut self.tx)
    }
}

pub struct Common<'d, PIO: Instance> {
    instructions_used: u32,
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> Drop for Common<'d, PIO> {
    fn drop(&mut self) {
        on_pio_drop::<PIO>();
    }
}

pub struct InstanceMemory<'d, PIO: Instance> {
    used_mask: u32,
    pio: PhantomData<&'d mut PIO>,
}

pub struct LoadedProgram<'d, PIO: Instance> {
    pub used_memory: InstanceMemory<'d, PIO>,
    origin: u8,
    wrap: Wrap,
    side_set: SideSet,
}

impl<'d, PIO: Instance> Common<'d, PIO> {
    pub fn load_program<const SIZE: usize>(&mut self, prog: &RelocatedProgram<SIZE>) -> LoadedProgram<'d, PIO> {
        match self.try_load_program(prog) {
            Ok(r) => r,
            Err(at) => panic!("Trying to write already used PIO instruction memory at {}", at),
        }
    }

    pub fn try_load_program<const SIZE: usize>(
        &mut self,
        prog: &RelocatedProgram<SIZE>,
    ) -> Result<LoadedProgram<'d, PIO>, usize> {
        let used_memory = self.try_write_instr(prog.origin() as _, prog.code())?;
        Ok(LoadedProgram {
            used_memory,
            origin: prog.origin(),
            wrap: prog.wrap(),
            side_set: prog.side_set(),
        })
    }

    pub fn try_write_instr<I>(&mut self, start: usize, instrs: I) -> Result<InstanceMemory<'d, PIO>, usize>
    where
        I: Iterator<Item = u16>,
    {
        let mut used_mask = 0;
        for (i, instr) in instrs.enumerate() {
            let addr = (i + start) as u8;
            let mask = 1 << (addr as usize);
            if self.instructions_used & mask != 0 {
                return Err(addr as usize);
            }
            unsafe {
                PIO::PIO.instr_mem(addr as usize).write(|w| {
                    w.set_instr_mem(instr);
                });
            }
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

    pub fn set_input_sync_bypass<'a>(&'a mut self, bypass: u32, mask: u32) {
        unsafe {
            // this can interfere with per-pin bypass functions. splitting the
            // modification is going to be fine since nothing that relies on
            // it can reasonably run before we finish.
            PIO::PIO.input_sync_bypass().write_set(|w| *w = mask & bypass);
            PIO::PIO.input_sync_bypass().write_clear(|w| *w = mask & !bypass);
        }
    }

    pub fn get_input_sync_bypass(&self) -> u32 {
        unsafe { PIO::PIO.input_sync_bypass().read() }
    }

    /// Register a pin for PIO usage. Pins will be released from the PIO block
    /// (i.e., have their `FUNCSEL` reset to `NULL`) when the [`Common`] *and*
    /// all [`StateMachine`]s for this block have been dropped. **Other members
    /// of [`Pio`] do not keep pin registrations alive.**
    pub fn make_pio_pin(&mut self, pin: impl Peripheral<P = impl PioPin + 'd> + 'd) -> Pin<'d, PIO> {
        into_ref!(pin);
        unsafe {
            pin.io().ctrl().write(|w| w.set_funcsel(PIO::FUNCSEL.0));
        }
        // we can be relaxed about this because we're &mut here and nothing is cached
        PIO::state().used_pins.fetch_or(1 << pin.pin_bank(), Ordering::Relaxed);
        Pin {
            pin: pin.into_ref().map_into(),
            pio: PhantomData::default(),
        }
    }
}

pub struct Irq<'d, PIO: Instance, const N: usize> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance, const N: usize> Irq<'d, PIO, N> {
    pub fn wait<'a>(&'a mut self) -> IrqFuture<'a, 'd, PIO> {
        IrqFuture {
            pio: PhantomData,
            irq_no: N as u8,
        }
    }
}

#[derive(Clone)]
pub struct IrqFlags<'d, PIO: Instance> {
    pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> IrqFlags<'d, PIO> {
    pub fn check(&self, irq_no: u8) -> bool {
        assert!(irq_no < 8);
        self.check_any(1 << irq_no)
    }

    pub fn check_any(&self, irqs: u8) -> bool {
        unsafe { PIO::PIO.irq().read().irq() & irqs != 0 }
    }

    pub fn check_all(&self, irqs: u8) -> bool {
        unsafe { PIO::PIO.irq().read().irq() & irqs == irqs }
    }

    pub fn clear(&self, irq_no: usize) {
        assert!(irq_no < 8);
        self.clear_all(1 << irq_no);
    }

    pub fn clear_all(&self, irqs: u8) {
        unsafe { PIO::PIO.irq().write(|w| w.set_irq(irqs)) }
    }

    pub fn set(&self, irq_no: usize) {
        assert!(irq_no < 8);
        self.set_all(1 << irq_no);
    }

    pub fn set_all(&self, irqs: u8) {
        unsafe { PIO::PIO.irq_force().write(|w| w.set_irq_force(irqs)) }
    }
}

pub struct Pio<'d, PIO: Instance> {
    pub common: Common<'d, PIO>,
    pub irq_flags: IrqFlags<'d, PIO>,
    pub irq0: Irq<'d, PIO, 0>,
    pub irq1: Irq<'d, PIO, 1>,
    pub irq2: Irq<'d, PIO, 2>,
    pub irq3: Irq<'d, PIO, 3>,
    pub sm0: StateMachine<'d, PIO, 0>,
    pub sm1: StateMachine<'d, PIO, 1>,
    pub sm2: StateMachine<'d, PIO, 2>,
    pub sm3: StateMachine<'d, PIO, 3>,
    _pio: PhantomData<&'d mut PIO>,
}

impl<'d, PIO: Instance> Pio<'d, PIO> {
    pub fn new(_pio: impl Peripheral<P = PIO> + 'd) -> Self {
        PIO::state().users.store(5, Ordering::Release);
        PIO::state().used_pins.store(0, Ordering::Release);
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

// we need to keep a record of which pins are assigned to each PIO. make_pio_pin
// notionally takes ownership of the pin it is given, but the wrapped pin cannot
// be treated as an owned resource since dropping it would have to deconfigure
// the pin, breaking running state machines in the process. pins are also shared
// between all state machines, which makes ownership even messier to track any
// other way.
pub struct State {
    users: AtomicU8,
    used_pins: AtomicU32,
}

fn on_pio_drop<PIO: Instance>() {
    let state = PIO::state();
    if state.users.fetch_sub(1, Ordering::AcqRel) == 1 {
        let used_pins = state.used_pins.load(Ordering::Relaxed);
        let null = Gpio0ctrlFuncsel::NULL.0;
        for i in 0..32 {
            if used_pins & (1 << i) != 0 {
                unsafe {
                    pac::IO_BANK0.gpio(i).ctrl().write(|w| w.set_funcsel(null));
                }
            }
        }
    }
}

mod sealed {
    use super::*;

    pub trait PioPin {}

    pub trait Instance {
        const PIO_NO: u8;
        const PIO: &'static crate::pac::pio::Pio;
        const FUNCSEL: crate::pac::io::vals::Gpio0ctrlFuncsel;

        #[inline]
        fn state() -> &'static State {
            static STATE: State = State {
                users: AtomicU8::new(0),
                used_pins: AtomicU32::new(0),
            };

            &STATE
        }
    }
}

pub trait Instance: sealed::Instance + Sized + Unpin {}

macro_rules! impl_pio {
    ($name:ident, $pio:expr, $pac:ident, $funcsel:ident) => {
        impl sealed::Instance for peripherals::$name {
            const PIO_NO: u8 = $pio;
            const PIO: &'static pac::pio::Pio = &pac::$pac;
            const FUNCSEL: pac::io::vals::Gpio0ctrlFuncsel = pac::io::vals::Gpio0ctrlFuncsel::$funcsel;
        }
        impl Instance for peripherals::$name {}
    };
}

impl_pio!(PIO0, 0, PIO0, PIO0_0);
impl_pio!(PIO1, 1, PIO1, PIO1_0);

pub trait PioPin: sealed::PioPin + gpio::Pin {}

macro_rules! impl_pio_pin {
    ($( $num:tt )*) => {
        $(
            paste::paste!{
                impl sealed::PioPin for peripherals::[< PIN_ $num >] {}
                impl PioPin for peripherals::[< PIN_ $num >] {}
            }
        )*
    };
}

impl_pio_pin! {
    0 1 2 3 4 5 6 7 8 9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
}
