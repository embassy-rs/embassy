#[allow(unused_variables)]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

pub mod fd;
use embassy_hal_internal::{into_ref, PeripheralRef};
use fd::config::*;
use fd::filter::*;

use crate::can::fd::peripheral::Registers;
use crate::gpio::sealed::AFType;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub mod enums;
use enums::*;
pub mod util;

pub mod frame;
use frame::*;

#[cfg(feature = "time")]
type Timestamp = embassy_time::Instant;

#[cfg(not(feature = "time"))]
type Timestamp = u16;

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
            T::state().tx_waker.wake();
        }

        if ir.tefn() {
            regs.ir().write(|w| w.set_tefn(true));
            T::state().tx_waker.wake();
        }

        if ir.ped() || ir.pea() {
            regs.ir().write(|w| {
                w.set_ped(true);
                w.set_pea(true);
            });
        }

        if ir.rfn(0) {
            let fifonr = 0 as usize;
            regs.ir().write(|w| w.set_rfn(fifonr, true));

            T::state().rx_waker.wake();
        }

        if ir.rfn(1) {
            regs.ir().write(|w| w.set_rfn(1, true));
            T::state().rx_waker.wake();
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

/// Allows for Transmit Operations
pub trait Transmit {}
/// Allows for Receive Operations
pub trait Receive {}

/// Allows for the FdCan Instance to be released or to enter ConfigMode
pub struct PoweredDownMode;
/// Allows for the configuration for the Instance
pub struct ConfigMode;
/// This mode can be used for a “Hot Selftest”, meaning the FDCAN can be tested without
/// affecting a running CAN system connected to the FDCAN_TX and FDCAN_RX pins. In this
/// mode, FDCAN_RX pin is disconnected from the FDCAN and FDCAN_TX pin is held
/// recessive.
pub struct InternalLoopbackMode;
impl Transmit for InternalLoopbackMode {}
impl Receive for InternalLoopbackMode {}
/// This mode is provided for hardware self-test. To be independent from external stimulation,
/// the FDCAN ignores acknowledge errors (recessive bit sampled in the acknowledge slot of a
/// data / remote frame) in Loop Back mode. In this mode the FDCAN performs an internal
/// feedback from its transmit output to its receive input. The actual value of the FDCAN_RX
/// input pin is disregarded by the FDCAN. The transmitted messages can be monitored at the
/// FDCAN_TX transmit pin.
pub struct ExternalLoopbackMode;
impl Transmit for ExternalLoopbackMode {}
impl Receive for ExternalLoopbackMode {}
/// The normal use of the FdCan instance after configurations
pub struct NormalOperationMode;
impl Transmit for NormalOperationMode {}
impl Receive for NormalOperationMode {}
/// In Restricted operation mode the node is able to receive data and remote frames and to give
/// acknowledge to valid frames, but it does not send data frames, remote frames, active error
/// frames, or overload frames. In case of an error condition or overload condition, it does not
/// send dominant bits, instead it waits for the occurrence of bus idle condition to resynchronize
/// itself to the CAN communication. The error counters for transmit and receive are frozen while
/// error logging (can_errors) is active. TODO: automatically enter in this mode?
pub struct RestrictedOperationMode;
impl Receive for RestrictedOperationMode {}
///  In Bus monitoring mode (for more details refer to ISO11898-1, 10.12 Bus monitoring),
/// the FDCAN is able to receive valid data frames and valid remote frames, but cannot start a
/// transmission. In this mode, it sends only recessive bits on the CAN bus. If the FDCAN is
/// required to send a dominant bit (ACK bit, overload flag, active error flag), the bit is
/// rerouted internally so that the FDCAN can monitor it, even if the CAN bus remains in recessive
/// state. In Bus monitoring mode the TXBRP register is held in reset state. The Bus monitoring
/// mode can be used to analyze the traffic on a CAN bus without affecting it by the transmission
/// of dominant bits.
pub struct BusMonitoringMode;
impl Receive for BusMonitoringMode {}
/// Test mode must be used for production tests or self test only. The software control for
/// FDCAN_TX pin interferes with all CAN protocol functions. It is not recommended to use test
/// modes for application.
pub struct TestMode;

/// Operating modes trait
pub trait FdcanOperatingMode {}
impl FdcanOperatingMode for PoweredDownMode {}
impl FdcanOperatingMode for ConfigMode {}
impl FdcanOperatingMode for InternalLoopbackMode {}
impl FdcanOperatingMode for ExternalLoopbackMode {}
impl FdcanOperatingMode for NormalOperationMode {}
impl FdcanOperatingMode for RestrictedOperationMode {}
impl FdcanOperatingMode for BusMonitoringMode {}
impl FdcanOperatingMode for TestMode {}

/// FDCAN Instance
pub struct Fdcan<'d, T: Instance, M: FdcanOperatingMode> {
    config: crate::can::fd::config::FdCanConfig,
    /// Reference to internals.
    instance: FdcanInstance<'d, T>,
    _mode: PhantomData<M>,
    ns_per_timer_tick: u64, // For FDCAN internal timer
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

#[cfg(feature = "time")]
fn calc_timestamp<T: Instance>(ns_per_timer_tick: u64, ts_val: u16) -> Timestamp {
    let now_embassy = embassy_time::Instant::now();
    if ns_per_timer_tick == 0 {
        return now_embassy;
    }
    let cantime = { T::regs().tscv().read().tsc() };
    let delta = cantime.overflowing_sub(ts_val).0 as u64;
    let ns = ns_per_timer_tick * delta as u64;
    now_embassy - embassy_time::Duration::from_nanos(ns)
}

#[cfg(not(feature = "time"))]
fn calc_timestamp<T: Instance>(_ns_per_timer_tick: u64, ts_val: u16) -> Timestamp {
    ts_val
}

impl<'d, T: Instance> Fdcan<'d, T, ConfigMode> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
            + 'd,
    ) -> Fdcan<'d, T, ConfigMode> {
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

        let ns_per_timer_tick = calc_ns_per_timer_tick::<T>(config.frame_transmit);
        Self {
            config,
            instance: FdcanInstance(peri),
            _mode: PhantomData::<ConfigMode>,
            ns_per_timer_tick,
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

    /// Configures the bit timings for VBR data calculated from supplied bitrate.
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
}

