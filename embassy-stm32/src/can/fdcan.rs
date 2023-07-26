use core::cell::{RefCell, RefMut};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

pub use fdcan;
use fdcan::frame::{RxFrameInfo, TxFrameHeader};
use fdcan::id::{StandardId, ExtendedId, Id};
use embassy_hal_common::{into_ref, PeripheralRef};
use fdcan::message_ram::RegisterBlock;
use fdcan::{BusMonitoringMode, InternalLoopbackMode, LastErrorCode, RestrictedOperationMode};
use futures::FutureExt;

use crate::gpio::sealed::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::{interrupt, peripherals, Peripheral};

// as far as I can tell, embedded-hal/can doesn't have any fdcan frame support
pub struct RxFrame {
    header: RxFrameInfo,
    data: Data,
}

pub struct TxFrame {
    header: TxFrameHeader,
    data: Data,
}

impl TxFrame {
    pub fn new(header: TxFrameHeader, data: &[u8]) -> Option<Self> {
        let Some(data) = Data::new(data) else {
            return None
        };

        Some(TxFrame {
            header,
            data,
        })
    }

    fn from_preserved(header: TxFrameHeader, data32: &[u32]) -> Option<Self> {
        //let data = unsafe { core::mem::transmute(*data32) };
        let mut data = [0u8; 64];

        for i in 0..data32.len() {
            data[4*i..][..4].copy_from_slice(&data32[i].to_le_bytes());
        }

        let Some(data) = Data::new(&data) else {
            return None
        };

        Some(TxFrame {
            header,
            data,
        })
    }
}

impl RxFrame {
    pub(crate) fn new(header: RxFrameInfo, data: &[u8]) -> Self {
        let data = Data::new(&data).unwrap_or_else(|| Data::empty());

        RxFrame {
            header,
            data,
        }
    }
}

/// Payload of a (FD)CAN data frame.
///
/// Contains 0 to 64 Bytes of data.
#[derive(Debug, Copy, Clone)]
pub struct Data {
    pub(crate) len: u8,
    pub(crate) bytes: [u8; 64],
}

impl Data {
    /// Creates a data payload from a raw byte slice.
    ///
    /// Returns `None` if `data` is more than 64 bytes (which is the maximum) or
    /// cannot be represented with an FDCAN DLC.
    pub fn new(data: &[u8]) -> Option<Self> {
        if !Data::is_valid_len(data.len()) {
            return None;
        }

        let mut bytes = [0; 64];
        bytes[..data.len()].copy_from_slice(data);

        Some(Self {
            len: data.len() as u8,
            bytes,
        })
    }

    pub fn get(&self) -> &[u8] {
        &self.bytes[..(self.len as usize)]
    }

/*
    /// Gets the encoded data length of the frame for FDCAN DLC field
    pub fn get_dlc(&self) -> u8 {
        match self.bytes.len() {
            0..=8 => self.bytes.len(),
            12 => 9,
            16 => 10,
            20 => 11,
            24 => 12,
            32 => 13,
            48 => 14,
            64 => 15,
            _ => panic!("length cannot be represented with fdcan dlc")
        }
    }
*/
    /// Checks if the length can be encoded in FDCAN DLC field.
    pub const fn is_valid_len(len: usize) -> bool {
        match len {
            0..=8 => true,
            12 => true,
            16 => true,
            20 => true,
            24 => true,
            32 => true,
            48 => true,
            64 => true,
            _ => false
        }
    }

    /// Creates an empty data payload containing 0 bytes.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            len: 0,
            bytes: [0; 64],
        }
    }
}

/// Interrupt handler.
pub struct IT0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

// We use IT0 for TX events and IT1 for RX events.
impl<T: Instance> interrupt::typelevel::Handler<T::IT0Interrupt> for IT0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().ir().read().tc() {
            T::regs().ir().write(|w| w.set_tc(true));
            T::state().tx_waker.wake();
        }
    }
}

