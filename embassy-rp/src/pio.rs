use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin as FuturePin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::PeripheralRef;
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Channel, Transfer};
use crate::gpio::sealed::Pin as SealedPin;
use crate::gpio::{Drive, Pin, Pull, SlewRate};
use crate::pac::dma::vals::{DataSize, TreqSel};
use crate::{interrupt, pac, peripherals};

const PIOS: [&pac::pio::Pio; 2] = [&pac::PIO0, &pac::PIO1];
const NEW_AW: AtomicWaker = AtomicWaker::new();
const PIO_WAKERS_INIT: [AtomicWaker; 4] = [NEW_AW; 4];
static FIFO_OUT_WAKERS: [[AtomicWaker; 4]; 2] = [PIO_WAKERS_INIT; 2];
static FIFO_IN_WAKERS: [[AtomicWaker; 4]; 2] = [PIO_WAKERS_INIT; 2];
static IRQ_WAKERS: [[AtomicWaker; 4]; 2] = [PIO_WAKERS_INIT; 2];

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
unsafe fn PIO0_IRQ_1() {
    use crate::pac;
    let ints = pac::PIO0.irqs(1).ints().read().0;
    let inte = pac::PIO0.irqs(1).inte();
    for i in 0..4 {
        // Check RXNEMPTY
        if ints & (RXNEMPTY_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(RXNEMPTY_MASK << i);
            });
            FIFO_IN_WAKERS[0][i].wake();
        }
        // Check IRQ flgs
        if ints & (SMIRQ_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(SMIRQ_MASK << i);
            });
            IRQ_WAKERS[0][i].wake();
        }
    }
}

#[interrupt]
unsafe fn PIO1_IRQ_1() {
    use crate::pac;
    let ints = pac::PIO1.irqs(1).ints().read().0;
    let inte = pac::PIO1.irqs(1).inte();
    for i in 0..4 {
        // Check all RXNEMPTY
        if ints & (RXNEMPTY_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(RXNEMPTY_MASK << i);
            });
            FIFO_IN_WAKERS[1][i].wake();
        }
        // Check IRQ flgs
        if ints & (SMIRQ_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(SMIRQ_MASK << i);
            });
            IRQ_WAKERS[1][i].wake();
        }
    }
}

#[interrupt]
unsafe fn PIO0_IRQ_0() {
    use crate::pac;
    let ints = pac::PIO0.irqs(0).ints().read().0;
    let inte = pac::PIO0.irqs(0).inte();
    //debug!("!{:04x}",ints);
    // Check all TXNFULL
    for i in 0..4 {
        if ints & (TXNFULL_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(TXNFULL_MASK << i);
            });
            FIFO_OUT_WAKERS[0][i].wake();
        }
    }
}

#[interrupt]
unsafe fn PIO1_IRQ_0() {
    use crate::pac;
    let ints = pac::PIO1.irqs(0).ints().read().0;
    let inte = pac::PIO1.irqs(0).inte();
    // Check all TXNFULL
    for i in 0..4 {
        if ints & (TXNFULL_MASK << i) != 0 {
            inte.modify(|m| {
                m.0 &= !(TXNFULL_MASK << i);
            });
            FIFO_OUT_WAKERS[1][i].wake();
        }
    }
}

/// Future that waits for TX-FIFO to become writable
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoOutFuture<'a, PIO: PioInstance, SM: PioStateMachine + Unpin> {
    sm: &'a mut SM,
    pio: PhantomData<PIO>,
    value: u32,
}

impl<'a, PIO: PioInstance, SM: PioStateMachine + Unpin> FifoOutFuture<'a, PIO, SM> {
    pub fn new(sm: &'a mut SM, value: u32) -> Self {
        unsafe {
            critical_section::with(|_| {
                let irq = PIO::IrqOut::steal();
                irq.set_priority(interrupt::Priority::P3);

                irq.enable();
            });
        }
        FifoOutFuture {
            sm,
            pio: PhantomData::default(),
            value,
        }
    }
}

