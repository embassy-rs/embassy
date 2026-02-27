#[allow(unused_variables)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::waitqueue::AtomicWaker;

use crate::can::fd::peripheral::Registers;
use crate::gpio::{AfType, OutputType, Pull, SealedPin as _, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::{self, RccInfo, RccPeripheral, SealedRccPeripheral};
use crate::{Peri, interrupt, peripherals};

pub(crate) mod fd;

use self::fd::config::*;
use self::fd::filter::*;
pub use self::fd::{config, filter};
pub use super::common::{BufferedCanReceiver, BufferedCanSender};
use super::common::{InfoRef, RxInfoRef, TxInfoRef};
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
        let regs = T::registers().regs;

        let ir = regs.ir().read();

        if ir.tc() {
            regs.ir().write(|w| w.set_tc(true));
        }
        if ir.tefn() {
            regs.ir().write(|w| w.set_tefn(true));
        }

        let recover_from_bo = T::info().state.lock(|s| {
            let state = s.borrow_mut();
            match &state.tx_mode {
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
                state.rx_mode.on_interrupt::<T>(0, state.ns_per_timer_tick);
            }
            if ir.rfn(1) {
                state.rx_mode.on_interrupt::<T>(1, state.ns_per_timer_tick);
            }

            state.automatic_bus_off_recovery
        });

        if ir.bo() {
            regs.ir().write(|w| w.set_bo(true));
            if let Some(true) = recover_from_bo
                && regs.psr().read().bo()
            {
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

fn calc_ns_per_timer_tick(
    info: &'static Info,
    freq: crate::time::Hertz,
    mode: crate::can::fd::config::FrameTransmissionConfig,
) -> u64 {
    match mode {
        // Use timestamp from Rx FIFO to adjust timestamp reported to user
        crate::can::fd::config::FrameTransmissionConfig::ClassicCanOnly => {
            let prescale: u64 = ({ info.regs.regs.nbtp().read().nbrp() } + 1) as u64
                * ({ info.regs.regs.tscc().read().tcp() } + 1) as u64;
            1_000_000_000 as u64 / (freq.0 as u64 * prescale)
        }
        // For VBR this is too hard because the FDCAN timer switches clock rate you need to configure to use
        // timer3 instead which is too hard to do from this module.
        _ => 0,
    }
}

/// FDCAN Configuration instance instance
/// Create instance of this first
pub struct CanConfigurator<'d> {
    _phantom: PhantomData<&'d ()>,
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    properties: Properties,
    periph_clock: crate::time::Hertz,
    info: InfoRef,
}

impl<'d> CanConfigurator<'d> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
        + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
        + 'd,
    ) -> CanConfigurator<'d> {
        set_as_af!(rx, AfType::input(Pull::None));
        set_as_af!(tx, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        rcc::enable_and_reset::<T>();

        let info = T::info();
        T::info().state.lock(|s| {
            s.borrow_mut().tx_pin_port = Some(tx.pin_port());
            s.borrow_mut().rx_pin_port = Some(rx.pin_port());
        });

        let mut config = crate::can::fd::config::FdCanConfig::default();
        config.timestamp_source = TimestampSource::Prescaler(TimestampPrescaler::_1);
        T::registers().into_config_mode(config);

        unsafe {
            T::IT0Interrupt::unpend(); // Not unsafe
            T::IT0Interrupt::enable();

            T::IT1Interrupt::unpend(); // Not unsafe
            T::IT1Interrupt::enable();
        }
        Self {
            _phantom: PhantomData,
            config,
            properties: Properties::new(T::info()),
            periph_clock: T::frequency(),
            info: InfoRef::new(info),
        }
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties {
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
        let bit_timing = util::calc_can_timings(self.periph_clock, bitrate).unwrap();

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
        let bit_timing = util::calc_can_timings(self.periph_clock, bitrate).unwrap();
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
    pub fn start(self, mode: OperatingMode) -> Can<'d> {
        let ns_per_timer_tick = calc_ns_per_timer_tick(&self.info, self.periph_clock, self.config.frame_transmit);
        self.info.state.lock(|s| {
            let mut state = s.borrow_mut();
            state.ns_per_timer_tick = ns_per_timer_tick;
            state.automatic_bus_off_recovery = Some(self.config.automatic_bus_off_recovery);
        });
        self.info.regs.into_mode(self.config, mode);
        Can {
            _phantom: PhantomData,
            config: self.config,
            _mode: mode,
            properties: Properties::new(&self.info),
            info: InfoRef::new(&self.info),
        }
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_normal_mode(self) -> Can<'d> {
        self.start(OperatingMode::NormalOperationMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_internal_loopback_mode(self) -> Can<'d> {
        self.start(OperatingMode::InternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_external_loopback_mode(self) -> Can<'d> {
        self.start(OperatingMode::ExternalLoopbackMode)
    }
}

/// FDCAN Instance
pub struct Can<'d> {
    _phantom: PhantomData<&'d ()>,
    config: crate::can::fd::config::FdCanConfig,
    _mode: OperatingMode,
    properties: Properties,
    info: InfoRef,
}

impl<'d> Can<'d> {
    /// Get driver properties
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// Flush one of the TX mailboxes.
    pub async fn flush(&self, idx: usize) {
        poll_fn(|cx| {
            self.info.state.lock(|s| {
                s.borrow_mut().tx_mode.register(cx.waker());
            });

            if idx > 3 {
                panic!("Bad mailbox");
            }
            let idx = 1 << idx;
            if !self.info.regs.regs.txbrp().read().trp(idx) {
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
        TxMode::write(&self.info, frame).await
    }

    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        RxMode::read_classic(&self.info).await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        TxMode::write_fd(&self.info, frame).await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<FdEnvelope, BusError> {
        RxMode::read_fd(&self.info).await
    }

    /// Split instance into separate portions: Tx(write), Rx(read), common properties
    pub fn split(self) -> (CanTx<'d>, CanRx<'d>, Properties) {
        (
            CanTx {
                _phantom: PhantomData,
                config: self.config,
                _mode: self._mode,
                info: TxInfoRef::new(&self.info),
            },
            CanRx {
                _phantom: PhantomData,
                _mode: self._mode,
                info: RxInfoRef::new(&self.info),
            },
            Properties {
                info: self.properties.info,
            },
        )
    }
    /// Join split rx and tx portions back together
    pub fn join(tx: CanTx<'d>, rx: CanRx<'d>) -> Self {
        Can {
            _phantom: PhantomData,
            config: tx.config,
            _mode: rx._mode,
            properties: Properties::new(&tx.info),
            info: InfoRef::new(&tx.info),
        }
    }

    /// Return a buffered instance of driver without CAN FD support. User must supply Buffers
    pub fn buffered<const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        self,
        tx_buf: &'static mut TxBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxBuf<RX_BUF_SIZE>,
    ) -> BufferedCan<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
        BufferedCan::new(&self.info, self._mode, tx_buf, rxb)
    }

    /// Return a buffered instance of driver with CAN FD support. User must supply Buffers
    pub fn buffered_fd<const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>(
        self,
        tx_buf: &'static mut TxFdBuf<TX_BUF_SIZE>,
        rxb: &'static mut RxFdBuf<RX_BUF_SIZE>,
    ) -> BufferedCanFd<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
        BufferedCanFd::new(&self.info, self._mode, tx_buf, rxb)
    }
}

/// User supplied buffer for RX Buffering
pub type RxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<Envelope, BusError>, BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Frame, BUF_SIZE>;

/// Buffered FDCAN Instance
pub struct BufferedCan<'d, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _phantom: PhantomData<&'d ()>,
    _mode: OperatingMode,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
    properties: Properties,
    info: InfoRef,
}

impl<'c, 'd, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> BufferedCan<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
    fn new(
        info: &'static Info,
        _mode: OperatingMode,
        tx_buf: &'static TxBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCan {
            _phantom: PhantomData,
            _mode,
            tx_buf,
            rx_buf,
            properties: Properties::new(info),
            info: InfoRef::new(info),
        }
        .setup()
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        self.info.state.lock(|s| {
            let rx_inner = super::common::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = super::common::ClassicBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            s.borrow_mut().rx_mode = RxMode::ClassicBuffered(rx_inner);
            s.borrow_mut().tx_mode = TxMode::ClassicBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: Frame) {
        self.tx_buf.send(frame).await;
        self.info.interrupt0.pend(); // Wake for Tx
        //T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        self.rx_buf.receive().await
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        BufferedCanSender {
            tx_buf: self.tx_buf.sender().into(),
            info: TxInfoRef::new(&self.info),
        }
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedCanReceiver {
        BufferedCanReceiver {
            rx_buf: self.rx_buf.receiver().into(),
            info: RxInfoRef::new(&self.info),
        }
    }
}

/// User supplied buffer for RX Buffering
pub type RxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, Result<FdEnvelope, BusError>, BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, FdFrame, BUF_SIZE>;

/// Sender that can be used for sending Classic CAN frames.
pub type BufferedFdCanSender = super::common::BufferedSender<'static, FdFrame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedFdCanReceiver = super::common::BufferedReceiver<'static, FdEnvelope>;

/// Buffered FDCAN Instance
pub struct BufferedCanFd<'d, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _phantom: PhantomData<&'d ()>,
    _mode: OperatingMode,
    tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
    properties: Properties,
    info: InfoRef,
}

impl<'c, 'd, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> BufferedCanFd<'d, TX_BUF_SIZE, RX_BUF_SIZE> {
    fn new(
        info: &'static Info,
        _mode: OperatingMode,
        tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCanFd {
            _phantom: PhantomData,
            _mode,
            tx_buf,
            rx_buf,
            properties: Properties::new(info),
            info: InfoRef::new(info),
        }
        .setup()
    }

    /// Get driver properties
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        self.info.state.lock(|s| {
            let rx_inner = super::common::FdBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = super::common::FdBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            s.borrow_mut().rx_mode = RxMode::FdBuffered(rx_inner);
            s.borrow_mut().tx_mode = TxMode::FdBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FdFrame) {
        self.tx_buf.send(frame).await;
        self.info.interrupt0.pend(); // Wake for Tx
        //T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<FdEnvelope, BusError> {
        self.rx_buf.receive().await
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedFdCanSender {
        BufferedFdCanSender {
            tx_buf: self.tx_buf.sender().into(),
            info: TxInfoRef::new(&self.info),
        }
    }

    /// Returns a receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
    pub fn reader(&self) -> BufferedFdCanReceiver {
        BufferedFdCanReceiver {
            rx_buf: self.rx_buf.receiver().into(),
            info: RxInfoRef::new(&self.info),
        }
    }
}

/// FDCAN Rx only Instance
pub struct CanRx<'d> {
    _phantom: PhantomData<&'d ()>,
    _mode: OperatingMode,
    info: RxInfoRef,
}

impl<'d> CanRx<'d> {
    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<Envelope, BusError> {
        RxMode::read_classic(&self.info).await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<FdEnvelope, BusError> {
        RxMode::read_fd(&self.info).await
    }
}

/// FDCAN Tx only Instance
pub struct CanTx<'d> {
    _phantom: PhantomData<&'d ()>,
    config: crate::can::fd::config::FdCanConfig,
    _mode: OperatingMode,
    info: TxInfoRef,
}

impl<'c, 'd> CanTx<'d> {
    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &Frame) -> Option<Frame> {
        TxMode::write(&self.info, frame).await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        TxMode::write_fd(&self.info, frame).await
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

    fn on_interrupt<T: Instance>(&self, fifonr: usize, ns_per_timer_tick: u64) {
        match self {
            RxMode::NonBuffered(waker) => {
                T::registers().regs.ir().write(|w| w.set_rfn(fifonr, true));
                waker.wake();
            }
            RxMode::ClassicBuffered(buf) => {
                T::registers().regs.ir().write(|w| w.set_rfn(fifonr, true));
                loop {
                    match self.try_read::<T>(ns_per_timer_tick) {
                        Some(Ok(envelope)) => {
                            let _ = buf.rx_sender.try_send(Ok(envelope));
                        }
                        Some(Err(err)) => {
                            // bus error states can persist; emit once and return to avoid
                            // spinning forever in interrupt context when no frames are available
                            let _ = buf.rx_sender.try_send(Err(err));
                            break;
                        }
                        None => break,
                    }
                }
            }
            RxMode::FdBuffered(buf) => {
                T::registers().regs.ir().write(|w| w.set_rfn(fifonr, true));
                loop {
                    match self.try_read_fd::<T>(ns_per_timer_tick) {
                        Some(Ok(envelope)) => {
                            let _ = buf.rx_sender.try_send(Ok(envelope));
                        }
                        Some(Err(err)) => {
                            // bus error states can persist; emit once and return to avoid
                            // spinning forever in interrupt context when no frames are available
                            let _ = buf.rx_sender.try_send(Err(err));
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    }

    //async fn read_classic<T: Instance>(&self) -> Result<Envelope, BusError> {
    fn try_read<T: Instance>(&self, ns_per_timer_tick: u64) -> Option<Result<Envelope, BusError>> {
        if let Some((frame, ts)) = T::registers().read(0) {
            let ts = T::registers().calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok(Envelope { ts, frame }))
        } else if let Some((frame, ts)) = T::registers().read(1) {
            let ts = T::registers().calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok(Envelope { ts, frame }))
        } else if let Some(err) = T::registers().curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    fn try_read_fd<T: Instance>(&self, ns_per_timer_tick: u64) -> Option<Result<FdEnvelope, BusError>> {
        if let Some((frame, ts)) = T::registers().read(0) {
            let ts = T::registers().calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok(FdEnvelope { ts, frame }))
        } else if let Some((frame, ts)) = T::registers().read(1) {
            let ts = T::registers().calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok(FdEnvelope { ts, frame }))
        } else if let Some(err) = T::registers().curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    fn read<F: CanHeader>(info: &'static Info, ns_per_timer_tick: u64) -> Option<Result<(F, Timestamp), BusError>> {
        if let Some((msg, ts)) = info.regs.read(0) {
            let ts = info.regs.calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok((msg, ts)))
        } else if let Some((msg, ts)) = info.regs.read(1) {
            let ts = info.regs.calc_timestamp(ns_per_timer_tick, ts);
            Some(Ok((msg, ts)))
        } else if let Some(err) = info.regs.curr_error() {
            // TODO: this is probably wrong
            Some(Err(err))
        } else {
            None
        }
    }

    async fn read_async<F: CanHeader>(info: &'static Info) -> Result<(F, Timestamp), BusError> {
        poll_fn(move |cx| {
            let ns_per_timer_tick = info.state.lock(|s| {
                let state = s.borrow_mut();
                state.err_waker.register(cx.waker());
                state.rx_mode.register(cx.waker());
                state.ns_per_timer_tick
            });
            match RxMode::read::<_>(info, ns_per_timer_tick) {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        })
        .await
    }

    async fn read_classic(info: &'static Info) -> Result<Envelope, BusError> {
        match RxMode::read_async::<_>(info).await {
            Ok((frame, ts)) => Ok(Envelope { ts, frame }),
            Err(e) => Err(e),
        }
    }

    async fn read_fd(info: &'static Info) -> Result<FdEnvelope, BusError> {
        match RxMode::read_async::<_>(info).await {
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
    async fn write_generic<F: embedded_can::Frame + CanHeader>(info: &'static Info, frame: &F) -> Option<F> {
        poll_fn(|cx| {
            info.state.lock(|s| {
                s.borrow_mut().tx_mode.register(cx.waker());
            });

            if let Ok(dropped) = info.regs.write(frame) {
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
    async fn write(info: &'static Info, frame: &Frame) -> Option<Frame> {
        TxMode::write_generic::<_>(info, frame).await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    async fn write_fd(info: &'static Info, frame: &FdFrame) -> Option<FdFrame> {
        TxMode::write_generic::<_>(info, frame).await
    }
}

/// Common driver properties, including filters and error counters
pub struct Properties {
    info: &'static Info,
    // phantom pointer to ensure !Sync
    //instance: PhantomData<*const T>,
}

impl Properties {
    fn new(info: &'static Info) -> Self {
        Self {
            info,
            //instance: Default::default(),
        }
    }

    /// Set a standard address CAN filter in the specified slot in FDCAN memory.
    #[inline]
    pub fn set_standard_filter(&self, slot: StandardFilterSlot, filter: StandardFilter) {
        self.info.regs.msg_ram_mut().filters.flssa[slot as usize].activate(filter);
    }

    /// Set the full array of standard address CAN filters in FDCAN memory.
    /// Overwrites all standard address filters in memory.
    pub fn set_standard_filters(&self, filters: &[StandardFilter; STANDARD_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            self.info.regs.msg_ram_mut().filters.flssa[i].activate(*f);
        }
    }

    /// Set an extended address CAN filter in the specified slot in FDCAN memory.
    #[inline]
    pub fn set_extended_filter(&self, slot: ExtendedFilterSlot, filter: ExtendedFilter) {
        self.info.regs.msg_ram_mut().filters.flesa[slot as usize].activate(filter);
    }

    /// Set the full array of extended address CAN filters in FDCAN memory.
    /// Overwrites all extended address filters in memory.
    pub fn set_extended_filters(&self, filters: &[ExtendedFilter; EXTENDED_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            self.info.regs.msg_ram_mut().filters.flesa[i].activate(*f);
        }
    }

    /// Get the CAN RX error counter
    pub fn rx_error_count(&self) -> u8 {
        self.info.regs.regs.ecr().read().rec()
    }

    /// Get the CAN TX error counter
    pub fn tx_error_count(&self) -> u8 {
        self.info.regs.regs.ecr().read().tec()
    }

    /// Get the current bus error mode
    pub fn bus_error_mode(&self) -> BusErrorMode {
        // This read will clear LEC and DLEC. This is not ideal, but protocol
        // error reporting in this driver should have a big ol' FIXME on it
        // anyway!
        let psr = self.info.regs.regs.psr().read();
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
    receiver_instance_count: usize,
    sender_instance_count: usize,
    tx_pin_port: Option<u8>,
    rx_pin_port: Option<u8>,
    automatic_bus_off_recovery: Option<bool>, // controlled by CanConfigurator::start()
    pub err_waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            rx_mode: RxMode::NonBuffered(AtomicWaker::new()),
            tx_mode: TxMode::NonBuffered(AtomicWaker::new()),
            ns_per_timer_tick: 0,
            err_waker: AtomicWaker::new(),
            receiver_instance_count: 0,
            sender_instance_count: 0,
            tx_pin_port: None,
            rx_pin_port: None,
            automatic_bus_off_recovery: None,
        }
    }
}

type SharedState = embassy_sync::blocking_mutex::Mutex<CriticalSectionRawMutex, core::cell::RefCell<State>>;
pub(crate) struct Info {
    regs: Registers,
    interrupt0: crate::interrupt::Interrupt,
    _interrupt1: crate::interrupt::Interrupt,
    pub(crate) tx_waker: fn(),
    state: SharedState,
    rcc_info: RccInfo,
}

impl Info {
    pub(crate) fn adjust_reference_counter(&self, val: RefCountOp) {
        self.state.lock(|s| {
            let mut mut_state = s.borrow_mut();
            match val {
                RefCountOp::NotifySenderCreated => {
                    mut_state.sender_instance_count += 1;
                }
                RefCountOp::NotifySenderDestroyed => {
                    mut_state.sender_instance_count -= 1;
                    if 0 == mut_state.sender_instance_count {
                        (*mut_state).tx_mode = TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
                    }
                }
                RefCountOp::NotifyReceiverCreated => {
                    mut_state.receiver_instance_count += 1;
                }
                RefCountOp::NotifyReceiverDestroyed => {
                    mut_state.receiver_instance_count -= 1;
                    if 0 == mut_state.receiver_instance_count {
                        (*mut_state).rx_mode = RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
                    }
                }
            }
            if mut_state.sender_instance_count == 0 && mut_state.receiver_instance_count == 0 {
                unsafe {
                    let tx_pin = crate::gpio::AnyPin::steal(mut_state.tx_pin_port.unwrap());
                    tx_pin.set_as_disconnected();
                    let rx_pin = crate::gpio::AnyPin::steal(mut_state.rx_pin_port.unwrap());
                    rx_pin.set_as_disconnected();
                    self.interrupt0.disable();
                    self.rcc_info.disable();
                }
            }
        });
    }
}

trait SealedInstance {
    const MSG_RAM_OFFSET: usize;

    fn info() -> &'static Info;
    fn registers() -> crate::can::fd::peripheral::Registers;
}

/// Instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static {
    /// Interrupt 0
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// Interrupt 1
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Fdcan Instance struct
pub struct FdcanInstance<'a, T: Instance>(Peri<'a, T>);

macro_rules! impl_fdcan {
    ($inst:ident,
        //$irq0:ident, $irq1:ident,
        $msg_ram_inst:ident, $msg_ram_offset:literal) => {
        impl SealedInstance for peripherals::$inst {
            const MSG_RAM_OFFSET: usize = $msg_ram_offset;

            fn info() -> &'static Info {

                static INFO: Info = Info {
                    regs: Registers{regs: crate::pac::$inst, msgram: crate::pac::$msg_ram_inst, msg_ram_offset: $msg_ram_offset},
                    interrupt0: crate::_generated::peripheral_interrupts::$inst::IT0::IRQ,
                    _interrupt1: crate::_generated::peripheral_interrupts::$inst::IT1::IRQ,
                    tx_waker: crate::_generated::peripheral_interrupts::$inst::IT0::pend,
                    state: embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(State::new())),
                    rcc_info: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
            fn registers() -> Registers {
                Registers{regs: crate::pac::$inst, msgram: crate::pac::$msg_ram_inst, msg_ram_offset: Self::MSG_RAM_OFFSET}
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