pub struct IT1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::IT1Interrupt> for IT1InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().ir().read().rfn(0) {
            T::regs().ir().write(|w| w.set_rfn(0, true));
            T::state().rx_waker.wake();
            //Fdcan::<T>::receive_fifo(RxFifo::Fifo0);
        }
        if T::regs().ir().read().rfn(1) {
            T::regs().ir().write(|w| w.set_rfn(1, true));
            T::state().rx_waker.wake();
            //Fdcan::<T>::receive_fifo(RxFifo::Fifo1);
        }
    }
}

/*  TODO: handle error
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
 */


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

impl BusError {
    fn try_from(lec: LastErrorCode) -> Option<BusError> {
        match lec {
            LastErrorCode::AckError => Some(BusError::Acknowledge),
            LastErrorCode::Bit0Error => Some(BusError::BitRecessive), // TODO: verify
            LastErrorCode::Bit1Error => Some(BusError::BitDominant), // TODO: verify
            LastErrorCode::CRCError => Some(BusError::Crc),
            LastErrorCode::FormError => Some(BusError::Form),
            LastErrorCode::StuffError => Some(BusError::Stuff),
            _ => None
        }
    }
}

pub trait FdcanOperatingMode {}
impl FdcanOperatingMode for fdcan::PoweredDownMode {}
impl FdcanOperatingMode for fdcan::ConfigMode {}
impl FdcanOperatingMode for fdcan::InternalLoopbackMode {}
impl FdcanOperatingMode for fdcan::ExternalLoopbackMode {}
impl FdcanOperatingMode for fdcan::NormalOperationMode {}
impl FdcanOperatingMode for fdcan::RestrictedOperationMode {}
impl FdcanOperatingMode for fdcan::BusMonitoringMode {}
impl FdcanOperatingMode for fdcan::TestMode {}

/*
pub enum FdcanInstanceMode<T: fdcan::Instance> {
    PoweredDownMode(fdcan::FdCan<T, fdcan::PoweredDownMode>),
    ConfigMode(fdcan::FdCan<T, fdcan::ConfigMode>),
    InternalLoopbackMode(fdcan::FdCan<T, fdcan::InternalLoopbackMode>),
    ExternalLoopbackMode(fdcan::FdCan<T, fdcan::ExternalLoopbackMode>),
    NormalOperationMode(fdcan::FdCan<T, fdcan::NormalOperationMode>),
    RestrictedOperationMode(fdcan::FdCan<T, fdcan::RestrictedOperationMode>),
    BusMonitoringMode(fdcan::FdCan<T, fdcan::BusMonitoringMode>),
    TestMode(fdcan::FdCan<T, fdcan::TestMode>),
}
 */

pub struct Fdcan<'d, T: Instance, M: FdcanOperatingMode> {
    pub can: RefCell<fdcan::FdCan<FdcanInstance<'d, T>, M>>
}

impl<'d, T: Instance> Fdcan<'d, T, fdcan::PoweredDownMode> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new<M>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
        + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
        + 'd,
    ) -> Fdcan<'d, T, fdcan::PoweredDownMode> {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable();
        T::reset();

        {
            T::regs().ie().write(|w| {
                // TODO: handle errors
                w.set_epe(true);
                w.set_ewe(true);
                w.set_boe(true);


                w.set_rfne(0, true);
                w.set_rfne(1, true);

                // TODO: trigger off of TEFW?
                //w.set_tmeie(true);
            });

            T::regs().tscc().write(|w| {
                // Enable timestamps on rx messages

                w.set_tss(0b01);
            });
        }

        unsafe {
            T::configure_msg_ram();

            T::IT0Interrupt::unpend();
            T::IT0Interrupt::enable();

            T::IT1Interrupt::unpend();
            T::IT1Interrupt::enable();
        }

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        //let can = fdcan::FdCan::builder(FdcanInstance(peri)).leave_disabled();
        let can = fdcan::FdCan::new(FdcanInstance(peri));
        let can_ref_cell = RefCell::new(can);
        Self { can: can_ref_cell }
    }
}

