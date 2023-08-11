use core::cell::{RefCell, RefMut};
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
use crate::pac::can::vals::{Lec, RirIde};
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{interrupt, peripherals, Peripheral};

/// Contains CAN frame and additional metadata.
///
/// Timestamp is available if `time` feature is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Envelope {
    #[cfg(feature = "time")]
    pub ts: embassy_time::Instant,
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

pub struct Rx0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX0Interrupt> for Rx0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("rx0 irq");
        Can::<T>::receive_fifo(RxFifo::Fifo0);
    }
}

pub struct Rx1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RX1Interrupt> for Rx1InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // info!("rx1 irq");
        Can::<T>::receive_fifo(RxFifo::Fifo1);
    }
}

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

pub struct Can<'d, T: Instance> {
    pub can: RefCell<bxcan::Can<BxcanInstance<'d, T>>>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusError {
    Stuff,
    Form,
    Acknowledge,
    BitRecessive,
    BitDominant,
    Crc,
    Software,
    BusOff,
    BusPassive,
    BusWarning,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReadError {
    /// Bus error
    BusError(BusError),
    /// Receive buffer is empty
    Empty,
}

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

        T::enable();
        T::reset();

        {
            use crate::pac::can::vals::{Errie, Fmpie, Tmeie};

            T::regs().ier().write(|w| {
                // TODO: fix metapac

                w.set_errie(Errie::from_bits(1));
                w.set_fmpie(0, Fmpie::from_bits(1));
                w.set_fmpie(1, Fmpie::from_bits(1));
                w.set_tmeie(Tmeie::from_bits(1));
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
        let can_ref_cell = RefCell::new(can);
        Self { can: can_ref_cell }
    }

    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = Self::calc_bxcan_timings(T::frequency(), bitrate).unwrap();
        self.can
            .borrow_mut()
            .modify_config()
            .set_bit_timing(bit_timing)
            .leave_disabled();
    }

    /// Enables the peripheral and synchronizes with the bus.
    ///
    /// This will wait for 11 consecutive recessive bits (bus idle state).
    /// Contrary to enable method from bxcan library, this will not freeze the executor while waiting.
    pub async fn enable(&mut self) {
        while self.borrow_mut().enable_non_blocking().is_err() {
            // SCE interrupt is only generated for entering sleep mode, but not leaving.
            // Yield to allow other tasks to execute while can bus is initializing.
            embassy_futures::yield_now().await;
        }
    }

    /// Queues the message to be sent but exerts backpressure
    pub async fn write(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        CanTx { can: &self.can }.write(frame).await
    }

    /// Attempts to transmit a frame without blocking.
    ///
    /// Returns [Err(TryWriteError::Full)] if all transmit mailboxes are full.
    pub fn try_write(&mut self, frame: &Frame) -> Result<bxcan::TransmitStatus, TryWriteError> {
        CanTx { can: &self.can }.try_write(frame)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: bxcan::Mailbox) {
        CanTx { can: &self.can }.flush(mb).await
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
        CanTx { can: &self.can }.flush_any().await
    }

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
        CanTx { can: &self.can }.flush_all().await
    }

    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        CanRx { can: &self.can }.read().await
    }

    /// Attempts to read a can frame without blocking.
    ///
    /// Returns [Err(TryReadError::Empty)] if there are no frames in the rx queue.
    pub fn try_read(&mut self) -> Result<Envelope, TryReadError> {
        CanRx { can: &self.can }.try_read()
    }

    /// Waits while receive queue is empty.
    pub async fn wait_not_empty(&mut self) {
        CanRx { can: &self.can }.wait_not_empty().await
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
            let id = if rir.ide() == RirIde::STANDARD {
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

    pub const fn calc_bxcan_timings(periph_clock: Hertz, can_bitrate: u32) -> Option<u32> {
        const BS1_MAX: u8 = 16;
        const BS2_MAX: u8 = 8;
        const MAX_SAMPLE_POINT_PERMILL: u16 = 900;

        let periph_clock = periph_clock.0;

        if can_bitrate < 1000 {
            return None;
        }

        // Ref. "Automatic Baudrate Detection in CANopen Networks", U. Koppe, MicroControl GmbH & Co. KG
        //      CAN in Automation, 2003
        //
        // According to the source, optimal quanta per bit are:
        //   Bitrate        Optimal Maximum
        //   1000 kbps      8       10
        //   500  kbps      16      17
        //   250  kbps      16      17
        //   125  kbps      16      17
        let max_quanta_per_bit: u8 = if can_bitrate >= 1_000_000 { 10 } else { 17 };

        // Computing (prescaler * BS):
        //   BITRATE = 1 / (PRESCALER * (1 / PCLK) * (1 + BS1 + BS2))       -- See the Reference Manual
        //   BITRATE = PCLK / (PRESCALER * (1 + BS1 + BS2))                 -- Simplified
        // let:
        //   BS = 1 + BS1 + BS2                                             -- Number of time quanta per bit
        //   PRESCALER_BS = PRESCALER * BS
        // ==>
        //   PRESCALER_BS = PCLK / BITRATE
        let prescaler_bs = periph_clock / can_bitrate;

        // Searching for such prescaler value so that the number of quanta per bit is highest.
        let mut bs1_bs2_sum = max_quanta_per_bit - 1;
        while (prescaler_bs % (1 + bs1_bs2_sum) as u32) != 0 {
            if bs1_bs2_sum <= 2 {
                return None; // No solution
            }
            bs1_bs2_sum -= 1;
        }

        let prescaler = prescaler_bs / (1 + bs1_bs2_sum) as u32;
        if (prescaler < 1) || (prescaler > 1024) {
            return None; // No solution
        }

        // Now we have a constraint: (BS1 + BS2) == bs1_bs2_sum.
        // We need to find such values so that the sample point is as close as possible to the optimal value,
        // which is 87.5%, which is 7/8.
        //
        //   Solve[(1 + bs1)/(1 + bs1 + bs2) == 7/8, bs2]  (* Where 7/8 is 0.875, the recommended sample point location *)
        //   {{bs2 -> (1 + bs1)/7}}
        //
        // Hence:
        //   bs2 = (1 + bs1) / 7
        //   bs1 = (7 * bs1_bs2_sum - 1) / 8
        //
        // Sample point location can be computed as follows:
        //   Sample point location = (1 + bs1) / (1 + bs1 + bs2)
        //
        // Since the optimal solution is so close to the maximum, we prepare two solutions, and then pick the best one:
        //   - With rounding to nearest
        //   - With rounding to zero
        let mut bs1 = ((7 * bs1_bs2_sum - 1) + 4) / 8; // Trying rounding to nearest first
        let mut bs2 = bs1_bs2_sum - bs1;
        core::assert!(bs1_bs2_sum > bs1);

        let sample_point_permill = 1000 * ((1 + bs1) / (1 + bs1 + bs2)) as u16;
        if sample_point_permill > MAX_SAMPLE_POINT_PERMILL {
            // Nope, too far; now rounding to zero
            bs1 = (7 * bs1_bs2_sum - 1) / 8;
            bs2 = bs1_bs2_sum - bs1;
        }

        // Check is BS1 and BS2 are in range
        if (bs1 < 1) || (bs1 > BS1_MAX) || (bs2 < 1) || (bs2 > BS2_MAX) {
            return None;
        }

        // Check if final bitrate matches the requested
        if can_bitrate != (periph_clock / (prescaler * (1 + bs1 + bs2) as u32)) {
            return None;
        }

        // One is recommended by DS-015, CANOpen, and DeviceNet
        let sjw = 1;

        // Pack into BTR register values
        Some((sjw - 1) << 24 | (bs1 as u32 - 1) << 16 | (bs2 as u32 - 1) << 20 | (prescaler - 1))
    }

    pub fn split<'c>(&'c self) -> (CanTx<'c, 'd, T>, CanRx<'c, 'd, T>) {
        (CanTx { can: &self.can }, CanRx { can: &self.can })
    }

    pub fn as_mut(&self) -> RefMut<'_, bxcan::Can<BxcanInstance<'d, T>>> {
        self.can.borrow_mut()
    }
}

