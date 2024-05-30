#[allow(unused_variables)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender};
use embassy_sync::waitqueue::AtomicWaker;

use crate::can::fd::peripheral::Registers;
use crate::gpio::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub(crate) mod fd;

use self::fd::config::*;
use self::fd::filter::*;
pub use self::fd::{config, filter};
pub use super::common::{BufferedCanReceiver, BufferedCanSender};
use super::enums::*;
use super::frame::*;
use super::util;

/// Timestamp for incoming packets. Use Embassy time when enabled.
#[cfg(feature = "time")]
pub type Timestamp = embassy_time::Instant;

/// Timestamp for incoming packets.
#[cfg(not(feature = "time"))]
pub type Timestamp = u16;

/// Interrupt handler channel 0.
pub struct IT0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

// We use IT0 for everything currently
impl<T: Instance> interrupt::typelevel::Handler<T::IT0Interrupt> for IT0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        let ir = regs.ir().read();

        if ir.tc() {
            regs.ir().write(|w| w.set_tc(true));
        }
        if ir.tefn() {
            regs.ir().write(|w| w.set_tefn(true));
        }

        match &T::state().tx_mode {
            TxMode::NonBuffered(waker) => waker.wake(),
            TxMode::ClassicBuffered(buf) => {
                if !T::registers().tx_queue_is_full() {
                    match buf.tx_receiver.try_receive() {
                        Ok(frame) => {
                            _ = T::registers().write(&frame);
                        }
                        Err(_) => {}
                    }
                }
            }
            TxMode::FdBuffered(buf) => {
                if !T::registers().tx_queue_is_full() {
                    match buf.tx_receiver.try_receive() {
                        Ok(frame) => {
                            _ = T::registers().write(&frame);
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        if ir.rfn(0) {
            T::state().rx_mode.on_interrupt::<T>(0);
        }
        if ir.rfn(1) {
            T::state().rx_mode.on_interrupt::<T>(1);
        }

        if ir.bo() {
            regs.ir().write(|w| w.set_bo(true));
            if regs.psr().read().bo() {
                // Initiate bus-off recovery sequence by resetting CCCR.INIT
                regs.cccr().modify(|w| w.set_init(false));
            }
        }
    }
}

/// Interrupt handler channel 1.
pub struct IT1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::IT1Interrupt> for IT1InterruptHandler<T> {
    unsafe fn on_interrupt() {}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Different operating modes
pub enum OperatingMode {
    //PoweredDownMode,
    //ConfigMode,
    /// This mode can be used for a “Hot Selftest”, meaning the FDCAN can be tested without
    /// affecting a running CAN system connected to the FDCAN_TX and FDCAN_RX pins. In this
    /// mode, FDCAN_RX pin is disconnected from the FDCAN and FDCAN_TX pin is held
    /// recessive.
    InternalLoopbackMode,
    /// This mode is provided for hardware self-test. To be independent from external stimulation,
    /// the FDCAN ignores acknowledge errors (recessive bit sampled in the acknowledge slot of a
    /// data / remote frame) in Loop Back mode. In this mode the FDCAN performs an internal
    /// feedback from its transmit output to its receive input. The actual value of the FDCAN_RX
    /// input pin is disregarded by the FDCAN. The transmitted messages can be monitored at the
    /// FDCAN_TX transmit pin.
    ExternalLoopbackMode,
    /// The normal use of the Fdcan instance after configurations
    NormalOperationMode,
    /// In Restricted operation mode the node is able to receive data and remote frames and to give
    /// acknowledge to valid frames, but it does not send data frames, remote frames, active error
    /// frames, or overload frames. In case of an error condition or overload condition, it does not
    /// send dominant bits, instead it waits for the occurrence of bus idle condition to resynchronize
    /// itself to the CAN communication. The error counters for transmit and receive are frozen while
    /// error logging (can_errors) is active. TODO: automatically enter in this mode?
    RestrictedOperationMode,
    ///  In Bus monitoring mode (for more details refer to ISO11898-1, 10.12 Bus monitoring),
    /// the FDCAN is able to receive valid data frames and valid remote frames, but cannot start a
    /// transmission. In this mode, it sends only recessive bits on the CAN bus. If the FDCAN is
    /// required to send a dominant bit (ACK bit, overload flag, active error flag), the bit is
    /// rerouted internally so that the FDCAN can monitor it, even if the CAN bus remains in recessive
    /// state. In Bus monitoring mode the TXBRP register is held in reset state. The Bus monitoring
    /// mode can be used to analyze the traffic on a CAN bus without affecting it by the transmission
    /// of dominant bits.
    BusMonitoringMode,
    //TestMode,
}

/// FDCAN Configuration instance instance
/// Create instance of this first
pub struct CanConfigurator<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    instance: FdcanInstance<'d, T>,
    properties: Properties<T>,
}

fn calc_ns_per_timer_tick<T: Instance>(mode: crate::can::fd::config::FrameTransmissionConfig) -> u64 {
    match mode {
        // Use timestamp from Rx FIFO to adjust timestamp reported to user
        crate::can::fd::config::FrameTransmissionConfig::ClassicCanOnly => {
            let freq = T::frequency();
            let prescale: u64 =
                ({ T::regs().nbtp().read().nbrp() } + 1) as u64 * ({ T::regs().tscc().read().tcp() } + 1) as u64;
            1_000_000_000 as u64 / (freq.0 as u64 * prescale)
        }
        // For VBR this is too hard because the FDCAN timer switches clock rate you need to configure to use
        // timer3 instead which is too hard to do from this module.
        _ => 0,
    }
}

impl<'d, T: Instance> CanConfigurator<'d, T> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
            + 'd,
    ) -> CanConfigurator<'d, T> {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable_and_reset();

        let mut config = crate::can::fd::config::FdCanConfig::default();
        config.timestamp_source = TimestampSource::Prescaler(TimestampPrescaler::_1);
        T::registers().into_config_mode(config);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        unsafe {
            T::IT0Interrupt::unpend(); // Not unsafe
            T::IT0Interrupt::enable();

            T::IT1Interrupt::unpend(); // Not unsafe
            T::IT1Interrupt::enable();
        }

        Self {
            config,
            instance: FdcanInstance(peri),
            properties: Properties::new(),
        }
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties<T> {
        &self.properties
    }

    /// Get configuration
    pub fn config(&self) -> crate::can::fd::config::FdCanConfig {
        return self.config;
    }

    /// Set configuration
    pub fn set_config(&mut self, config: crate::can::fd::config::FdCanConfig) {
        self.config = config;
    }

    /// Configures the bit timings calculated from supplied bitrate.
    pub fn set_bitrate(&mut self, bitrate: u32) {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();

        let nbtr = crate::can::fd::config::NominalBitTiming {
            sync_jump_width: bit_timing.sync_jump_width,
            prescaler: bit_timing.prescaler,
            seg1: bit_timing.seg1,
            seg2: bit_timing.seg2,
        };
        self.config = self.config.set_nominal_bit_timing(nbtr);
    }

    /// Configures the bit timings for VBR data calculated from supplied bitrate. This also sets confit to allow can FD and VBR
    pub fn set_fd_data_bitrate(&mut self, bitrate: u32, transceiver_delay_compensation: bool) {
        let bit_timing = util::calc_can_timings(T::frequency(), bitrate).unwrap();
        // Note, used existing calcluation for normal(non-VBR) bitrate, appears to work for 250k/1M
        let nbtr = crate::can::fd::config::DataBitTiming {
            transceiver_delay_compensation,
            sync_jump_width: bit_timing.sync_jump_width,
            prescaler: bit_timing.prescaler,
            seg1: bit_timing.seg1,
            seg2: bit_timing.seg2,
        };
        self.config.frame_transmit = FrameTransmissionConfig::AllowFdCanAndBRS;
        self.config = self.config.set_data_bit_timing(nbtr);
    }

    /// Start in mode.
    pub fn start(self, mode: OperatingMode) -> Can<'d, T> {
        let ns_per_timer_tick = calc_ns_per_timer_tick::<T>(self.config.frame_transmit);
        critical_section::with(|_| unsafe {
            T::mut_state().ns_per_timer_tick = ns_per_timer_tick;
        });
        T::registers().into_mode(self.config, mode);
        let ret = Can {
            config: self.config,
            instance: self.instance,
            _mode: mode,
            properties: self.properties,
        };
        ret
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_normal_mode(self) -> Can<'d, T> {
        self.start(OperatingMode::NormalOperationMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_internal_loopback_mode(self) -> Can<'d, T> {
        self.start(OperatingMode::InternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_external_loopback_mode(self) -> Can<'d, T> {
        self.start(OperatingMode::ExternalLoopbackMode)
    }
}

/// FDCAN Instance
pub struct Can<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    instance: FdcanInstance<'d, T>,
    _mode: OperatingMode,
    properties: Properties<T>,
}

impl<'d, T: Instance> Can<'d, T> {
    /// Get driver properties
    pub fn properties(&self) -> &Properties<T> {
        &self.properties
    }

    /// Flush one of the TX mailboxes.
    pub async fn flush(&self, idx: usize) {
        poll_fn(|cx| {
            T::state().tx_mode.register(cx.waker());

            if idx > 3 {
                panic!("Bad mailbox");
            }
            let idx = 1 << idx;
            if !T::regs().txbrp().read().trp(idx) {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &Frame) -> Option<Frame> {
        T::state().tx_mode.write::<T>(frame).await
    }

    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        T::state().rx_mode.read_classic::<T>().await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        T::state().tx_mode.write_fd::<T>(frame).await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<FdEnvelope, BusError> {
        T::state().rx_mode.read_fd::<T>().await
    }

    /// Split instance into separate portions: Tx(write), Rx(read), common properties
    pub fn split(self) -> (CanTx<'d, T>, CanRx<'d, T>, Properties<T>) {
        (
            CanTx {
                config: self.config,
                _instance: self.instance,
                _mode: self._mode,
            },
            CanRx {
                _instance1: PhantomData::<T>,
                _instance2: T::regs(),
                _mode: self._mode,
            },
            self.properties,
        )
    }

    /// Join split rx and tx portions back together
    pub fn join(tx: CanTx<'d, T>, rx: CanRx<'d, T>) -> Self {
        Can {
            config: tx.config,
            //_instance2: T::regs(),
            instance: tx._instance,
            _mode: rx._mode,
            properties: Properties::new(),
        }
    }

    /// Return a buffered instance of driver without CAN FD support. User must supply Buffers
    pub fn buffered<const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        &self,
        tx_buf: &'static mut TxBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE> {
        BufferedCan::new(PhantomData::<T>, T::regs(), self._mode, tx_buf, rxb)
    }

    /// Return a buffered instance of driver with CAN FD support. User must supply Buffers
    pub fn buffered_fd<const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        &self,
        tx_buf: &'static mut TxFdBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxFdBuf<RX_BUF_SIZE>,
    ) -> BufferedCanFd<'d, T, TX_BUF_SIZE, RX_BUF_SIZE> {
        BufferedCanFd::new(PhantomData::<T>, T::regs(), self._mode, tx_buf, rxb)
    }
}

/// User supplied buffer for RX Buffering
pub type RxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<Envelope, BusError>, BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Frame, BUF_SIZE>;

/// Buffered FDCAN Instance
pub struct BufferedCan<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: OperatingMode,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
    properties: Properties<T>,
}

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>
    BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn new(
        _instance1: PhantomData<T>,
        _instance2: &'d crate::pac::can::Fdcan,
        _mode: OperatingMode,
        tx_buf: &'static TxBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCan {
            _instance1,
            _instance2,
            _mode,
            tx_buf,
            rx_buf,
            properties: Properties::new(),
        }
        .setup()
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties<T> {
        &self.properties
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = super::common::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = super::common::ClassicBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            T::mut_state().rx_mode = RxMode::ClassicBuffered(rx_inner);
            T::mut_state().tx_mode = TxMode::ClassicBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: Frame) {
        self.tx_buf.send(frame).await;
        T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.rx_buf.receive().await
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        BufferedCanSender {
            tx_buf: self.tx_buf.sender().into(),
            waker: T::IT0Interrupt::pend,
        }
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedCanReceiver {
        self.rx_buf.receiver().into()
    }
}

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> Drop
    for BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn drop(&mut self) {
        critical_section::with(|_| unsafe {
            T::mut_state().rx_mode = RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            T::mut_state().tx_mode = TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

/// User supplied buffer for RX Buffering
pub type RxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<FdEnvelope, BusError>, BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, FdFrame, BUF_SIZE>;

/// Buffered FDCAN Instance
pub struct BufferedCanFd<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: OperatingMode,
    tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
    properties: Properties<T>,
}

/// Sender that can be used for sending CAN frames.
#[derive(Copy, Clone)]
pub struct BufferedFdCanSender {
    tx_buf: DynamicSender<'static, FdFrame>,
    waker: fn(),
}

impl BufferedFdCanSender {
    /// Async write frame to TX buffer.
    pub fn try_write(&mut self, frame: FdFrame) -> Result<(), embassy_sync::channel::TrySendError<FdFrame>> {
        self.tx_buf.try_send(frame)?;
        (self.waker)();
        Ok(())
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FdFrame) {
        self.tx_buf.send(frame).await;
        (self.waker)();
    }

    /// Allows a poll_fn to poll until the channel is ready to write
    pub fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.tx_buf.poll_ready_to_send(cx)
    }
}

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedFdCanReceiver = DynamicReceiver<'static, Result<FdEnvelope, BusError>>;

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>
    BufferedCanFd<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn new(
        _instance1: PhantomData<T>,
        _instance2: &'d crate::pac::can::Fdcan,
        _mode: OperatingMode,
        tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCanFd {
            _instance1,
            _instance2,
            _mode,
            tx_buf,
            rx_buf,
            properties: Properties::new(),
        }
        .setup()
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties<T> {
        &self.properties
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = super::common::FdBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = super::common::FdBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            T::mut_state().rx_mode = RxMode::FdBuffered(rx_inner);
            T::mut_state().tx_mode = TxMode::FdBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FdFrame) {
        self.tx_buf.send(frame).await;
        T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<FdEnvelope, BusError> {
        self.rx_buf.receive().await
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedFdCanSender {
        BufferedFdCanSender {
            tx_buf: self.tx_buf.sender().into(),
            waker: T::IT0Interrupt::pend,
        }
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedFdCanReceiver {
        self.rx_buf.receiver().into()
    }
}

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> Drop
    for BufferedCanFd<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn drop(&mut self) {
        critical_section::with(|_| unsafe {
            T::mut_state().rx_mode = RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            T::mut_state().tx_mode = TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

/// FDCAN Rx only Instance
pub struct CanRx<'d, T: Instance> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: OperatingMode,
}

/// FDCAN Tx only Instance
pub struct CanTx<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    _instance: FdcanInstance<'d, T>, //(PeripheralRef<'a, T>);
    _mode: OperatingMode,
}

impl<'c, 'd, T: Instance> CanTx<'d, T> {
    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &Frame) -> Option<Frame> {
        T::state().tx_mode.write::<T>(frame).await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        T::state().tx_mode.write_fd::<T>(frame).await
    }
}

impl<'c, 'd, T: Instance> CanRx<'d, T> {
    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        T::state().rx_mode.read_classic::<T>().await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<FdEnvelope, BusError> {
        T::state().rx_mode.read_fd::<T>().await
    }
}

enum RxMode {
    NonBuffered(AtomicWaker),
    ClassicBuffered(super::common::ClassicBufferedRxInner),
    FdBuffered(super::common::FdBufferedRxInner),
}

impl RxMode {
    fn register(&self, arg: &core::task::Waker) {
        match self {
            RxMode::NonBuffered(waker) => waker.register(arg),
            _ => {
                panic!("Bad Mode")
            }
        }
    }

    fn on_interrupt<T: Instance>(&self, fifonr: usize) {
        T::regs().ir().write(|w| w.set_rfn(fifonr, true));
        match self {
            RxMode::NonBuffered(waker) => {
                waker.wake();
            }
            RxMode::ClassicBuffered(buf) => {
                if let Some(result) = self.try_read::<T>() {
                    let _ = buf.rx_sender.try_send(result);
                }
            }
            RxMode::FdBuffered(buf) => {
                if let Some(result) = self.try_read_fd::<T>() {
                    let _ = buf.rx_sender.try_send(result);
                }
            }
        }
    }

    //async fn read_classic<T: Instance>(&self) -> Result<Envelope, BusError> {
    fn try_read<T: Instance>(&self) -> Option<Result<Envelope, BusError>> {
        if let Some((frame, ts)) = T::registers().read(0) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok(Envelope { ts, frame }))
        } else if let Some((frame, ts)) = T::registers().read(1) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok(Envelope { ts, frame }))
        } else if let Some(err) = T::registers().curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    //async fn read_classic<T: Instance>(&self) -> Result<Envelope, BusError> {
    fn try_read_fd<T: Instance>(&self) -> Option<Result<FdEnvelope, BusError>> {
        if let Some((frame, ts)) = T::registers().read(0) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok(FdEnvelope { ts, frame }))
        } else if let Some((frame, ts)) = T::registers().read(1) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok(FdEnvelope { ts, frame }))
        } else if let Some(err) = T::registers().curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    fn read<T: Instance, F: CanHeader>(&self) -> Option<Result<(F, Timestamp), BusError>> {
        if let Some((msg, ts)) = T::registers().read(0) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok((msg, ts)))
        } else if let Some((msg, ts)) = T::registers().read(1) {
            let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
            Some(Ok((msg, ts)))
        } else if let Some(err) = T::registers().curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    async fn read_async<T: Instance, F: CanHeader>(&self) -> Result<(F, Timestamp), BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            self.register(cx.waker());
            match self.read::<T, _>() {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        })
        .await
    }

    async fn read_classic<T: Instance>(&self) -> Result<Envelope, BusError> {
        match self.read_async::<T, _>().await {
            Ok((frame, ts)) => Ok(Envelope { ts, frame }),
            Err(e) => Err(e),
        }
    }

    async fn read_fd<T: Instance>(&self) -> Result<FdEnvelope, BusError> {
        match self.read_async::<T, _>().await {
            Ok((frame, ts)) => Ok(FdEnvelope { ts, frame }),
            Err(e) => Err(e),
        }
    }
}