impl<'d, T: Instance, M: FdcanOperatingMode> Fdcan<'d, T, M> {

/*
    //pub(crate) fn as_tx(&mut self) -> Option<RefCell<FdcanInstanceMode<FdcanInstance<'d, T>>>> {
    pub(crate) fn as_tx(&mut self) -> fdcan::FdCan<FdcanInstance<'d, T>, M> {
        match *self.can.borrow() {
            FdcanInstanceMode::NormalOperationMode(x) => x,
            _ => panic!("test")
        }
    }

    pub(crate) fn as_rx(&mut self) -> Option<RefCell<FdcanInstanceMode<FdcanInstance<'d, T>>>>  {
        match *self.can.borrow() {
            NormalOperationMode => Some(self.can),
            _ => None
        }
    }
 */
}

/*
impl<'d, T: Instance> Fdcan<'d, T, ConfigMode> {
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = Self::calc_fdcan_timings(T::frequency(), bitrate).unwrap();
        self.can
            .borrow_mut()
            .modify_config()
            .set_bit_timing(bit_timing)
            .leave_disabled();
    }
}
*/

impl<'d, T: Instance, M: FdcanOperatingMode> Fdcan<'d, T, M>
    where
        M: fdcan::Transmit,
        M: fdcan::Receive
{
    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.
    pub async fn write(&mut self, frame: &TxFrame) -> Option<TxFrame> {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if let Ok(dropped) = self.can.borrow_mut().transmit_preserve(frame.header, &frame.data.bytes, &mut |_,hdr,data32| TxFrame::from_preserved(hdr, data32)) {
                return Poll::Ready(dropped.flatten());
            }
            Poll::Pending
        })
        .await
    }

    /*
    // can't implement this right now because metapac's TRP definition is wrong
    pub async fn flush(&self, mb: fdcan::Mailbox) {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            let idx: u8 = mb.into();
            let idx: u32 = 1u32 << (idx as u32);
            if !(T::regs().txbrp().read().trp(idx) != 0) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }
*/

    /// Returns a tuple of the time the message was received and the message frame
    pub async fn read(&mut self) -> Result<RxFrame, BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            // TODO: handle fifo0 AND fifo1
            let mut buffer: [u8; 64] = [0; 64];
            if let Ok(rx) = self.can.borrow_mut().receive0(&mut buffer) {
                // rx: fdcan::ReceiveOverrun<RxFrameInfo>
                // TODO: report overrun?
                //  for now we just drop it
                let frame: RxFrame = RxFrame::new(rx.unwrap(), &buffer);
                return Poll::Ready(Ok(frame));
            } else if let Some(err) = self.curr_error() {  // TODO: this is probably wrong
                return Poll::Ready(Err(err));
            }

            // TODO: how to store buffers ?
/*
            if let Poll::Ready(frame) = T::state().rx_queue.recv().poll_unpin(cx) {
                return Poll::Ready(Ok(frame));
            } else if let Some(err) = self.curr_error() {
                return Poll::Ready(Err(err));
            }
*/
            Poll::Pending
        })
        .await
    }

    fn curr_error(&self) -> Option<BusError> {
        let err = { T::regs().psr().read() };
        if err.bo() {
            return Some(BusError::BusOff);
        } else if err.ep() {
            return Some(BusError::BusPassive);
        } else if err.ew() {
            return Some(BusError::BusWarning);
        } else if let Ok(err) = LastErrorCode::try_from(err.lec()) {
            return BusError::try_from(err);
        }
        None
    }