macro_rules! impl_transition {
    ($from_mode:ident, $to_mode:ident, $name:ident, $func: ident) => {
        impl<'d, T: Instance> Fdcan<'d, T, $from_mode> {
            /// Transition from $from_mode:ident mode to $to_mode:ident mode
            pub fn $name(self) -> Fdcan<'d, T, $to_mode> {
                let ns_per_timer_tick = calc_ns_per_timer_tick::<T>(self.config.frame_transmit);
                T::registers().$func(self.config);
                let ret = Fdcan {
                    config: self.config,
                    instance: self.instance,
                    _mode: PhantomData::<$to_mode>,
                    ns_per_timer_tick,
                };
                ret
            }
        }
    };
}

impl_transition!(PoweredDownMode, ConfigMode, into_config_mode, into_config_mode);
impl_transition!(InternalLoopbackMode, ConfigMode, into_config_mode, into_config_mode);

impl_transition!(ConfigMode, NormalOperationMode, into_normal_mode, into_normal);
impl_transition!(
    ConfigMode,
    ExternalLoopbackMode,
    into_external_loopback_mode,
    into_external_loopback
);
impl_transition!(
    ConfigMode,
    InternalLoopbackMode,
    into_internal_loopback_mode,
    into_internal_loopback
);

impl<'d, T: Instance, M: FdcanOperatingMode> Fdcan<'d, T, M>
where
    M: Transmit,
    M: Receive,
{
    /// Flush one of the TX mailboxes.
    pub async fn flush(&self, idx: usize) {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());
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
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            if let Ok(dropped) = T::registers().write_classic(frame) {
                return Poll::Ready(dropped);
            }

            // Couldn't replace any lower priority frames.  Need to wait for some mailboxes
            // to clear.
            Poll::Pending
        })
        .await
    }

    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<(ClassicFrame, Timestamp), BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            if let Some((msg, ts)) = T::registers().read_classic(0) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some((msg, ts)) = T::registers().read_classic(1) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some(err) = T::registers().curr_error() {
                // TODO: this is probably wrong
                return Poll::Ready(Err(err));
            }
            Poll::Pending
        })
        .await
    }

    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            if let Ok(dropped) = T::registers().write_fd(frame) {
                return Poll::Ready(dropped);
            }

            // Couldn't replace any lower priority frames.  Need to wait for some mailboxes
            // to clear.
            Poll::Pending
        })
        .await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<(FdFrame, Timestamp), BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            if let Some((msg, ts)) = T::registers().read_fd(0) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some((msg, ts)) = T::registers().read_fd(1) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some(err) = T::registers().curr_error() {
                // TODO: this is probably wrong
                return Poll::Ready(Err(err));
            }
            Poll::Pending
        })
        .await
    }

    /// Split instance into separate Tx(write) and Rx(read) portions
    pub fn split(self) -> (FdcanTx<'d, T, M>, FdcanRx<'d, T, M>) {
        (
            FdcanTx {
                _instance: self.instance,
                _mode: self._mode,
            },
            FdcanRx {
                _instance1: PhantomData::<T>,
                _instance2: T::regs(),
                _mode: self._mode,
                ns_per_timer_tick: self.ns_per_timer_tick,
            },
        )
    }
}