pub struct CanTx<'c, 'd, T: Instance> {
    can: &'c RefCell<bxcan::Can<BxcanInstance<'d, T>>>,
}

impl<'c, 'd, T: Instance> CanTx<'c, 'd, T> {
    pub async fn write(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if let Ok(status) = self.can.borrow_mut().transmit(frame) {
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
        self.can.borrow_mut().transmit(frame).map_err(|_| TryWriteError::Full)
    }

    /// Waits for a specific transmit mailbox to become empty
    pub async fn flush(&self, mb: bxcan::Mailbox) {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Waits until any of the transmit mailboxes become empty
    pub async fn flush_any(&self) {
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

    /// Waits until all of the transmit mailboxes become empty
    pub async fn flush_all(&self) {
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
}

#[allow(dead_code)]
pub struct CanRx<'c, 'd, T: Instance> {
    can: &'c RefCell<bxcan::Can<BxcanInstance<'d, T>>>,
}

impl<'c, 'd, T: Instance> CanRx<'c, 'd, T> {
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
    type Target = RefCell<bxcan::Can<BxcanInstance<'d, T>>>;

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

        fn regs() -> &'static crate::pac::can::Can;
        fn state() -> &'static State;
    }
}

pub trait TXInstance {
    type TXInterrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait RX0Instance {
    type RX0Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait RX1Instance {
    type RX1Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait SCEInstance {
    type SCEInterrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait InterruptableInstance: TXInstance + RX0Instance + RX1Instance + SCEInstance {}
pub trait Instance: sealed::Instance + RccPeripheral + InterruptableInstance + 'static {}

pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> bxcan::Instance for BxcanInstance<'d, T> {
    const REGISTERS: *mut bxcan::RegisterBlock = T::REGISTERS;
}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.as_ptr() as *mut _;

            fn regs() -> &'static crate::pac::can::Can {
                &crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {}

        foreach_interrupt!(
            ($inst,can,CAN,TX,$irq:ident) => {
                impl TXInstance for peripherals::$inst {
                    type TXInterrupt = crate::interrupt::typelevel::$irq;
                }
            };
            ($inst,can,CAN,RX0,$irq:ident) => {
                impl RX0Instance for peripherals::$inst {
                    type RX0Interrupt = crate::interrupt::typelevel::$irq;
                }
            };
            ($inst,can,CAN,RX1,$irq:ident) => {
                impl RX1Instance for peripherals::$inst {
                    type RX1Interrupt = crate::interrupt::typelevel::$irq;
                }
            };
            ($inst,can,CAN,SCE,$irq:ident) => {
                impl SCEInstance for peripherals::$inst {
                    type SCEInterrupt = crate::interrupt::typelevel::$irq;
                }
            };
        );

        impl InterruptableInstance for peripherals::$inst {}
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