/*
    unsafe fn receive_fifo(fifo: RxFifo) {
        let state = T::state();
        let regs = T::regs();
        let fifo_idx = match fifo {
            RxFifo::Fifo0 => 0usize,
            RxFifo::Fifo1 => 1usize,
        };
        let rxfs = regs.rxfs(fifo_idx);
        //let fifo = regs.rx(fifo_idx);

        // If there are no pending messages, there is nothing to do
        if rxfs.read().ffl() == 0 {
            return;
        }

        state.rx_waker.wake();

            let id = if rir.ide() == RirIde::STANDARD {
                Id::from(StandardId::new_unchecked(rir.stid()))
            } else {
                let stid = (rir.stid() & 0x7FF) as u32;
                let exid = rir.exid() & 0x3FFFF;
                let id = (stid << 18) | (exid as u32);
                Id::from(ExtendedId::new_unchecked(id))
            };
            let data_len = fifo.rdtr().read().dlc() as usize;
            let mut data: [u8; 8] = [0; 8];
            data[0..4].copy_from_slice(&fifo.rdlr().read().0.to_ne_bytes());
            data[4..8].copy_from_slice(&fifo.rdhr().read().0.to_ne_bytes());

            let time = fifo.rdtr().read().time();
            let frame = Frame::new_data(id, Data::new(&data[0..data_len]).unwrap());

            rfr.modify(|v| v.set_rfom(true));

            T::regs().


                NOTE: consensus was reached that if rx_queue is full, packets should be dropped
            //let _ = state.rx_queue.try_send((time, frame));
    }

 */

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
        Some((sjw - 1) << 24 | (bs1 as u32 - 1) << 16 | (bs2 as u32 - 1) << 20 | (prescaler as u32 - 1))
    }

    pub fn split<'c>(&'c self) -> (FdcanTx<'c, 'd, T, M>, FdcanRx<'c, 'd, T, M>) {
        (FdcanTx { can: &self.can }, FdcanRx { can: &self.can })
    }

    pub fn as_mut(&self) -> RefMut<'_, fdcan::FdCan<FdcanInstance<'d, T>, M>> {
        self.can.borrow_mut()
    }
}

pub struct FdcanTx<'c, 'd, T: Instance, M: fdcan::Transmit> {
    can: &'c RefCell<fdcan::FdCan<FdcanInstance<'d, T>, M>>,
}

impl<'c, 'd, T: Instance, M: fdcan::Transmit> FdcanTx<'c, 'd, T, M> {
    pub async fn write(&mut self, frame: &TxFrame) -> Option<TxFrame> {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if let Ok(dropped) = self.can.borrow_mut().transmit_preserve(frame.header, &frame.data.bytes, &mut |_,hdr,data| TxFrame::from_preserved(hdr, data)) {
                return Poll::Ready(dropped.flatten());
            }
            Poll::Pending
        })
        .await
    }

    /*
    pub async fn flush(&self, mb: bxcan::Mailbox) {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }*/
}

#[allow(dead_code)]
pub struct FdcanRx<'c, 'd, T: Instance, M: fdcan::Receive> {
    can: &'c RefCell<fdcan::FdCan<FdcanInstance<'d, T>, M>>,
}

impl<'c, 'd, T: Instance, M: fdcan::Receive> FdcanRx<'c, 'd, T, M> {
    // todo: impl same as unsplit
    /*
    pub async fn read(&mut self) -> Result<RxFrame, BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            /*
            if let Poll::Ready(frame) = T::state().rx_queue.recv().poll_unpin(cx) {
                return Poll::Ready(Ok(frame));
            } else if let Some(err) = self.curr_error() {
                return Poll::Ready(Err(err));
            }
            */

            Poll::Pending
        })
        .await
    }
     */

    fn curr_error(&self) -> Option<BusError> {
        let err = { T::regs().psr().read() };
        if err.bo() {
            return Some(BusError::BusOff);
        } else if err.ep() {
            return Some(BusError::BusPassive);
        } else if err.ew() {
            return Some(BusError::BusWarning);
        } else if let Ok(err) = LastErrorCode::try_from(err.lec()) {
            return BusError::try_from(err);
        }
        None
    }
}

enum RxFifo {
    Fifo0,
    Fifo1,
}

impl<'d, T: Instance, M: FdcanOperatingMode> Drop for Fdcan<'d, T, M> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        T::regs().cccr().write(|w| w.set_init(true));
        T::disable();
    }
}

