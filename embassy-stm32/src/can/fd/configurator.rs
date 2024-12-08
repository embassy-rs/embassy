use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use bit_field::BitField;
use embassy_hal_internal::into_ref;
use embedded_can::{ExtendedId, StandardId};

use crate::can::config::ErrorHandlingMode;
use crate::can::{Classic, Fd};
use crate::interrupt::typelevel::Interrupt;
use crate::{
    can::{
        common::CanMode,
        config::{TimestampPrescaler, TimestampSource},
        filter::{ExtendedFilter, StandardFilter},
        util, Can, Instance, OperatingMode, RxPin, TxPin,
    },
    gpio::{AfType, OutputType, Pull, Speed},
    interrupt, rcc, Peripheral,
};

use super::config::CanFdMode;
use super::filter::Filter;
use super::{calc_ns_per_timer_tick, IT0InterruptHandler, IT1InterruptHandler, Info, State};

/// FDCAN Configuration instance instance
/// Create instance of this first
pub struct CanConfigurator<'d> {
    _phantom: PhantomData<&'d ()>,
    config: crate::can::fd::config::FdCanConfig,
    info: &'static Info,
    state: &'static State,
    /// Reference to internals.
    properties: Properties,
    periph_clock: crate::time::Hertz,
}

impl<'d> CanConfigurator<'d> {
    /// Creates a new Fdcan instance, keeping the peripheral in sleep mode.
    /// You must call [Fdcan::enable_non_blocking] to use the peripheral.
    pub fn new<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irqs: impl interrupt::typelevel::Binding<T::IT0Interrupt, IT0InterruptHandler<T>>
            + interrupt::typelevel::Binding<T::IT1Interrupt, IT1InterruptHandler<T>>
            + 'd,
    ) -> CanConfigurator<'d> {
        into_ref!(_peri, rx, tx);

        rx.set_as_af(rx.af_num(), AfType::input(Pull::None));
        tx.set_as_af(tx.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));

        rcc::enable_and_reset::<T>();

        let mut config = crate::can::fd::config::FdCanConfig::default();
        config.timestamp_source = TimestampSource::Prescaler(TimestampPrescaler::_1);

        {
            let info = T::info();
            info.low.enter_init_mode();
            info.low.apply_config(&config);
        }

        //unsafe { T::mut_info() }.low.apply_message_ram_config();

        rx.set_as_af(rx.af_num(), AfType::input(Pull::None));
        tx.set_as_af(tx.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));

        unsafe {
            T::IT0Interrupt::unpend(); // Not unsafe
            T::IT0Interrupt::enable();

            T::IT1Interrupt::unpend(); // Not unsafe
            T::IT1Interrupt::enable();
        }
        Self {
            _phantom: PhantomData,
            config,
            info: T::info(),
            state: T::state(),
            properties: Properties::new(T::state()),
            periph_clock: T::frequency(),
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
        self.config.can_fd_mode = CanFdMode::AllowFdCanAndBRS;
        self.config = self.config.set_data_bit_timing(nbtr);
    }

    /// Start in mode.
    pub fn start<M: CanMode>(self, mode: OperatingMode) -> Can<'d, M> {
        use crate::can::common::DynCanMode;

        match M::dyn_can_mode() {
            // Creating a FD parametrized CAN instance for Classic hardware config
            // is supported. Can be used to be able to switch back and fourth without
            // refactoring all your code or parametrizing it.
            DynCanMode::Fd => (),
            // Creating a Classic parametrized CAN instance for FD hardware config
            // could lead to unexpected behaviour, and is not supported.
            DynCanMode::Classic => assert!(
                self.config.can_fd_mode == CanFdMode::ClassicCanOnly,
                "Short frame types are not supported for FD hardware configuration"
            ),
        }

        let ns_per_timer_tick = calc_ns_per_timer_tick(self.info, self.periph_clock, self.config.can_fd_mode);

        // TODO: I really don't like this..
        critical_section::with(|_| {
            let state = self.state as *const State;
            unsafe {
                let mut_state = state as *mut State;
                (*mut_state).ns_per_timer_tick = ns_per_timer_tick;
            }

            let info = self.info as *const Info;
            unsafe {
                let mut_info = info as *mut Info;
                (*mut_info).low.apply_message_ram_config(self.config.message_ram_config);
            }
        });

        let mut settings_flags = 0;
        match self.config.error_handling_mode {
            ErrorHandlingMode::Auto => {
                settings_flags.set_bit(State::FLAG_AUTO_RECOVER_BUS_OFF, true);
            }
            ErrorHandlingMode::Local => {
                settings_flags.set_bit(State::FLAG_PROPAGATE_ERRORS_TO_RX, true);
                settings_flags.set_bit(State::FLAG_PROPAGATE_ERRORS_TO_TX, true);
            }
            ErrorHandlingMode::LocalRx => {
                settings_flags.set_bit(State::FLAG_PROPAGATE_ERRORS_TO_RX, true);
            }
            ErrorHandlingMode::Central => (),
        };
        self.state.settings_flags.store(settings_flags, Ordering::Relaxed);

        self.info.low.apply_config(&self.config);
        self.info.low.into_mode(mode);

        Can {
            _phantom: PhantomData,
            info: self.info,
            state: self.state,
            _mode: mode,
            properties: Properties::new(self.state),
        }
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_normal_mode_classic(self) -> Can<'d, Classic> {
        self.start(OperatingMode::NormalOperationMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_normal_mode_fd(self) -> Can<'d, Fd> {
        self.start(OperatingMode::NormalOperationMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_internal_loopback_mode_classic(self) -> Can<'d, Classic> {
        self.start(OperatingMode::InternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_internal_loopback_mode_fd(self) -> Can<'d, Fd> {
        self.start(OperatingMode::InternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_external_loopback_mode_classic(self) -> Can<'d, Classic> {
        self.start(OperatingMode::ExternalLoopbackMode)
    }

    /// Start, entering mode. Does same as start(mode)
    pub fn into_external_loopback_mode_fd(self) -> Can<'d, Fd> {
        self.start(OperatingMode::ExternalLoopbackMode)
    }
}

trait SealedIdType: core::fmt::Debug + Clone + Copy {
    type Storage: core::fmt::Debug + Clone + Copy;
    fn alloc_slot(state: &'static State) -> Option<u8>;
    fn dealloc_slot(state: &'static State, slot_idx: u8);
    fn set_slot(state: &'static State, slot: u8, filter: Filter<Self, Self::Storage>);
}
#[allow(private_bounds)]
pub trait IdType: SealedIdType {}

impl SealedIdType for StandardId {
    type Storage = u16;
    fn alloc_slot(state: &'static State) -> Option<u8> {
        state.standard_filter_alloc.allocate().map(|v| v as u8)
    }
    fn dealloc_slot(state: &'static State, slot_idx: u8) {
        state.standard_filter_alloc.deallocate(slot_idx as usize);
    }
    fn set_slot(state: &'static State, slot: u8, filter: StandardFilter) {
        state.info.low.set_standard_filter(slot, filter);
    }
}
impl IdType for StandardId {}

impl SealedIdType for ExtendedId {
    type Storage = u32;
    fn alloc_slot(state: &'static State) -> Option<u8> {
        state.extended_filter_alloc.allocate().map(|v| v as u8)
    }
    fn dealloc_slot(state: &'static State, slot_idx: u8) {
        state.extended_filter_alloc.deallocate(slot_idx as usize);
    }
    fn set_slot(state: &'static State, slot: u8, filter: ExtendedFilter) {
        state.info.low.set_extended_filter(slot, filter);
    }
}
impl IdType for ExtendedId {}

pub struct FilterSlot<I: IdType> {
    _phantom: PhantomData<I>,
    state: &'static State,
    slot_idx: u8,
}

impl<I: IdType> FilterSlot<I> {
    /// Sets the filter slot to a given filter.
    #[allow(private_interfaces)]
    pub fn set(&self, filter: Filter<I, I::Storage>) {
        I::set_slot(self.state, self.slot_idx, filter);
    }
}

impl<I: IdType> Drop for FilterSlot<I> {
    fn drop(&mut self) {
        I::dealloc_slot(self.state, self.slot_idx);
    }
}

/// Common driver properties, including filters and error counters
pub struct Properties {
    info: &'static Info,
    state: &'static State,
}

impl Properties {
    fn new(state: &'static State) -> Self {
        Self {
            info: state.info,
            state,
        }
    }

    pub fn alloc_filter<I: IdType>(&self) -> Option<FilterSlot<I>> {
        I::alloc_slot(self.state).map(|slot_idx| FilterSlot {
            _phantom: PhantomData,
            state: self.state,
            slot_idx,
        })
    }

    pub fn alloc_standard_filter(&self) -> Option<FilterSlot<StandardId>> {
        self.alloc_filter()
    }

    pub fn alloc_extended_filter(&self) -> Option<FilterSlot<ExtendedId>> {
        self.alloc_filter()
    }

    /// Get the CAN RX error counter
    pub fn rx_error_count(&self) -> u8 {
        self.info.low.regs.ecr().read().rec()
    }

    /// Get the CAN TX error counter
    pub fn tx_error_count(&self) -> u8 {
        self.info.low.regs.ecr().read().tec()
    }

    // /// Get the current bus error mode
    // pub fn bus_error_mode(&self) -> BusErrorMode {
    //     // This read will clear LEC and DLEC. This is not ideal, but protocol
    //     // error reporting in this driver should have a big ol' FIXME on it
    //     // anyway!
    //     let psr = self.info.low.regs.psr().read();
    //     match (psr.bo(), psr.ep()) {
    //         (false, false) => BusErrorMode::ErrorActive,
    //         (false, true) => BusErrorMode::ErrorPassive,
    //         (true, _) => BusErrorMode::BusOff,
    //     }
    // }
}