impl<'d, PIO: PioInstance, SM: PioStateMachine + Unpin> Future for FifoOutFuture<'d, PIO, SM> {
    type Output = ();
    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);
        let value = self.value;
        if self.get_mut().sm.try_push_tx(value) {
            Poll::Ready(())
        } else {
            FIFO_OUT_WAKERS[PIO::PIO_NO as usize][SM::Sm::SM_NO as usize].register(cx.waker());
            unsafe {
                let irq = PIO::IrqOut::steal();
                irq.disable();
                critical_section::with(|_| {
                    PIOS[PIO::PIO_NO as usize].irqs(0).inte().modify(|m| {
                        m.0 |= TXNFULL_MASK << SM::Sm::SM_NO;
                    });
                });
                irq.enable();
            }
            // debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'d, PIO: PioInstance, SM: PioStateMachine + Unpin> Drop for FifoOutFuture<'d, PIO, SM> {
    fn drop(&mut self) {
        unsafe {
            critical_section::with(|_| {
                PIOS[PIO::PIO_NO as usize].irqs(0).inte().modify(|m| {
                    m.0 &= !(TXNFULL_MASK << SM::Sm::SM_NO);
                });
            });
        }
    }
}

/// Future that waits for RX-FIFO to become readable
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FifoInFuture<'a, PIO: PioInstance, SM: PioStateMachine> {
    sm: &'a mut SM,
    pio: PhantomData<PIO>,
}

impl<'a, PIO: PioInstance, SM: PioStateMachine> FifoInFuture<'a, PIO, SM> {
    pub fn new(sm: &'a mut SM) -> Self {
        unsafe {
            critical_section::with(|_| {
                let irq = PIO::IrqIn::steal();
                irq.set_priority(interrupt::Priority::P3);

                irq.enable();
            });
        }
        FifoInFuture {
            sm,
            pio: PhantomData::default(),
        }
    }
}

impl<'d, PIO: PioInstance, SM: PioStateMachine> Future for FifoInFuture<'d, PIO, SM> {
    type Output = u32;
    fn poll(mut self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);
        if let Some(v) = self.sm.try_pull_rx() {
            Poll::Ready(v)
        } else {
            FIFO_IN_WAKERS[PIO::PIO_NO as usize][SM::Sm::SM_NO as usize].register(cx.waker());
            unsafe {
                let irq = PIO::IrqIn::steal();
                irq.disable();
                critical_section::with(|_| {
                    PIOS[PIO::PIO_NO as usize].irqs(1).inte().modify(|m| {
                        m.0 |= RXNEMPTY_MASK << SM::Sm::SM_NO;
                    });
                });
                irq.enable();
            }
            //debug!("Pending");
            Poll::Pending
        }
    }
}

impl<'d, PIO: PioInstance, SM: PioStateMachine> Drop for FifoInFuture<'d, PIO, SM> {
    fn drop(&mut self) {
        unsafe {
            critical_section::with(|_| {
                PIOS[PIO::PIO_NO as usize].irqs(1).inte().modify(|m| {
                    m.0 &= !(RXNEMPTY_MASK << SM::Sm::SM_NO);
                });
            });
        }
    }
}

/// Future that waits for IRQ
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct IrqFuture<PIO: PioInstance> {
    pio: PhantomData<PIO>,
    irq_no: u8,
}

impl<'a, PIO: PioInstance> IrqFuture<PIO> {
    pub fn new(irq_no: u8) -> Self {
        unsafe {
            critical_section::with(|_| {
                let irq = PIO::IrqSm::steal();
                irq.set_priority(interrupt::Priority::P3);

                irq.enable();
            });
        }
        IrqFuture {
            pio: PhantomData::default(),
            irq_no,
        }
    }
}

