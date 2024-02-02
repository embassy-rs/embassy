use core::convert::AsMut;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

pub use bxcan;
use bxcan::{Data, ExtendedId, Frame, Id, StandardId};
use embassy_hal_internal::{into_ref, PeripheralRef};
use futures::FutureExt;

use crate::gpio::sealed::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::can::vals::{Ide, Lec};
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub mod enums;
use enums::*;
pub mod util;

/// Contains CAN frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Envelope {
    /// Reception time.
    #[cfg(feature = "time")]
    pub ts: embassy_time::Instant,
    /// The actual CAN frame.
    pub frame: bxcan::Frame,
}

/// Interrupt handler.
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TXInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().tsr().write(|v| {
            v.set_rqcp(0, true);
            v.set_rqcp(1, true);
            v.set_rqcp(2, true);
        });

        T::state().tx_waker.wake();
    }
}

/// RX0 interrupt handler.
pub struct Rx0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX0Interrupt> for Rx0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("rx0 irq");
        Can::<T>::receive_fifo(RxFifo::Fifo0);
    }
}

/// RX1 interrupt handler.
pub struct Rx1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX1Interrupt> for Rx1InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("rx1 irq");
        Can::<T>::receive_fifo(RxFifo::Fifo1);
    }
}

/// SCE interrupt handler.
pub struct SceInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::SCEInterrupt> for SceInterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("sce irq");
        let msr = T::regs().msr();
        let msr_val = msr.read();

        if msr_val.erri() {
            msr.modify(|v| v.set_erri(true));
            T::state().err_waker.wake();
        }
    }
}

/// CAN driver
pub struct Can<'d, T: Instance> {
    can: bxcan::Can<BxcanInstance<'d, T>>,
}

/// Error returned by `try_read`
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReadError {
    /// Bus error
    BusError(BusError),
    /// Receive buffer is empty
    Empty,
}

/// Error returned by `try_write`
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryWriteError {
    /// All transmit mailboxes are full
    Full,
}

impl<'d, T: Instance> Can<'d, T> {
    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::TXInterrupt, TxInterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX0Interrupt, Rx0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::RX1Interrupt, Rx1InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::SCEInterrupt, SceInterruptHandler<T>>
            + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable_and_reset();

        {
            T::regs().ier().write(|w| {
                w.set_errie(true);
                w.set_fmpie(0, true);
                w.set_fmpie(1, true);
                w.set_tmeie(true);
            });

            T::regs().mcr().write(|w| {
                // Enable timestamps on rx messages

                w.set_ttcm(true);
            });
        }

        unsafe {
            T::TXInterrupt::unpend();
            T::TXInterrupt::enable();

            T::RX0Interrupt::unpend();
            T::RX0Interrupt::enable();

            T::RX1Interrupt::unpend();
            T::RX1Interrupt::enable();

            T::SCEInterrupt::unpend();
            T::SCEInterrupt::enable();
        }

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        let can = bxcan::Can::builder(BxcanInstance(peri)).leave_disabled();
        Self { can }
    }

    /// Set CAN bit rate.
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();
        let sjw = u8::from(bit_timing.sync_jump_width) as u32;
        let seg1 = u8::from(bit_timing.seg1) as u32;
        let seg2 = u8::from(bit_timing.seg2) as u32;
        let prescaler = u16::from(bit_timing.prescaler) as u32;
        self.can
            .modify_config()
            .set_bit_timing((sjw - 1) << 24 | (seg1 - 1) << 16 | (seg2 - 1) << 20 | (prescaler - 1))
            .leave_disabled();
    }

    /// Enables the peripheral and synchronizes with the bus.
    ///
    /// This will wait for 11 consecutive recessive bits (bus idle state).
    /// Contrary to enable method from bxcan library, this will not freeze the executor while waiting.
    pub async fn enable(&mut self) {
        while self.enable_non_blocking().is_err() {
            // SCE interrupt is only generated for entering sleep mode, but not leaving.
            // Yield to allow other tasks to execute while can bus is initializing.
            embassy_futures::yield_now().await;
        }
    }

    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        self.split().0.write(frame).await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<bxcan::TransmitStatus, TryWriteError> {
        self.split().0.try_write(frame)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: bxcan::Mailbox) {
        CanTx::<T>::flush_inner(mb).await
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
        CanTx::<T>::flush_any_inner().await
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        CanTx::<T>::flush_all_inner().await
    }

    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.split().1.read().await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        self.split().1.try_read()
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        self.split().1.wait_not_empty().await
    }

    unsafe fn receive_fifo(fifo: RxFifo) {
        // Generate timestamp as early as possible
        #[cfg(feature = "time")]
        let ts = embassy_time::Instant::now();

        let state = T::state();
        let regs = T::regs();
        let fifo_idx = match fifo {
            RxFifo::Fifo0 => 0usize,
            RxFifo::Fifo1 => 1usize,
        };
        let rfr = regs.rfr(fifo_idx);
        let fifo = regs.rx(fifo_idx);

        loop {
            // If there are no pending messages, there is nothing to do
            if rfr.read().fmp() == 0 {
                return;
            }

            let rir = fifo.rir().read();
            let id = if rir.ide() == Ide::STANDARD {
                Id::from(StandardId::new_unchecked(rir.stid()))
            } else {
                let stid = (rir.stid() & 0x7FF) as u32;
                let exid = rir.exid() & 0x3FFFF;
                let id = (stid << 18) | (exid);
                Id::from(ExtendedId::new_unchecked(id))
            };
            let data_len = fifo.rdtr().read().dlc() as usize;
            let mut data: [u8; 8] = [0; 8];
            data[0..4].copy_from_slice(&fifo.rdlr().read().0.to_ne_bytes());
            data[4..8].copy_from_slice(&fifo.rdhr().read().0.to_ne_bytes());

            let frame = Frame::new_data(id, Data::new(&data[0..data_len]).unwrap());
            let envelope = Envelope {
                #[cfg(feature = "time")]
                ts,
                frame,
            };

            rfr.modify(|v| v.set_rfom(true));

            /*
                NOTE: consensus was reached that if rx_queue is full, packets should be dropped
            */
            let _ = state.rx_queue.try_send(envelope);
        }
    }

    /// Split the CAN driver into transmit and receive halves.
    ///
    /// Useful for doing separate transmit/receive tasks.
    pub fn split<'c>(&'c mut self) -> (CanTx<'c, 'd, T>, CanRx<'c, 'd, T>) {
        let (tx, rx0, rx1) = self.can.split_by_ref();
        (CanTx { tx }, CanRx { rx0, rx1 })
    }
}

