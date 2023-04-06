use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

pub use bxcan;
use bxcan::{Data, ExtendedId, Frame, Id, StandardId};
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::gpio::sealed::AFType;
use crate::interrupt::InterruptExt;
use crate::pac::can::vals::{Lec, RirIde};
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

pub struct Can<'d, T: Instance> {
    can: bxcan::Can<BxcanInstance<'d, T>>,
}

#[derive(Debug)]
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

pub enum FrameOrError {
    Frame(Frame),
    Error(BusError),
}

impl<'d, T: Instance> Can<'d, T> {
    /// Creates a new Bxcan instance, blocking for 11 recessive bits to sync with the CAN bus.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        T::enable();
        T::reset();

        Self {
            can: bxcan::Can::builder(BxcanInstance(peri)).enable(),
        }
    }

    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new_disabled(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_irq: impl Peripheral<P = T::TXInterrupt> + 'd,
        rx0_irq: impl Peripheral<P = T::RX0Interrupt> + 'd,
        rx1_irq: impl Peripheral<P = T::RX1Interrupt> + 'd,
        sce_irq: impl Peripheral<P = T::SCEInterrupt> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx, tx_irq, rx0_irq, rx1_irq, sce_irq);

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        T::enable();
        T::reset();

        tx_irq.unpend();
        tx_irq.set_handler(Self::tx_interrupt);
        tx_irq.enable();

        rx0_irq.unpend();
        rx0_irq.set_handler(Self::rx0_interrupt);
        rx0_irq.enable();

        rx1_irq.unpend();
        rx1_irq.set_handler(Self::rx1_interrupt);
        rx1_irq.enable();

        sce_irq.unpend();
        sce_irq.set_handler(Self::sce_interrupt);
        sce_irq.enable();

        let can = bxcan::Can::builder(BxcanInstance(peri)).leave_disabled();
        Self { can }
    }

    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = Self::calc_bxcan_timings(T::frequency(), bitrate).unwrap();
        self.can.modify_config().set_bit_timing(bit_timing).leave_disabled();
    }

    pub async fn transmit(&mut self, frame: &Frame) {
        let tx_status = self.queue_transmit(frame).await;
        self.wait_transission(tx_status.mailbox()).await;
    }

    async fn queue_transmit(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        poll_fn(|cx| {
            if let Ok(status) = self.can.transmit(frame) {
                return Poll::Ready(status);
            }
            T::state().tx_waker.register(cx.waker());
            Poll::Pending
        })
        .await
    }

    async fn wait_transission(&self, mb: bxcan::Mailbox) {
        poll_fn(|cx| unsafe {
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }
            T::state().tx_waker.register(cx.waker());
            Poll::Pending
        })
        .await;
    }

    pub async fn receive_frame_or_error(&mut self) -> FrameOrError {
        poll_fn(|cx| {
            if let Some(frame) = T::state().rx_queue.dequeue() {
                return Poll::Ready(FrameOrError::Frame(frame));
            } else if let Some(err) = self.curr_error() {
                return Poll::Ready(FrameOrError::Error(err));
            }
            T::state().rx_waker.register(cx.waker());
            T::state().err_waker.register(cx.waker());
            Poll::Pending
        })
        .await
    }

    fn curr_error(&self) -> Option<BusError> {
        let err = unsafe { T::regs().esr().read() };
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

    unsafe fn sce_interrupt(_: *mut ()) {
        let msr = T::regs().msr();
        let msr_val = msr.read();

        if msr_val.erri() {
            msr.modify(|v| v.set_erri(true));
            T::state().err_waker.wake();
            return;
        }
    }

    unsafe fn tx_interrupt(_: *mut ()) {
        T::regs().tsr().write(|v| {
            v.set_rqcp(0, true);
            v.set_rqcp(1, true);
            v.set_rqcp(2, true);
        });
        T::state().tx_waker.wake();
    }

    unsafe fn rx0_interrupt(_: *mut ()) {
        Self::receive_fifo(RxFifo::Fifo0);
    }

    unsafe fn rx1_interrupt(_: *mut ()) {
        Self::receive_fifo(RxFifo::Fifi1);
    }

    unsafe fn receive_fifo(fifo: RxFifo) {
        let state = T::state();
        let regs = T::regs();
        let fifo_idx = match fifo {
            RxFifo::Fifo0 => 0usize,
            RxFifo::Fifi1 => 1usize,
        };
        let rfr = regs.rfr(fifo_idx);
        let fifo = regs.rx(fifo_idx);

        // If there are no pending messages, there is nothing to do
        if rfr.read().fmp() == 0 {
            return;
        }

        let rir = fifo.rir().read();
        let id = if rir.ide() == RirIde::STANDARD {
            Id::from(StandardId::new_unchecked(rir.stid()))
        } else {
            Id::from(ExtendedId::new_unchecked(rir.exid()))
        };
        let data_len = fifo.rdtr().read().dlc() as usize;
        let mut data: [u8; 8] = [0; 8];
        data[0..4].copy_from_slice(&fifo.rdlr().read().0.to_ne_bytes());
        data[4..8].copy_from_slice(&fifo.rdhr().read().0.to_ne_bytes());

        let frame = Frame::new_data(id, Data::new(&data[0..data_len]).unwrap());

        rfr.modify(|v| v.set_rfom(true));

        match state.rx_queue.enqueue(frame) {
            Ok(_) => {}
            Err(_) => defmt::error!("RX queue overflow"),
        }
        state.rx_waker.wake();
    }

    pub fn calc_bxcan_timings(periph_clock: Hertz, can_bitrate: u32) -> Option<u32> {
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
        assert!(bs1_bs2_sum > bs1);

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
        Some((sjw - 1) << 24 | (bs1 as u32 - 1) << 16 | (bs2 as u32 - 1) << 20 | (prescaler as u32 - 1))
    }
}

enum RxFifo {
    Fifo0,
    Fifi1,
}

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        unsafe { T::regs().mcr().write(|w| w.set_reset(true)) }
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
    use embassy_sync::waitqueue::AtomicWaker;
    use heapless::mpmc::Q8;

    pub struct State {
        pub tx_waker: AtomicWaker,
        pub rx_waker: AtomicWaker,
        pub err_waker: AtomicWaker,
        pub rx_queue: Q8<bxcan::Frame>,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                tx_waker: AtomicWaker::new(),
                rx_waker: AtomicWaker::new(),
                err_waker: AtomicWaker::new(),
                rx_queue: Q8::new(),
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
    type TXInterrupt: crate::interrupt::Interrupt;
}

pub trait RX0Instance {
    type RX0Interrupt: crate::interrupt::Interrupt;
}

pub trait RX1Instance {
    type RX1Interrupt: crate::interrupt::Interrupt;
}

pub trait SCEInstance {
    type SCEInterrupt: crate::interrupt::Interrupt;
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
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.0 as *mut _;

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
                    type TXInterrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,RX0,$irq:ident) => {
                impl RX0Instance for peripherals::$inst {
                    type RX0Interrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,RX1,$irq:ident) => {
                impl RX1Instance for peripherals::$inst {
                    type RX1Interrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,SCE,$irq:ident) => {
                impl SCEInstance for peripherals::$inst {
                    type SCEInterrupt = crate::interrupt::$irq;
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
