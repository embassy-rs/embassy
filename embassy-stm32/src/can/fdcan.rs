#[allow(unused_variables)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::can::fd::peripheral::Registers;
use crate::gpio::sealed::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub mod enums;
pub(crate) mod fd;
pub mod frame;
mod util;

use enums::*;
use fd::config::*;
use fd::filter::*;
pub use fd::{config, filter};
use frame::*;

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

        {
            if ir.tc() {
                regs.ir().write(|w| w.set_tc(true));
            }
            if ir.tefn() {
                regs.ir().write(|w| w.set_tefn(true));
            }

            match &T::state().tx_mode {
                sealed::TxMode::NonBuffered(waker) => waker.wake(),
                sealed::TxMode::ClassicBuffered(buf) => {
                    if !T::registers().tx_queue_is_full() {
                        match buf.tx_receiver.try_receive() {
                            Ok(frame) => {
                                _ = T::registers().write(&frame);
                            }
                            Err(_) => {}
                        }
                    }
                }
                sealed::TxMode::FdBuffered(buf) => {
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
        }

        if ir.ped() || ir.pea() {
            regs.ir().write(|w| {
                w.set_ped(true);
                w.set_pea(true);
            });
        }

        if ir.rfn(0) {
            T::state().rx_mode.on_interrupt::<T>(0);
        }

        if ir.rfn(1) {
            T::state().rx_mode.on_interrupt::<T>(1);
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
pub enum FdcanOperatingMode {
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
pub struct FdcanConfigurator<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    instance: FdcanInstance<'d, T>,
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

impl<'d, T: Instance> FdcanConfigurator<'d, T> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
            + 'd,
    ) -> FdcanConfigurator<'d, T> {
        into_ref!(peri, rx, tx);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::enable_and_reset();

        let mut config = crate::can::fd::config::FdCanConfig::default();
        T::registers().into_config_mode(config);

        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        T::configure_msg_ram();
        unsafe {
            // Enable timestamping
            #[cfg(not(stm32h7))]
            T::regs()
                .tscc()
                .write(|w| w.set_tss(stm32_metapac::can::vals::Tss::INCREMENT));
            #[cfg(stm32h7)]
            T::regs().tscc().write(|w| w.set_tss(0x01));
            config.timestamp_source = TimestampSource::Prescaler(TimestampPrescaler::_1);

            T::IT0Interrupt::unpend(); // Not unsafe
            T::IT0Interrupt::enable();

            T::IT1Interrupt::unpend(); // Not unsafe
            T::IT1Interrupt::enable();

            // this isn't really documented in the reference manual
            // but corresponding txbtie bit has to be set for the TC (TxComplete) interrupt to fire
            T::regs().txbtie().write(|w| w.0 = 0xffff_ffff);
        }

        T::regs().ie().modify(|w| {
            w.set_rfne(0, true); // Rx Fifo 0 New Msg
            w.set_rfne(1, true); // Rx Fifo 1 New Msg
            w.set_tce(true); //  Tx Complete
        });
        T::regs().ile().modify(|w| {
            w.set_eint0(true); // Interrupt Line 0
            w.set_eint1(true); // Interrupt Line 1
        });

        Self {
            config,
            instance: FdcanInstance(peri),
        }
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

    /// Set an Standard Address CAN filter into slot 'id'
    #[inline]
    pub fn set_standard_filter(&mut self, slot: StandardFilterSlot, filter: StandardFilter) {
        T::registers().msg_ram_mut().filters.flssa[slot as usize].activate(filter);
    }

    /// Set an array of Standard Address CAN filters and overwrite the current set
    pub fn set_standard_filters(&mut self, filters: &[StandardFilter; STANDARD_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            T::registers().msg_ram_mut().filters.flssa[i].activate(*f);
        }
    }

    /// Set an Extended Address CAN filter into slot 'id'
    #[inline]
    pub fn set_extended_filter(&mut self, slot: ExtendedFilterSlot, filter: ExtendedFilter) {
        T::registers().msg_ram_mut().filters.flesa[slot as usize].activate(filter);
    }

    /// Set an array of Extended Address CAN filters and overwrite the current set
    pub fn set_extended_filters(&mut self, filters: &[ExtendedFilter; EXTENDED_FILTER_MAX as usize]) {
        for (i, f) in filters.iter().enumerate() {
            T::registers().msg_ram_mut().filters.flesa[i].activate(*f);
        }
    }

    /// Start in mode.
    pub fn start(self, mode: FdcanOperatingMode) -> Fdcan<'d, T> {
        let ns_per_timer_tick = calc_ns_per_timer_tick::<T>(self.config.frame_transmit);
        critical_section::with(|_| unsafe {
            T::mut_state().ns_per_timer_tick = ns_per_timer_tick;
        });
        T::registers().into_mode(self.config, mode);
        let ret = Fdcan {
            config: self.config,
            instance: self.instance,
            _mode: mode,
        };
        ret
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_normal_mode(self) -> Fdcan<'d, T> {
        self.start(FdcanOperatingMode::NormalOperationMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_internal_loopback_mode(self) -> Fdcan<'d, T> {
        self.start(FdcanOperatingMode::InternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_external_loopback_mode(self) -> Fdcan<'d, T> {
        self.start(FdcanOperatingMode::ExternalLoopbackMode)
    }
}

/// FDCAN Instance
pub struct Fdcan<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    instance: FdcanInstance<'d, T>,
    _mode: FdcanOperatingMode,
}

impl<'d, T: Instance> Fdcan<'d, T> {
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
    pub async fn write(&mut self, frame: &ClassicFrame) -> Option<ClassicFrame> {
        T::state().tx_mode.write::<T>(frame).await
    }

    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<(ClassicFrame, Timestamp), BusError> {
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
    pub async fn read_fd(&mut self) -> Result<(FdFrame, Timestamp), BusError> {
        T::state().rx_mode.read_fd::<T>().await
    }

    /// Split instance into separate Tx(write) and Rx(read) portions
    pub fn split(self) -> (FdcanTx<'d, T>, FdcanRx<'d, T>) {
        (
            FdcanTx {
                config: self.config,
                _instance: self.instance,
                _mode: self._mode,
            },
            FdcanRx {
                _instance1: PhantomData::<T>,
                _instance2: T::regs(),
                _mode: self._mode,
            },
        )
    }

    /// Join split rx and tx portions back together
    pub fn join(tx: FdcanTx<'d, T>, rx: FdcanRx<'d, T>) -> Self {
        Fdcan {
            config: tx.config,
            //_instance2: T::regs(),
            instance: tx._instance,
            _mode: rx._mode,
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
pub type RxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, (ClassicFrame, Timestamp), BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, ClassicFrame, BUF_SIZE>;

/// Buffered FDCAN Instance
pub struct BufferedCan<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: FdcanOperatingMode,
    tx_buf: &'static TxBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxBuf<RX_BUF_SIZE>,
}

/// Sender that can be used for sending CAN frames.
pub type BufferedCanSender = embassy_sync::channel::DynamicSender<'static, ClassicFrame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedCanReceiver = embassy_sync::channel::DynamicReceiver<'static, (ClassicFrame, Timestamp)>;

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>
    BufferedCan<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn new(
        _instance1: PhantomData<T>,
        _instance2: &'d crate::pac::can::Fdcan,
        _mode: FdcanOperatingMode,
        tx_buf: &'static TxBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCan {
            _instance1,
            _instance2,
            _mode,
            tx_buf,
            rx_buf,
        }
        .setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = sealed::ClassicBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = sealed::ClassicBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            T::mut_state().rx_mode = sealed::RxMode::ClassicBuffered(rx_inner);
            T::mut_state().tx_mode = sealed::TxMode::ClassicBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: ClassicFrame) {
        self.tx_buf.send(frame).await;
        T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<(ClassicFrame, Timestamp), BusError> {
        Ok(self.rx_buf.receive().await)
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedCanSender {
        self.tx_buf.sender().into()
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
            T::mut_state().rx_mode = sealed::RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            T::mut_state().tx_mode = sealed::TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

/// User supplied buffer for RX Buffering
pub type RxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, (FdFrame, Timestamp), BUF_SIZE>;

/// User supplied buffer for TX buffering
pub type TxFdBuf<const BUF_SIZE: usize> = Channel<CriticalSectionRawMutex, FdFrame, BUF_SIZE>;

/// Buffered FDCAN Instance
pub struct BufferedCanFd<'d, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: FdcanOperatingMode,
    tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
    rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
}

/// Sender that can be used for sending CAN frames.
pub type BufferedFdCanSender = embassy_sync::channel::DynamicSender<'static, FdFrame>;

/// Receiver that can be used for receiving CAN frames. Note, each CAN frame will only be received by one receiver.
pub type BufferedFdCanReceiver = embassy_sync::channel::DynamicReceiver<'static, (FdFrame, Timestamp)>;

impl<'c, 'd, T: Instance, const TX_BUF_SIZE: usize, const RX_BUF_SIZE: usize>
    BufferedCanFd<'d, T, TX_BUF_SIZE, RX_BUF_SIZE>
{
    fn new(
        _instance1: PhantomData<T>,
        _instance2: &'d crate::pac::can::Fdcan,
        _mode: FdcanOperatingMode,
        tx_buf: &'static TxFdBuf<TX_BUF_SIZE>,
        rx_buf: &'static RxFdBuf<RX_BUF_SIZE>,
    ) -> Self {
        BufferedCanFd {
            _instance1,
            _instance2,
            _mode,
            tx_buf,
            rx_buf,
        }
        .setup()
    }

    fn setup(self) -> Self {
        // We don't want interrupts being processed while we change modes.
        critical_section::with(|_| unsafe {
            let rx_inner = sealed::FdBufferedRxInner {
                rx_sender: self.rx_buf.sender().into(),
            };
            let tx_inner = sealed::FdBufferedTxInner {
                tx_receiver: self.tx_buf.receiver().into(),
            };
            T::mut_state().rx_mode = sealed::RxMode::FdBuffered(rx_inner);
            T::mut_state().tx_mode = sealed::TxMode::FdBuffered(tx_inner);
        });
        self
    }

    /// Async write frame to TX buffer.
    pub async fn write(&mut self, frame: FdFrame) {
        self.tx_buf.send(frame).await;
        T::IT0Interrupt::pend(); // Wake for Tx
    }

    /// Async read frame from RX buffer.
    pub async fn read(&mut self) -> Result<(FdFrame, Timestamp), BusError> {
        Ok(self.rx_buf.receive().await)
    }

    /// Returns a sender that can be used for sending CAN frames.
    pub fn writer(&self) -> BufferedFdCanSender {
        self.tx_buf.sender().into()
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
            T::mut_state().rx_mode = sealed::RxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
            T::mut_state().tx_mode = sealed::TxMode::NonBuffered(embassy_sync::waitqueue::AtomicWaker::new());
        });
    }
}

/// FDCAN Rx only Instance
pub struct FdcanRx<'d, T: Instance> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: FdcanOperatingMode,
}

/// FDCAN Tx only Instance
pub struct FdcanTx<'d, T: Instance> {
    config: crate::can::fd::config::FdCanConfig,
    _instance: FdcanInstance<'d, T>, //(PeripheralRef<'a, T>);
    _mode: FdcanOperatingMode,
}

impl<'c, 'd, T: Instance> FdcanTx<'d, T> {
    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &ClassicFrame) -> Option<ClassicFrame> {
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

impl<'c, 'd, T: Instance> FdcanRx<'d, T> {
    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<(ClassicFrame, Timestamp), BusError> {
        T::state().rx_mode.read_classic::<T>().await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<(FdFrame, Timestamp), BusError> {
        T::state().rx_mode.read_fd::<T>().await
    }
}

pub(crate) mod sealed {
    use core::future::poll_fn;
    use core::task::Poll;

    use embassy_sync::channel::{DynamicReceiver, DynamicSender};
    use embassy_sync::waitqueue::AtomicWaker;

    use super::CanHeader;
    use crate::can::_version::{BusError, Timestamp};
    use crate::can::frame::{ClassicFrame, FdFrame};

    pub struct ClassicBufferedRxInner {
        pub rx_sender: DynamicSender<'static, (ClassicFrame, Timestamp)>,
    }
    pub struct ClassicBufferedTxInner {
        pub tx_receiver: DynamicReceiver<'static, ClassicFrame>,
    }

    pub struct FdBufferedRxInner {
        pub rx_sender: DynamicSender<'static, (FdFrame, Timestamp)>,
    }
    pub struct FdBufferedTxInner {
        pub tx_receiver: DynamicReceiver<'static, FdFrame>,
    }

    pub enum RxMode {
        NonBuffered(AtomicWaker),
        ClassicBuffered(ClassicBufferedRxInner),
        FdBuffered(FdBufferedRxInner),
    }

    impl RxMode {
        pub fn register(&self, arg: &core::task::Waker) {
            match self {
                RxMode::NonBuffered(waker) => waker.register(arg),
                _ => {
                    panic!("Bad Mode")
                }
            }
        }

        pub fn on_interrupt<T: Instance>(&self, fifonr: usize) {
            T::regs().ir().write(|w| w.set_rfn(fifonr, true));
            match self {
                RxMode::NonBuffered(waker) => {
                    waker.wake();
                }
                RxMode::ClassicBuffered(buf) => {
                    if let Some(r) = T::registers().read(fifonr) {
                        let ts = T::calc_timestamp(T::state().ns_per_timer_tick, r.1);
                        let _ = buf.rx_sender.try_send((r.0, ts));
                    }
                }
                RxMode::FdBuffered(buf) => {
                    if let Some(r) = T::registers().read(fifonr) {
                        let ts = T::calc_timestamp(T::state().ns_per_timer_tick, r.1);
                        let _ = buf.rx_sender.try_send((r.0, ts));
                    }
                }
            }
        }

        async fn read<T: Instance, F: CanHeader>(&self) -> Result<(F, Timestamp), BusError> {
            poll_fn(|cx| {
                T::state().err_waker.register(cx.waker());
                self.register(cx.waker());

                if let Some((msg, ts)) = T::registers().read(0) {
                    let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
                    return Poll::Ready(Ok((msg, ts)));
                } else if let Some((msg, ts)) = T::registers().read(1) {
                    let ts = T::calc_timestamp(T::state().ns_per_timer_tick, ts);
                    return Poll::Ready(Ok((msg, ts)));
                } else if let Some(err) = T::registers().curr_error() {
                    // TODO: this is probably wrong
                    return Poll::Ready(Err(err));
                }
                Poll::Pending
            })
            .await
        }

        pub async fn read_classic<T: Instance>(&self) -> Result<(ClassicFrame, Timestamp), BusError> {
            self.read::<T, _>().await
        }

        pub async fn read_fd<T: Instance>(&self) -> Result<(FdFrame, Timestamp), BusError> {
            self.read::<T, _>().await
        }
    }

    pub enum TxMode {
        NonBuffered(AtomicWaker),
        ClassicBuffered(ClassicBufferedTxInner),
        FdBuffered(FdBufferedTxInner),
    }

    impl TxMode {
        pub fn register(&self, arg: &core::task::Waker) {
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
        pub async fn write<T: Instance>(&self, frame: &ClassicFrame) -> Option<ClassicFrame> {
            self.write_generic::<T, _>(frame).await
        }

        /// Queues the message to be sent but exerts backpressure.  If a lower-priority
        /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
        /// can be replaced, this call asynchronously waits for a frame to be successfully
        /// transmitted, then tries again.
        pub async fn write_fd<T: Instance>(&self, frame: &FdFrame) -> Option<FdFrame> {
            self.write_generic::<T, _>(frame).await
        }
    }

    pub struct State {
        pub rx_mode: RxMode,
        pub tx_mode: TxMode,
        pub ns_per_timer_tick: u64,

        pub err_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                rx_mode: RxMode::NonBuffered(AtomicWaker::new()),
                tx_mode: TxMode::NonBuffered(AtomicWaker::new()),
                ns_per_timer_tick: 0,
                err_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        const MSG_RAM_OFFSET: usize;

        fn regs() -> &'static crate::pac::can::Fdcan;
        fn registers() -> crate::can::fd::peripheral::Registers;
        fn ram() -> &'static crate::pac::fdcanram::Fdcanram;
        fn state() -> &'static State;
        unsafe fn mut_state() -> &'static mut State;
        fn calc_timestamp(ns_per_timer_tick: u64, ts_val: u16) -> Timestamp;

        #[cfg(not(stm32h7))]
        fn configure_msg_ram() {}

        #[cfg(stm32h7)]
        fn configure_msg_ram() {
            let r = Self::regs();

            use crate::can::fd::message_ram::*;
            //use fdcan::message_ram::*;
            let mut offset_words = Self::MSG_RAM_OFFSET as u16;

            // 11-bit filter
            r.sidfc().modify(|w| w.set_flssa(offset_words));
            offset_words += STANDARD_FILTER_MAX as u16;

            // 29-bit filter
            r.xidfc().modify(|w| w.set_flesa(offset_words));
            offset_words += 2 * EXTENDED_FILTER_MAX as u16;

            // Rx FIFO 0 and 1
            for i in 0..=1 {
                r.rxfc(i).modify(|w| {
                    w.set_fsa(offset_words);
                    w.set_fs(RX_FIFO_MAX);
                    w.set_fwm(RX_FIFO_MAX);
                });
                offset_words += 18 * RX_FIFO_MAX as u16;
            }

            // Rx buffer - see below
            // Tx event FIFO
            r.txefc().modify(|w| {
                w.set_efsa(offset_words);
                w.set_efs(TX_EVENT_MAX);
                w.set_efwm(TX_EVENT_MAX);
            });
            offset_words += 2 * TX_EVENT_MAX as u16;

            // Tx buffers
            r.txbc().modify(|w| {
                w.set_tbsa(offset_words);
                w.set_tfqs(TX_FIFO_MAX);
            });
            offset_words += 18 * TX_FIFO_MAX as u16;

            // Rx Buffer - not used
            r.rxbc().modify(|w| {
                w.set_rbsa(offset_words);
            });

            // TX event FIFO?
            // Trigger memory?

            // Set the element sizes to 16 bytes
            r.rxesc().modify(|w| {
                w.set_rbds(0b111);
                for i in 0..=1 {
                    w.set_fds(i, 0b111);
                }
            });
            r.txesc().modify(|w| {
                w.set_tbds(0b111);
            })
        }
    }
}

/// Instance trait
pub trait Instance: sealed::Instance + RccPeripheral + 'static {
    /// Interrupt 0
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// Interrupt 0
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Fdcan Instance struct
pub struct FdcanInstance<'a, T>(PeripheralRef<'a, T>);

macro_rules! impl_fdcan {
    ($inst:ident, $msg_ram_inst:ident, $msg_ram_offset:literal) => {
        impl sealed::Instance for peripherals::$inst {
            const MSG_RAM_OFFSET: usize = $msg_ram_offset;

            fn regs() -> &'static crate::pac::can::Fdcan {
                &crate::pac::$inst
            }
            fn registers() -> Registers {
                Registers{regs: &crate::pac::$inst, msgram: &crate::pac::$msg_ram_inst}
            }
            fn ram() -> &'static crate::pac::fdcanram::Fdcanram {
                &crate::pac::$msg_ram_inst
            }
            unsafe fn mut_state() -> & 'static mut sealed::State {
                static mut STATE: sealed::State = sealed::State::new();
                & mut STATE
            }
            fn state() -> &'static sealed::State {
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

#[cfg(not(stm32h7))]
foreach_peripheral!(
    (can, FDCAN) => { impl_fdcan!(FDCAN, FDCANRAM); };
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM1); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM2); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM3); };
);

#[cfg(stm32h7)]
foreach_peripheral!(
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM, 0x0000); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM, 0x0C00); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM, 0x1800); };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