/// FDCAN Rx only Instance
#[allow(dead_code)]
pub struct FdcanRx<'d, T: Instance, M: Receive> {
    _instance1: PhantomData<T>,
    _instance2: &'d crate::pac::can::Fdcan,
    _mode: PhantomData<M>,
    ns_per_timer_tick: u64, // For FDCAN internal timer
}

/// FDCAN Tx only Instance
pub struct FdcanTx<'d, T: Instance, M: Transmit> {
    _instance: FdcanInstance<'d, T>, //(PeripheralRef<'a, T>);
    _mode: PhantomData<M>,
}

impl<'c, 'd, T: Instance, M: Transmit> FdcanTx<'d, T, M> {
    /// Queues the message to be sent but exerts backpressure.  If a lower-priority
    /// frame is dropped from the mailbox, it is returned.  If no lower-priority frames
    /// can be replaced, this call asynchronously waits for a frame to be successfully
    /// transmitted, then tries again.
    pub async fn write(&mut self, frame: &ClassicFrame) -> Option<ClassicFrame> {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            if let Ok(dropped) = T::registers().write_classic(frame) {
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
    pub async fn write_fd(&mut self, frame: &FdFrame) -> Option<FdFrame> {
        poll_fn(|cx| {
            T::state().tx_waker.register(cx.waker());

            if let Ok(dropped) = T::registers().write_fd(frame) {
                return Poll::Ready(dropped);
            }

            // Couldn't replace any lower priority frames.  Need to wait for some mailboxes
            // to clear.
            Poll::Pending
        })
        .await
    }
}

impl<'c, 'd, T: Instance, M: Receive> FdcanRx<'d, T, M> {
    /// Returns the next received message frame
    pub async fn read(&mut self) -> Result<(ClassicFrame, Timestamp), BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            if let Some((msg, ts)) = T::registers().read_classic(0) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some((msg, ts)) = T::registers().read_classic(1) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some(err) = T::registers().curr_error() {
                // TODO: this is probably wrong
                return Poll::Ready(Err(err));
            }
            Poll::Pending
        })
        .await
    }

    /// Returns the next received message frame
    pub async fn read_fd(&mut self) -> Result<(FdFrame, Timestamp), BusError> {
        poll_fn(|cx| {
            T::state().err_waker.register(cx.waker());
            T::state().rx_waker.register(cx.waker());

            if let Some((msg, ts)) = T::registers().read_fd(0) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some((msg, ts)) = T::registers().read_fd(1) {
                let ts = calc_timestamp::<T>(self.ns_per_timer_tick, ts);
                return Poll::Ready(Ok((msg, ts)));
            } else if let Some(err) = T::registers().curr_error() {
                // TODO: this is probably wrong
                return Poll::Ready(Err(err));
            }
            Poll::Pending
        })
        .await
    }
}

pub(crate) mod sealed {
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
        const MSG_RAM_OFFSET: usize;

        fn regs() -> &'static crate::pac::can::Fdcan;
        fn registers() -> crate::can::fd::peripheral::Registers;
        fn ram() -> &'static crate::pac::fdcanram::Fdcanram;
        fn state() -> &'static State;

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

/// Trait for FDCAN interrupt channel 0
pub trait IT0Instance {
    /// Type for FDCAN interrupt channel 0
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Trait for FDCAN interrupt channel 1
pub trait IT1Instance {
    /// Type for FDCAN interrupt channel 1
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// InterruptableInstance trait
pub trait InterruptableInstance: IT0Instance + IT1Instance {}
/// Instance trait
pub trait Instance: sealed::Instance + RccPeripheral + InterruptableInstance + 'static {}
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
            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
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