impl<'d, T: Instance, M: FdcanOperatingMode> Deref for Fdcan<'d, T, M> {
    type Target = RefCell<fdcan::FdCan<FdcanInstance<'d, T>, M>>;
    //type Target = RefCell<fdcan::FdCan<FdcanInstance<'d, T>>>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance, M: FdcanOperatingMode> DerefMut for Fdcan<'d, T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

pub(crate) mod sealed {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        pub tx_waker: AtomicWaker,
        pub err_waker: AtomicWaker,
        pub rx_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                tx_waker: AtomicWaker::new(),
                err_waker: AtomicWaker::new(),
                rx_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        const REGISTERS: *mut fdcan::RegisterBlock;
        const MSG_RAM: *mut fdcan::message_ram::RegisterBlock;

        fn regs() -> &'static crate::pac::can::Fdcan;
        fn state() -> &'static State;
        unsafe fn configure_msg_ram();
        /*
        fn msg_ram(&self) -> &RegisterBlock;
        fn msg_ram_mut(&mut self) -> &mut RegisterBlock;
         */
    }
}

pub trait IT0Instance {
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait IT1Instance {
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub trait InterruptableInstance: IT0Instance + IT1Instance {}
pub trait Instance: sealed::Instance + RccPeripheral + InterruptableInstance + 'static {}

pub struct FdcanInstance<'a, T>(PeripheralRef<'a, T>);

/*
unsafe impl<'d, T: Instance> fdcan::message_ram::Instance for FdcanInstance<'d, T> {
    const MSG_RAM: *mut RegisterBlock = T::MSG_RAM;
}
*/
/*
unsafe impl<'d, T: Instance> fdcan::message_ram::Instance for FdcanInstance<'d, T> where T: fdcan::message_ram::Instance {
    const MSG_RAM: *mut fdcan::message_ram::RegisterBlock = ;
}

unsafe impl<'d, T> fdcan::message_ram::Instance for FdcanInstance<'d, T> {
    const MSG_RAM: *mut RegisterBlock = (); // TODO ?
}


 */

unsafe impl<'d, T: Instance> fdcan::message_ram::Instance for FdcanInstance<'d, T> {
    const MSG_RAM: *mut RegisterBlock = T::MSG_RAM;
}

unsafe impl<'d, T: Instance> fdcan::Instance for FdcanInstance<'d, T> where FdcanInstance<'d, T>: fdcan::message_ram::Instance {
    const REGISTERS: *mut fdcan::RegisterBlock = T::REGISTERS;
}

// This macro taken from stm32h7xx-hal and adapted here
/// Configure Message RAM layout on H7 to match the fixed sized used on G4
///
/// These are protected bits, write access is only possible when bit CCE and bit
/// INIT for FDCAN_CCCR are set to 1