enum TxMode {
    NonBuffered(AtomicWaker),
    ClassicBuffered(super::common::ClassicBufferedTxInner),
    FdBuffered(super::common::FdBufferedTxInner),
}

impl TxMode {
    fn register(&self, arg: &core::task::Waker) {
        match self {
            TxMode::NonBuffered(waker) => {
                waker.register(arg);
            }
            _ => {
                panic!("Bad mode");
            }
        }
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    async fn write_generic<T: Instance, F: embedded_can::Frame + CanHeader>(&self, frame: &F) -> Option<F> {
        poll_fn(|cx| {
            self.register(cx.waker());

            if let Ok(dropped) = T::registers().write(frame) {
                return Poll::Ready(dropped);
            }

            // Couldn't replace any lower priority frames.  Need to wait for some mailboxes
            // to clear.
            Poll::Pending
        })
        .await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    async fn write<T: Instance>(&self, frame: &Frame) -> Option<Frame> {
        self.write_generic::<T, _>(frame).await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    async fn write_fd<T: Instance>(&self, frame: &FdFrame) -> Option<FdFrame> {
        self.write_generic::<T, _>(frame).await
    }
}

/// Common driver properties, including filters and error counters
pub struct Properties<T> {
    // phantom pointer to ensure !Sync
    instance: PhantomData<*const T>,
}

impl<T: Instance> Properties<T> {
    fn new() -> Self {
        Self {
            instance: Default::default(),
        }
    }

    /// Set a standard address CAN filter in the specified slot in FDCAN memory.
    #[inline]
    pub fn set_standard_filter(&self, slot: StandardFilterSlot, filter: StandardFilter) {
        T::registers().msg_ram_mut().filters.flssa[slot as usize].activate(filter);
    }

    /// Set the full array of standard address CAN filters in FDCAN memory.
    /// Overwrites all standard address filters in memory.
    pub fn set_standard_filters(&self, filters: &[StandardFilter; STANDARD_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            T::registers().msg_ram_mut().filters.flssa[i].activate(*f);
        }
    }

    /// Set an extended address CAN filter in the specified slot in FDCAN memory.
    #[inline]
    pub fn set_extended_filter(&self, slot: ExtendedFilterSlot, filter: ExtendedFilter) {
        T::registers().msg_ram_mut().filters.flesa[slot as usize].activate(filter);
    }

    /// Set the full array of extended address CAN filters in FDCAN memory.
    /// Overwrites all extended address filters in memory.
    pub fn set_extended_filters(&self, filters: &[ExtendedFilter; EXTENDED_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            T::registers().msg_ram_mut().filters.flesa[i].activate(*f);
        }
    }

    /// Get the CAN RX error counter
    pub fn rx_error_count(&self) -> u8 {
        T::regs().ecr().read().rec()
    }

    /// Get the CAN TX error counter
    pub fn tx_error_count(&self) -> u8 {
        T::regs().ecr().read().tec()
    }

    /// Get the current bus error mode
    pub fn bus_error_mode(&self) -> BusErrorMode {
        // This read will clear LEC and DLEC. This is not ideal, but protocol
        // error reporting in this driver should have a big ol' FIXME on it
        // anyway!
        let psr = T::regs().psr().read();
        match (psr.bo(), psr.ep()) {
            (false, false) => BusErrorMode::ErrorActive,
            (false, true) => BusErrorMode::ErrorPassive,
            (true, _) => BusErrorMode::BusOff,
        }
    }
}

struct State {
    pub rx_mode: RxMode,
    pub tx_mode: TxMode,
    pub ns_per_timer_tick: u64,

    pub err_waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            rx_mode: RxMode::NonBuffered(AtomicWaker::new()),
            tx_mode: TxMode::NonBuffered(AtomicWaker::new()),
            ns_per_timer_tick: 0,
            err_waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    const MSG_RAM_OFFSET: usize;

    fn regs() -> &'static crate::pac::can::Fdcan;
    fn registers() -> crate::can::fd::peripheral::Registers;
    fn state() -> &'static State;
    unsafe fn mut_state() -> &'static mut State;
    fn calc_timestamp(ns_per_timer_tick: u64, ts_val: u16) -> Timestamp;
}

/// Instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// Interrupt 0
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// Interrupt 1
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Fdcan Instance struct
pub struct FdcanInstance<'a, T>(PeripheralRef<'a, T>);

macro_rules! impl_fdcan {
    ($inst:ident, $msg_ram_inst:ident, $msg_ram_offset:literal) => {
        impl SealedInstance for peripherals::$inst {
            const MSG_RAM_OFFSET: usize = $msg_ram_offset;

            fn regs() -> &'static crate::pac::can::Fdcan {
                &crate::pac::$inst
            }
            fn registers() -> Registers {
                Registers{regs: &crate::pac::$inst, msgram: &crate::pac::$msg_ram_inst, msg_ram_offset: Self::MSG_RAM_OFFSET}
            }
            unsafe fn mut_state() -> &'static mut State {
                static mut STATE: State = State::new();
                &mut *core::ptr::addr_of_mut!(STATE)
            }
            fn state() -> &'static State {
                unsafe { peripherals::$inst::mut_state() }
            }

            #[cfg(feature = "time")]
            fn calc_timestamp(ns_per_timer_tick: u64, ts_val: u16) -> Timestamp {
                let now_embassy = embassy_time::Instant::now();
                if ns_per_timer_tick == 0 {
                    return now_embassy;
                }
                let cantime = { Self::regs().tscv().read().tsc() };
                let delta = cantime.overflowing_sub(ts_val).0 as u64;
                let ns = ns_per_timer_tick * delta as u64;
                now_embassy - embassy_time::Duration::from_nanos(ns)
            }

            #[cfg(not(feature = "time"))]
            fn calc_timestamp(_ns_per_timer_tick: u64, ts_val: u16) -> Timestamp {
                ts_val
            }

        }

        #[allow(non_snake_case)]
        pub(crate) mod $inst {

            foreach_interrupt!(
                ($inst,can,FDCAN,IT0,$irq:ident) => {
                    pub type Interrupt0 = crate::interrupt::typelevel::$irq;
                };
                ($inst,can,FDCAN,IT1,$irq:ident) => {
                    pub type Interrupt1 = crate::interrupt::typelevel::$irq;
                };
            );
        }
        impl Instance for peripherals::$inst {
            type IT0Interrupt = $inst::Interrupt0;
            type IT1Interrupt = $inst::Interrupt1;
        }
    };

    ($inst:ident, $msg_ram_inst:ident) => {
        impl_fdcan!($inst, $msg_ram_inst, 0);
    };
}

#[cfg(not(can_fdcan_h7))]
foreach_peripheral!(
    (can, FDCAN) => { impl_fdcan!(FDCAN, FDCANRAM); };
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM1); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM2); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM3); };
);

#[cfg(can_fdcan_h7)]
foreach_peripheral!(
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM, 0x0000); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM, 0x0C00); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM, 0x1800); };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