impl<'d, PIO: PioInstance> Future for IrqFuture<PIO> {
    type Output = ();
    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("Poll {},{}", PIO::PIO_NO, SM);

        // Check if IRQ flag is already set
        if critical_section::with(|_| unsafe {
            let irq_flags = PIOS[PIO::PIO_NO as usize].irq();
            if irq_flags.read().0 & (1 << self.irq_no) != 0 {
                irq_flags.write(|m| {
                    m.0 = 1 << self.irq_no;
                });
                true
            } else {
                false
            }
        }) {
            return Poll::Ready(());
        }

        IRQ_WAKERS[PIO::PIO_NO as usize][self.irq_no as usize].register(cx.waker());
        unsafe {
            let irq = PIO::IrqSm::steal();
            irq.disable();
            critical_section::with(|_| {
                PIOS[PIO::PIO_NO as usize].irqs(1).inte().modify(|m| {
                    m.0 |= SMIRQ_MASK << self.irq_no;
                });
            });
            irq.enable();
        }
        Poll::Pending
    }
}

impl<'d, PIO: PioInstance> Drop for IrqFuture<PIO> {
    fn drop(&mut self) {
        unsafe {
            critical_section::with(|_| {
                PIOS[PIO::PIO_NO as usize].irqs(1).inte().modify(|m| {
                    m.0 &= !(SMIRQ_MASK << self.irq_no);
                });
            });
        }
    }
}

pub struct PioPin<PIO: PioInstance> {
    pin_bank: u8,
    pio: PhantomData<PIO>,
}

impl<PIO: PioInstance> PioPin<PIO> {
    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        unsafe {
            self.pad_ctrl().modify(|w| {
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
            self.pad_ctrl().modify(|w| {
                w.set_slewfast(slew_rate == SlewRate::Fast);
            });
        }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        unsafe {
            self.pad_ctrl().modify(|w| match pull {
                Pull::Up => w.set_pue(true),
                Pull::Down => w.set_pde(true),
                Pull::None => {}
            });
        }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_schmitt(&mut self, enable: bool) {
        unsafe {
            self.pad_ctrl().modify(|w| {
                w.set_schmitt(enable);
            });
        }
    }

    pub fn set_input_sync_bypass<'a>(&mut self, bypass: bool) {
        let mask = 1 << self.pin();
        unsafe {
            PIOS[PIO::PIO_NO as usize]
                .input_sync_bypass()
                .modify(|w| *w = if bypass { *w & !mask } else { *w | mask });
        }
    }

    pub fn pin(&self) -> u8 {
        self._pin()
    }
}

impl<PIO: PioInstance> SealedPin for PioPin<PIO> {
    fn pin_bank(&self) -> u8 {
        self.pin_bank
    }
}

pub struct PioStateMachineInstance<PIO: PioInstance, SM: SmInstance> {
    pio: PhantomData<PIO>,
    sm: PhantomData<SM>,
}

impl<PIO: PioInstance, SM: SmInstance> PioStateMachine for PioStateMachineInstance<PIO, SM> {
    type Pio = PIO;
    type Sm = SM;
}

pub trait PioStateMachine: Sized + Unpin {
    type Pio: PioInstance;
    type Sm: SmInstance;
    fn pio_no(&self) -> u8 {
        let _ = self;
        Self::Pio::PIO_NO
    }

    fn sm_no(&self) -> u8 {
        Self::Sm::SM_NO
    }