impl<'d, T: Instance> AsMut<bxcan::Can<BxcanInstance<'d, T>>> for Can<'d, T> {
    /// Get mutable access to the lower-level driver from the `bxcan` crate.
    fn as_mut(&mut self) -> &mut bxcan::Can<BxcanInstance<'d, T>> {
        &mut self.can
    }
}

/// CAN driver, transmit half.
pub struct CanTx<'c, 'd, T: Instance> {
    tx: &'c mut bxcan::Tx<BxcanInstance<'d, T>>,
}

impl<'c, 'd, T: Instance> CanTx<'c, 'd, T> {
    /// Queues the message to be sent.
    ///
    /// If the TX queue is full, this will wait until there is space, therefore exerting backpressure.
    pub async fn write(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if let Ok(status) = self.tx.transmit(frame) {
                return Poll::Ready(status);
            }

            Poll::Pending
        })
        .await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<bxcan::TransmitStatus, TryWriteError> {
        self.tx.transmit(frame).map_err(|_| TryWriteError::Full)
    }

    async fn flush_inner(mb: bxcan::Mailbox) {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: bxcan::Mailbox) {
        Self::flush_inner(mb).await
    }

    async fn flush_any_inner() {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            let tsr = T::regs().tsr().read();
            if tsr.tme(bxcan::Mailbox::Mailbox0.index())
                || tsr.tme(bxcan::Mailbox::Mailbox1.index())
                || tsr.tme(bxcan::Mailbox::Mailbox2.index())
            {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
        Self::flush_any_inner().await
    }

    async fn flush_all_inner() {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            let tsr = T::regs().tsr().read();
            if tsr.tme(bxcan::Mailbox::Mailbox0.index())
                && tsr.tme(bxcan::Mailbox::Mailbox1.index())
                && tsr.tme(bxcan::Mailbox::Mailbox2.index())
            {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        Self::flush_all_inner().await
    }
}

/// CAN driver, receive half.
#[allow(dead_code)]
pub struct CanRx<'c, 'd, T: Instance> {
    rx0: &'c mut bxcan::Rx0<BxcanInstance<'d, T>>,
    rx1: &'c mut bxcan::Rx1<BxcanInstance<'d, T>>,
}

impl<'c, 'd, T: Instance> CanRx<'c, 'd, T> {
    /// Read a CAN frame.
    ///
    /// If no CAN frame is in the RX buffer, this will wait until there is one.
    ///
    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            if let Poll::Ready(envelope) = T::state().rx_queue.receive().poll_unpin(cx) {
                return Poll::Ready(Ok(envelope));
            } else if let Some(err) = self.curr_error() {
                return Poll::Ready(Err(err));
            }

            Poll::Pending
        })
        .await
    }

    /// Attempts to read a CAN frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        if let Ok(envelope) = T::state().rx_queue.try_receive() {
            return Ok(envelope);
        }

        if let Some(err) = self.curr_error() {
            return Err(TryReadError::BusError(err));
        }

        Err(TryReadError::Empty)
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        poll_fn(|cx| T::state().rx_queue.poll_ready_to_receive(cx)).await
    }

    fn curr_error(&self) -> Option<BusError> {
        let err = { T::regs().esr().read() };
        if err.boff() {
            return Some(BusError::BusOff);
        } else if err.epvf() {
            return Some(BusError::BusPassive);
        } else if err.ewgf() {
            return Some(BusError::BusWarning);
        } else if let Some(err) = err.lec().into_bus_err() {
            return Some(err);
        }
        None
    }
}