macro_rules! impl_fdcan {
    ($inst:ident, $msg_ram_addr:literal, $msg_ram_offset:literal) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut fdcan::RegisterBlock = crate::pac::$inst.as_ptr() as *mut _;
            const MSG_RAM: *mut fdcan::message_ram::RegisterBlock = (($msg_ram_addr+$msg_ram_offset) as *mut _);

            fn regs() -> &'static crate::pac::can::Fdcan {
                &crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }

            unsafe fn configure_msg_ram() {
                let regs = Self::REGISTERS;

                use fdcan::message_ram::*;
                let mut word_adr: u16 = $msg_ram_offset;

                // 11-bit filter

                (*regs).sidfc
                    .modify(|_, w| unsafe { w.flssa().bits(word_adr) });
                word_adr += STANDARD_FILTER_MAX as u16;
                // 29-bit filter
                (*regs).xidfc
                    .modify(|_, w| unsafe { w.flesa().bits(word_adr) });
                word_adr += 2 * EXTENDED_FILTER_MAX as u16;
                // Rx FIFO 0
                (*regs).rxf0c.modify(|_, w| unsafe {
                    w.f0sa()
                        .bits(word_adr)
                        .f0s()
                        .bits(RX_FIFO_MAX)
                        .f0wm()
                        .bits(RX_FIFO_MAX)
                });
                word_adr += 18 * RX_FIFO_MAX as u16;
                // Rx FIFO 1
                (*regs).rxf1c.modify(|_, w| unsafe {
                    w.f1sa()
                        .bits(word_adr)
                        .f1s()
                        .bits(RX_FIFO_MAX)
                        .f1wm()
                        .bits(RX_FIFO_MAX)
                });
                word_adr += 18 * RX_FIFO_MAX as u16;
                // Rx buffer - see below
                // Tx event FIFO
                (*regs).txefc.modify(|_, w| unsafe {
                    w.efsa()
                        .bits(word_adr)
                        .efs()
                        .bits(TX_EVENT_MAX)
                        .efwm()
                        .bits(TX_EVENT_MAX)
                });
                word_adr += 2 * TX_EVENT_MAX as u16;
                // Tx buffers
                (*regs).txbc.modify(|_, w| unsafe {
                    w.tbsa().bits(word_adr).tfqs().bits(TX_FIFO_MAX)
                });
                word_adr += 18 * TX_FIFO_MAX as u16;

                // Rx Buffer - not used
                (*regs).rxbc.modify(|_, w| unsafe { w.rbsa().bits(word_adr) });

                // TX event FIFO?
                // Trigger memory?

                // Set the element sizes to 16 bytes
                (*regs).rxesc.modify(|_, w| unsafe {
                    w.rbds().bits(0b111).f1ds().bits(0b111).f0ds().bits(0b111)
                });
                (*regs).txesc.modify(|_, w| unsafe { w.tbds().bits(0b111) });

                //message_ram_layout!(regs, $msg_ram_offset);
            }
        }

        impl Instance for peripherals::$inst {}

        foreach_interrupt!(
            ($inst,can,FDCAN,IT0,$irq:ident) => {
                impl IT0Instance for peripherals::$inst {
                    type IT0Interrupt = crate::interrupt::typelevel::$irq;
                }
            };
            ($inst,can,FDCAN,IT1,$irq:ident) => {
                impl IT1Instance for peripherals::$inst {
                    type IT1Interrupt = crate::interrupt::typelevel::$irq;
                }
            };
        );

        impl InterruptableInstance for peripherals::$inst {}
    }
}

foreach_peripheral!(
    (can, FDCAN) => { impl_fdcan!(FDCAN, 0x4000_ac00, 0x0000); };
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, 0x4000_ac00, 0x0000); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, 0x4000_ac00, 0x1000); };
);

foreach_peripheral!(
    (can, FDCAN) => {
        unsafe impl<'d> fdcan::FilterOwner for FdcanInstance<'d, peripherals::FDCAN> {
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
        unsafe impl<'d> fdcan::FilterOwner for FdcanInstance<'d, peripherals::CAN3> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);


// hack stolen from similar situation in DAC
// but then disabled because the metapac lists the registers under FDCAN1 so this causes a
// conflict.
// H7 uses single bit for both FDCAN1 and FDCAN2, this is a hack until a proper fix is implemented

#[cfg(rcc_h7)]
impl crate::rcc::sealed::RccPeripheral for peripherals::FDCAN2 {
fn frequency() -> crate::time::Hertz {
    critical_section::with(|_| unsafe { crate::rcc::get_freqs().apb1 })
}

fn reset() {
    critical_section::with(|_| {
        crate::pac::RCC.apb1hrstr().modify(|w| w.set_fdcanrst(true));
        crate::pac::RCC.apb1hrstr().modify(|w| w.set_fdcanrst(false));
    })
}

fn enable() {
    critical_section::with(|_| {
        crate::pac::RCC.apb1henr().modify(|w| w.set_fdcanen(true));
    })
}

fn disable() {
    critical_section::with(|_| {
        crate::pac::RCC.apb1henr().modify(|w| w.set_fdcanen(false))
    })
}
}

#[cfg(rcc_h7)]
impl crate::rcc::RccPeripheral for peripherals::FDCAN2 {}


pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

trait Index {
    fn index(&self) -> usize;
}

impl Index for fdcan::Mailbox {
    fn index(&self) -> usize {
        match self {
            fdcan::Mailbox::_0 => 0,
            fdcan::Mailbox::_1 => 1,
            fdcan::Mailbox::_2 => 2,
        }
    }
}