    fn restart(&mut self) {
        let _ = self;
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .ctrl()
                .modify(|w| w.set_sm_restart(1u8 << Self::Sm::SM_NO));
        }
    }
    fn set_enable(&mut self, enable: bool) {
        let _ = self;
        let mask = 1u8 << Self::Sm::SM_NO;
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .ctrl()
                .modify(|w| w.set_sm_enable((w.sm_enable() & !mask) | (if enable { mask } else { 0 })));
        }
    }

    fn is_enabled(&self) -> bool {
        let _ = self;
        unsafe { PIOS[Self::Pio::PIO_NO as usize].ctrl().read().sm_enable() & (1u8 << Self::Sm::SM_NO) != 0 }
    }

    fn is_tx_empty(&self) -> bool {
        let _ = self;
        unsafe { PIOS[Self::Pio::PIO_NO as usize].fstat().read().txempty() & (1u8 << Self::Sm::SM_NO) != 0 }
    }
    fn is_tx_full(&self) -> bool {
        let _ = self;
        unsafe { PIOS[Self::Pio::PIO_NO as usize].fstat().read().txfull() & (1u8 << Self::Sm::SM_NO) != 0 }
    }

    fn is_rx_empty(&self) -> bool {
        let _ = self;
        unsafe { PIOS[Self::Pio::PIO_NO as usize].fstat().read().rxempty() & (1u8 << Self::Sm::SM_NO) != 0 }
    }
    fn is_rx_full(&self) -> bool {
        let _ = self;
        unsafe { PIOS[Self::Pio::PIO_NO as usize].fstat().read().rxfull() & (1u8 << Self::Sm::SM_NO) != 0 }
    }

    fn tx_level(&self) -> u8 {
        unsafe {
            let flevel = PIOS[Self::Pio::PIO_NO as usize].flevel().read().0;
            (flevel >> (Self::Sm::SM_NO * 8)) as u8 & 0x0f
        }
    }

    fn rx_level(&self) -> u8 {
        unsafe {
            let flevel = PIOS[Self::Pio::PIO_NO as usize].flevel().read().0;
            (flevel >> (Self::Sm::SM_NO * 8 + 4)) as u8 & 0x0f
        }
    }

    fn push_tx(&mut self, v: u32) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .txf(Self::Sm::SM_NO as usize)
                .write_value(v);
        }
    }

    fn try_push_tx(&mut self, v: u32) -> bool {
        if self.is_tx_full() {
            return false;
        }
        self.push_tx(v);
        true
    }

    fn pull_rx(&mut self) -> u32 {
        unsafe { PIOS[Self::Pio::PIO_NO as usize].rxf(Self::Sm::SM_NO as usize).read() }
    }

    fn try_pull_rx(&mut self) -> Option<u32> {
        if self.is_rx_empty() {
            return None;
        }
        Some(self.pull_rx())
    }

    fn set_clkdiv(&mut self, div_x_256: u32) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .clkdiv()
                .write(|w| w.0 = div_x_256 << 8);
        }
    }

    fn get_clkdiv(&self) -> u32 {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .clkdiv()
                .read()
                .0
                >> 8
        }
    }

    fn clkdiv_restart(&mut self) {
        let _ = self;
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .ctrl()
                .modify(|w| w.set_clkdiv_restart(1u8 << Self::Sm::SM_NO));
        }
    }

    fn set_side_enable(&self, enable: bool) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .modify(|w| w.set_side_en(enable));
        }
    }

    fn is_side_enabled(&self) -> bool {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .read()
                .side_en()
        }
    }

    fn set_side_pindir(&mut self, pindir: bool) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .modify(|w| w.set_side_pindir(pindir));
        }
    }

    fn is_side_pindir(&self) -> bool {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .read()
                .side_pindir()
        }
    }

    fn set_jmp_pin(&mut self, pin: u8) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .modify(|w| w.set_jmp_pin(pin));
        }
    }

    fn get_jmp_pin(&mut self) -> u8 {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .read()
                .jmp_pin()
        }
    }

    fn set_wrap(&self, source: u8, target: u8) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .modify(|w| {
                    w.set_wrap_top(source);
                    w.set_wrap_bottom(target)
                });
        }
    }

    /// Get wrapping addresses. Returns (source, target).
    fn get_wrap(&self) -> (u8, u8) {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .execctrl()
                .read();
            (r.wrap_top(), r.wrap_bottom())
        }
    }

    fn set_fifo_join(&mut self, join: FifoJoin) {
        let (rx, tx) = match join {
            FifoJoin::Duplex => (false, false),
            FifoJoin::RxOnly => (true, false),
            FifoJoin::TxOnly => (false, true),
        };
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| {
                    w.set_fjoin_rx(rx);
                    w.set_fjoin_tx(tx)
                });
        }
    }
    fn get_fifo_join(&self) -> FifoJoin {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read();
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

    fn clear_fifos(&mut self) {
        // Toggle FJOIN_RX to flush FIFOs
        unsafe {
            let shiftctrl = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl();
            shiftctrl.modify(|w| {
                w.set_fjoin_rx(!w.fjoin_rx());
            });
            shiftctrl.modify(|w| {
                w.set_fjoin_rx(!w.fjoin_rx());
            });
        }
    }

    fn set_pull_threshold(&mut self, threshold: u8) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_pull_thresh(threshold));
        }
    }

    fn get_pull_threshold(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read();
            r.pull_thresh()
        }
    }
    fn set_push_threshold(&mut self, threshold: u8) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_push_thresh(threshold));
        }
    }

    fn get_push_threshold(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read();
            r.push_thresh()
        }
    }

    fn set_out_shift_dir(&mut self, dir: ShiftDirection) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_out_shiftdir(dir == ShiftDirection::Right));
        }
    }
    fn get_out_shiftdir(&self) -> ShiftDirection {
        unsafe {
            if PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read()
                .out_shiftdir()
            {
                ShiftDirection::Right
            } else {
                ShiftDirection::Left
            }
        }
    }

    fn set_in_shift_dir(&mut self, dir: ShiftDirection) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_in_shiftdir(dir == ShiftDirection::Right));
        }
    }
    fn get_in_shiftdir(&self) -> ShiftDirection {
        unsafe {
            if PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read()
                .in_shiftdir()
            {
                ShiftDirection::Right
            } else {
                ShiftDirection::Left
            }
        }
    }

    fn set_autopull(&mut self, auto: bool) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_autopull(auto));
        }
    }

    fn is_autopull(&self) -> bool {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read()
                .autopull()
        }
    }

    fn set_autopush(&mut self, auto: bool) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .modify(|w| w.set_autopush(auto));
        }
    }

    fn is_autopush(&self) -> bool {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .shiftctrl()
                .read()
                .autopush()
        }
    }

    fn get_addr(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .addr()
                .read();
            r.addr()
        }
    }
    fn set_sideset_count(&mut self, count: u8) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .modify(|w| w.set_sideset_count(count));
        }
    }

    fn get_sideset_count(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .read();
            r.sideset_count()
        }
    }

    fn make_pio_pin(&self, pin: impl Pin) -> PioPin<Self::Pio> {
        unsafe {
            pin.io().ctrl().write(|w| {
                w.set_funcsel(
                    if Self::Pio::PIO_NO == 1 {
                        pac::io::vals::Gpio0ctrlFuncsel::PIO1_0
                    } else {
                        // PIO == 0
                        pac::io::vals::Gpio0ctrlFuncsel::PIO0_0
                    }
                    .0,
                );
            });
        }
        PioPin {
            pin_bank: pin.pin_bank(),
            pio: PhantomData::default(),
        }
    }

    fn set_sideset_base_pin(&mut self, base_pin: &PioPin<Self::Pio>) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .modify(|w| w.set_sideset_base(base_pin.pin()));
        }
    }

    fn get_sideset_base(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .read();
            r.sideset_base()
        }
    }

    /// Set the range of out pins affected by a set instruction.
    fn set_set_range(&mut self, base: u8, count: u8) {
        assert!(base + count < 32);
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .modify(|w| {
                    w.set_set_base(base);
                    w.set_set_count(count)
                });
        }
    }

    /// Get the range of out pins affected by a set instruction. Returns (base, count).
    fn get_set_range(&self) -> (u8, u8) {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .read();
            (r.set_base(), r.set_count())
        }
    }

    fn set_in_base_pin(&mut self, base: &PioPin<Self::Pio>) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .modify(|w| w.set_in_base(base.pin()));
        }
    }

    fn get_in_base(&self) -> u8 {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .read();
            r.in_base()
        }
    }

    fn set_out_range(&mut self, base: u8, count: u8) {
        assert!(base + count < 32);
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .modify(|w| {
                    w.set_out_base(base);
                    w.set_out_count(count)
                });
        }
    }

    /// Get the range of out pins affected by a set instruction. Returns (base, count).
    fn get_out_range(&self) -> (u8, u8) {
        unsafe {
            let r = PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .pinctrl()
                .read();
            (r.out_base(), r.out_count())
        }
    }

    fn set_out_pins<'a, 'b: 'a>(&'a mut self, pins: &'b [&PioPin<Self::Pio>]) {
        let count = pins.len();
        assert!(count >= 1);
        let start = pins[0].pin() as usize;
        assert!(start + pins.len() <= 32);
        for i in 0..count {
            assert!(pins[i].pin() as usize == start + i, "Pins must be sequential");
        }
        self.set_out_range(start as u8, count as u8);
    }

    fn set_set_pins<'a, 'b: 'a>(&'a mut self, pins: &'b [&PioPin<Self::Pio>]) {
        let count = pins.len();
        assert!(count >= 1);
        let start = pins[0].pin() as usize;
        assert!(start + pins.len() <= 32);
        for i in 0..count {
            assert!(pins[i].pin() as usize == start + i, "Pins must be sequential");
        }
        self.set_set_range(start as u8, count as u8);
    }

    fn get_current_instr() -> u32 {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .instr()
                .read()
                .0
        }
    }

    fn exec_instr(&mut self, instr: u16) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .sm(Self::Sm::SM_NO as usize)
                .instr()
                .write(|w| w.set_instr(instr));
        }
    }

    fn write_instr<I>(&mut self, start: usize, instrs: I)
    where
        I: Iterator<Item = u16>,
    {
        let _ = self;
        write_instr(
            Self::Pio::PIO_NO,
            start,
            instrs,
            MEM_USED_BY_STATEMACHINE | Self::Sm::SM_NO as u32,
        );
    }

    fn is_irq_set(&self, irq_no: u8) -> bool {
        assert!(irq_no < 8);
        unsafe {
            let irq_flags = PIOS[Self::Pio::PIO_NO as usize].irq();
            irq_flags.read().0 & (1 << irq_no) != 0
        }
    }

    fn clear_irq(&mut self, irq_no: usize) {
        assert!(irq_no < 8);
        unsafe { PIOS[Self::Pio::PIO_NO as usize].irq().write(|w| w.set_irq(1 << irq_no)) }
    }

    fn wait_push<'a>(&'a mut self, value: u32) -> FifoOutFuture<'a, Self::Pio, Self> {
        FifoOutFuture::new(self, value)
    }

    fn wait_pull<'a>(&'a mut self) -> FifoInFuture<'a, Self::Pio, Self> {
        FifoInFuture::new(self)
    }

    fn wait_irq(&self, irq_no: u8) -> IrqFuture<Self::Pio> {
        IrqFuture::new(irq_no)
    }

    fn has_tx_stalled(&self) -> bool {
        unsafe {
            let fdebug = PIOS[Self::Pio::PIO_NO as usize].fdebug();
            let ret = fdebug.read().txstall() & (1 << Self::Sm::SM_NO) != 0;
            fdebug.write(|w| w.set_txstall(1 << Self::Sm::SM_NO));
            ret
        }
    }

    fn has_tx_overflowed(&self) -> bool {
        unsafe {
            let fdebug = PIOS[Self::Pio::PIO_NO as usize].fdebug();
            let ret = fdebug.read().txover() & (1 << Self::Sm::SM_NO) != 0;
            fdebug.write(|w| w.set_txover(1 << Self::Sm::SM_NO));
            ret
        }
    }

    fn has_rx_stalled(&self) -> bool {
        unsafe {
            let fdebug = PIOS[Self::Pio::PIO_NO as usize].fdebug();
            let ret = fdebug.read().rxstall() & (1 << Self::Sm::SM_NO) != 0;
            fdebug.write(|w| w.set_rxstall(1 << Self::Sm::SM_NO));
            ret
        }
    }

    fn has_rx_underflowed(&self) -> bool {
        unsafe {
            let fdebug = PIOS[Self::Pio::PIO_NO as usize].fdebug();
            let ret = fdebug.read().rxunder() & (1 << Self::Sm::SM_NO) != 0;
            fdebug.write(|w| w.set_rxunder(1 << Self::Sm::SM_NO));
            ret
        }
    }

    fn dma_push<'a, C: Channel>(&'a self, ch: PeripheralRef<'a, C>, data: &'a [u32]) -> Transfer<'a, C> {
        unsafe {
            dma::init();
            let pio_no = Self::Pio::PIO_NO;
            let sm_no = Self::Sm::SM_NO;
            let p = ch.regs();
            p.read_addr().write_value(data.as_ptr() as u32);
            p.write_addr()
                .write_value(PIOS[pio_no as usize].txf(sm_no as usize).ptr() as u32);
            p.trans_count().write_value(data.len() as u32);
            p.ctrl_trig().write(|w| {
                // Set TX DREQ for this statemachine
                w.set_treq_sel(TreqSel(pio_no * 8 + sm_no));
                w.set_data_size(DataSize::SIZE_WORD);
                w.set_chain_to(ch.number());
                w.set_incr_read(true);
                w.set_incr_write(false);
                w.set_en(true);
            });
            compiler_fence(Ordering::SeqCst);
        }
        Transfer::new(ch)
    }

    fn dma_pull<'a, C: Channel>(&'a self, ch: PeripheralRef<'a, C>, data: &'a mut [u32]) -> Transfer<'a, C> {
        unsafe {
            dma::init();
            let pio_no = Self::Pio::PIO_NO;
            let sm_no = Self::Sm::SM_NO;
            let p = ch.regs();
            p.write_addr().write_value(data.as_ptr() as u32);
            p.read_addr()
                .write_value(PIOS[pio_no as usize].rxf(sm_no as usize).ptr() as u32);
            p.trans_count().write_value(data.len() as u32);
            p.ctrl_trig().write(|w| {
                // Set TX DREQ for this statemachine
                w.set_treq_sel(TreqSel(pio_no * 8 + sm_no + 4));
                w.set_data_size(DataSize::SIZE_WORD);
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

/*
This is a bit array containing 4 bits for every word in the PIO instruction memory.
*/
// Bit 3-2
//const MEM_USE_MASK: u32 = 0b1100;
const MEM_NOT_USED: u32 = 0b0000;
const MEM_USED_BY_STATEMACHINE: u32 = 0b0100;
const MEM_USED_BY_COMMON: u32 = 0b1000;

// Bit 1-0 is the number of the state machine
//const MEM_STATE_MASK: u32 = 0b0011;

// Should use mutex if running on multiple cores
static mut INSTR_MEM_STATUS: &'static mut [[u32; 4]; 2] = &mut [[0; 4]; 2];

fn instr_mem_get_status(pio_no: u8, addr: u8) -> u32 {
    ((unsafe { INSTR_MEM_STATUS[pio_no as usize][(addr >> 3) as usize] }) >> ((addr & 0x07) * 4)) & 0xf
}

fn instr_mem_set_status(pio_no: u8, addr: u8, status: u32) {
    let w = unsafe { &mut INSTR_MEM_STATUS[pio_no as usize][(addr >> 3) as usize] };
    let shift = (addr & 0x07) * 4;
    *w = (*w & !(0xf << shift)) | (status << shift);
}

fn instr_mem_is_free(pio_no: u8, addr: u8) -> bool {
    instr_mem_get_status(pio_no, addr) == MEM_NOT_USED
}

pub struct PioCommonInstance<PIO: PioInstance> {
    pio: PhantomData<PIO>,
}

impl<PIO: PioInstance> PioCommon for PioCommonInstance<PIO> {
    type Pio = PIO;
}

fn write_instr<I>(pio_no: u8, start: usize, instrs: I, mem_user: u32)
where
    I: Iterator<Item = u16>,
{
    for (i, instr) in instrs.enumerate() {
        let addr = (i + start) as u8;
        assert!(
            instr_mem_is_free(pio_no, addr),
            "Trying to write already used PIO instruction memory at {}",
            addr
        );
        unsafe {
            PIOS[pio_no as usize].instr_mem(addr as usize).write(|w| {
                w.set_instr_mem(instr);
            });
            instr_mem_set_status(pio_no, addr, mem_user);
        }
    }
}

pub trait PioCommon: Sized {
    type Pio: PioInstance;

    fn write_instr<I>(&mut self, start: usize, instrs: I)
    where
        I: Iterator<Item = u16>,
    {
        let _ = self;
        write_instr(Self::Pio::PIO_NO, start, instrs, MEM_USED_BY_COMMON);
    }

    fn clear_irq(&mut self, irq_no: usize) {
        assert!(irq_no < 8);
        unsafe { PIOS[Self::Pio::PIO_NO as usize].irq().write(|w| w.set_irq(1 << irq_no)) }
    }

    fn clear_irqs(&mut self, mask: u8) {
        unsafe { PIOS[Self::Pio::PIO_NO as usize].irq().write(|w| w.set_irq(mask)) }
    }

    fn force_irq(&mut self, irq_no: usize) {
        assert!(irq_no < 8);
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .irq_force()
                .write(|w| w.set_irq_force(1 << irq_no))
        }
    }

    fn set_input_sync_bypass<'a>(&'a mut self, bypass: u32, mask: u32) {
        unsafe {
            PIOS[Self::Pio::PIO_NO as usize]
                .input_sync_bypass()
                .modify(|w| *w = (*w & !mask) | (bypass & mask));
        }
    }

    fn get_input_sync_bypass(&self) -> u32 {
        unsafe { PIOS[Self::Pio::PIO_NO as usize].input_sync_bypass().read() }
    }
}

// Identifies a specific state machine inside a PIO device
pub struct SmInstanceBase<const SM_NO: u8> {}

pub trait SmInstance: Unpin {
    const SM_NO: u8;
}

impl<const SM_NO: u8> SmInstance for SmInstanceBase<SM_NO> {
    const SM_NO: u8 = SM_NO;
}

pub trait PioPeripheral: Sized {
    type Pio: PioInstance;
    fn pio(&self) -> u8 {
        let _ = self;
        Self::Pio::PIO_NO
    }

    fn split(
        self,
    ) -> (
        PioCommonInstance<Self::Pio>,
        PioStateMachineInstance<Self::Pio, SmInstanceBase<0>>,
        PioStateMachineInstance<Self::Pio, SmInstanceBase<1>>,
        PioStateMachineInstance<Self::Pio, SmInstanceBase<2>>,
        PioStateMachineInstance<Self::Pio, SmInstanceBase<3>>,
    ) {
        let _ = self;
        (
            PioCommonInstance {
                pio: PhantomData::default(),
            },
            PioStateMachineInstance {
                sm: PhantomData::default(),
                pio: PhantomData::default(),
            },
            PioStateMachineInstance {
                sm: PhantomData::default(),
                pio: PhantomData::default(),
            },
            PioStateMachineInstance {
                sm: PhantomData::default(),
                pio: PhantomData::default(),
            },
            PioStateMachineInstance {
                sm: PhantomData::default(),
                pio: PhantomData::default(),
            },
        )
    }
}

// Identifies a specific PIO device
pub struct PioInstanceBase<const PIO_NO: u8> {}

pub trait PioInstance: Unpin {
    const PIO_NO: u8;
    type IrqOut: Interrupt;
    type IrqIn: Interrupt;
    type IrqSm: Interrupt;
}

impl PioInstance for PioInstanceBase<0> {
    const PIO_NO: u8 = 0;
    type IrqOut = interrupt::PIO0_IRQ_0;
    type IrqIn = interrupt::PIO0_IRQ_1;
    type IrqSm = interrupt::PIO0_IRQ_1;
}

impl PioInstance for PioInstanceBase<1> {
    const PIO_NO: u8 = 1;
    type IrqOut = interrupt::PIO1_IRQ_0;
    type IrqIn = interrupt::PIO1_IRQ_1;
    type IrqSm = interrupt::PIO1_IRQ_1;
}

pub type Pio0 = PioInstanceBase<0>;
pub type Pio1 = PioInstanceBase<1>;

pub type Sm0 = SmInstanceBase<0>;
pub type Sm1 = SmInstanceBase<1>;
pub type Sm2 = SmInstanceBase<2>;
pub type Sm3 = SmInstanceBase<3>;

macro_rules! impl_pio_sm {
    ($name:ident, $pio:expr) => {
        impl PioPeripheral for peripherals::$name {
            type Pio = PioInstanceBase<$pio>;
        }
    };
}

impl_pio_sm!(PIO0, 0);
impl_pio_sm!(PIO1, 1);