enum RxFifo {
    Fifo0,
    Fifo1,
}

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        T::regs().mcr().write(|w| w.set_reset(true));
        T::disable();
    }
}

impl<'d, T: Instance> Deref for Can<'d, T> {
    type Target = bxcan::Can<BxcanInstance<'d, T>>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance> DerefMut for Can<'d, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

pub(crate) mod sealed {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::channel::Channel;
    use embassy_sync::waitqueue::AtomicWaker;

    use super::Envelope;

    pub struct State {
        pub tx_waker: AtomicWaker,
        pub err_waker: AtomicWaker,
        pub rx_queue: Channel<CriticalSectionRawMutex, Envelope, 32>,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                tx_waker: AtomicWaker::new(),
                err_waker: AtomicWaker::new(),
                rx_queue: Channel::new(),
            }
        }
    }

    pub trait Instance {
        const REGISTERS: *mut bxcan::RegisterBlock;

        fn regs() -> crate::pac::can::Can;
        fn state() -> &'static State;
    }
}

/// CAN instance trait.
pub trait Instance: sealed::Instance + RccPeripheral + 'static {
    /// TX interrupt for this instance.
    type TXInterrupt: crate::interrupt::typelevel::Interrupt;
    /// RX0 interrupt for this instance.
    type RX0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// RX1 interrupt for this instance.
    type RX1Interrupt: crate::interrupt::typelevel::Interrupt;
    /// SCE interrupt for this instance.
    type SCEInterrupt: crate::interrupt::typelevel::Interrupt;
}

/// BXCAN instance newtype.
pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> bxcan::Instance for BxcanInstance<'d, T> {
    const REGISTERS: *mut bxcan::RegisterBlock = T::REGISTERS;
}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.as_ptr() as *mut _;

            fn regs() -> crate::pac::can::Can {
                crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type TXInterrupt = crate::_generated::peripheral_interrupts::$inst::TX;
            type RX0Interrupt = crate::_generated::peripheral_interrupts::$inst::RX0;
            type RX1Interrupt = crate::_generated::peripheral_interrupts::$inst::RX1;
            type SCEInterrupt = crate::_generated::peripheral_interrupts::$inst::SCE;
        }
    };
);

foreach_peripheral!(
    (can, CAN) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    // CAN1 and CAN2 is a combination of master and slave instance.
    // CAN1 owns the filter bank and needs to be enabled in order
    // for CAN2 to receive messages.
    (can, CAN1) => {
        cfg_if::cfg_if! {
            if #[cfg(all(
                any(stm32l4, stm32f72, stm32f73),
                not(any(stm32l49, stm32l4a))
            ))] {
                // Most L4 devices and some F7 devices use the name "CAN1"
                // even if there is no "CAN2" peripheral.
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl<'d> bxcan::MasterInstance for BxcanInstance<'d, peripherals::CAN1> {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN3> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

trait Index {
    fn index(&self) -> usize;
}

impl Index for bxcan::Mailbox {
    fn index(&self) -> usize {
        match self {
            bxcan::Mailbox::Mailbox0 => 0,
            bxcan::Mailbox::Mailbox1 => 1,
            bxcan::Mailbox::Mailbox2 => 2,
        }
    }
}

trait IntoBusError {
    fn into_bus_err(self) -> Option<BusError>;
}

impl IntoBusError for Lec {
    fn into_bus_err(self) -> Option<BusError> {
        match self {
            Lec::STUFF => Some(BusError::Stuff),
            Lec::FORM => Some(BusError::Form),
            Lec::ACK => Some(BusError::Acknowledge),
            Lec::BITRECESSIVE => Some(BusError::BitRecessive),
            Lec::BITDOMINANT => Some(BusError::BitDominant),
            Lec::CRC => Some(BusError::Crc),
            Lec::CUSTOM => Some(BusError::Software),
            _ => None,
        }
    }
}
