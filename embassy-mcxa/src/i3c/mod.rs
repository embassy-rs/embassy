//! I3C Support

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::I3cConfig;
use crate::dma::{DmaChannel, DmaRequest};
use crate::gpio::GpioPin;
use crate::{interrupt, pac};

pub mod controller;
pub mod target;

/// Device Characteristics Register (DCR) value advertised to the controller
/// via GETDCR and during ENTDAA.
///
/// Device-type codes are assigned by MIPI; see the current
/// [MIPI I3C DCR table](https://www.mipi.org/hubfs/I3C-Public-Tables/MIPI-I3C-v1-1-Current-DCR-Table.pdf).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DeviceCharacteristics {
    /// Generic device, DCR `0x00`.
    #[default]
    Generic,
    /// Heart-rate sensor, DCR `0x02`.
    HeartRateSensor,
    /// ECG sensor, DCR `0x03`.
    EcgSensor,
    /// EKG sensor, DCR `0x04`.
    EkgSensor,
    /// Galvanic skin response, DCR `0x05`.
    GalvanicSkinResponse,
    /// Breathalyzer, DCR `0x06`.
    Breathalyzer,
    /// Blood glucose, DCR `0x07`.
    BloodGlucose,
    /// Blood oxygenation, DCR `0x08`.
    BloodOxygenation,
    /// Electro neurography sensor, DCR `0x09`.
    EngsSensor,
    /// Touch, DCR `0x21`.
    Touch,
    /// Touch-less gesture, DCR `0x22`.
    GestureTouchLess,
    /// Grip, DCR `0x23`.
    Grip,
    /// Fingerprint, DCR `0x24`.
    Fingerprint,
    /// Haptic, DCR `0x25`.
    Haptic,
    /// Acoustic/ultrasonic gesture, DCR `0x26`.
    GestureAcousticUltrasonic,
    /// Audio alarm, DCR `0x27`.
    AudioAlarm,
    /// Human interface device, DCR `0x28`.
    Hid,
    /// Accelerometer, DCR `0x41`.
    Accelerometer,
    /// Gyroscope, DCR `0x42`.
    Gyroscope,
    /// Magnetometer, DCR `0x43`.
    Magnetometer,
    /// Accelerometer + gyroscope combo, DCR `0x44`.
    AccelGyroCombo,
    /// Accelerometer + magnetometer combo, DCR `0x45`.
    AccelMagCombo,
    /// Accelerometer + gyroscope + magnetometer combo, DCR `0x46`.
    AccelGyroMagCombo,
    /// Ambient light, DCR `0x61`.
    AmbientLight,
    /// Pressure, DCR `0x62`.
    Pressure,
    /// Temperature, DCR `0x63`.
    Temperature,
    /// Humidity, DCR `0x64`.
    Humidity,
    /// UV sensor, DCR `0x65`.
    UvSensor,
    /// Air quality, DCR `0x66`.
    AirQuality,
    /// IR sensor, DCR `0x67`.
    IrSensor,
    /// Proximity, DCR `0x81`.
    Proximity,
    /// RGB, DCR `0x82`.
    Rgb,
    /// Accelerometer (mechanical shock), DCR `0x83`.
    AccelerometerMechanicalShock,
    /// Oxygen sensor, DCR `0x84`.
    OxygenSensor,
    /// Mass-flow sensor, DCR `0x85`.
    MassFlowSensor,
    /// Switch/solenoid/valve control, DCR `0x86`.
    SwitchSolenoidValveControl,
    /// Goniometer, DCR `0x87`.
    Goniometer,
    /// Position sensor, DCR `0x88`.
    PositionSensor,
    /// Throttle control, DCR `0x89`.
    ThrottleControl,
    /// Force/stress sensor, DCR `0x8A`.
    ForceStressSensor,
    /// NFC, DCR `0xA1`.
    Nfc,
    /// IR data link, DCR `0xA2`.
    IrDataLink,
    /// RF data link, DCR `0xA3`.
    RfDataLink,
    /// RF link ranging/localization, DCR `0xA4`.
    RfLinkRangingLocalization,
    /// ETSI SSP security device, DCR `0xBA`.
    SecurityDeviceEtsiSsp,
    /// ETSI UICC security device, DCR `0xBB`.
    SecurityDeviceEtsiUicc,
    /// eSE security device, DCR `0xBC`.
    SecurityDeviceEse,
    /// OCP recovery, DCR `0xBD`.
    OcpRecovery,
    /// Bridge, DCR `0xC1`.
    Bridge,
    /// Hub, DCR `0xC2`.
    Hub,
    /// Bus monitor, DCR `0xC3`.
    BusMonitor,
    /// Secondary controller, DCR `0xC4`.
    SecondaryMaster,
    /// Memory, DCR `0xC5`.
    Memory,
    /// Microcontroller, DCR `0xC6`.
    Microcontroller,
    /// PMIC, DCR `0xC7`.
    Pmic,
    /// I/O expander, DCR `0xC8`.
    IoExpander,
    /// Debug target system, DCR `0xC9`.
    DebugTargetSystem,
    /// Debug and test system, DCR `0xCA`.
    DebugAndTestSystem,
    /// Dual-role debug system, DCR `0xCB`.
    DualRoleDebugSystem,
    /// MCTP, DCR `0xCC`.
    Mctp,
    /// Retimer, DCR `0xCD`.
    Retimer,
    /// Thermal sensor (first), DCR `0xD2`.
    ThermalSensorFirst,
    /// Differential DIMM memory, first buffer, DCR `0xD4`.
    DifferentialDimmMemoryFirstBuffer,
    /// Differential DIMM memory, second buffer, DCR `0xD5`.
    DifferentialDimmMemorySecondBuffer,
    /// Thermal sensor (second), DCR `0xD6`.
    ThermalSensorSecond,
    /// PMIC 2, DCR `0xD8`.
    Pmic2,
    /// PMIC 1, DCR `0xD9`.
    Pmic1,
    /// SPD hub, DCR `0xDA`.
    SpdHub,
    /// Registered clock divider, DCR `0xDB`.
    RegisteredClockDivider,
    /// PMIC 3, DCR `0xDC`.
    Pmic3,
    /// FPGA/PLD configuration, DCR `0xE1`.
    FpgaPldConfiguration,
    /// Camera photometer, DCR `0xE2`.
    CameraPhotometer,
    /// Camera shutter control, DCR `0xE3`.
    CameraShutterControl,
    /// Camera focus control, DCR `0xE4`.
    CameraFocusControl,
}

impl From<DeviceCharacteristics> for u8 {
    fn from(value: DeviceCharacteristics) -> Self {
        match value {
            DeviceCharacteristics::Generic => 0x00,
            DeviceCharacteristics::HeartRateSensor => 0x02,
            DeviceCharacteristics::EcgSensor => 0x03,
            DeviceCharacteristics::EkgSensor => 0x04,
            DeviceCharacteristics::GalvanicSkinResponse => 0x05,
            DeviceCharacteristics::Breathalyzer => 0x06,
            DeviceCharacteristics::BloodGlucose => 0x07,
            DeviceCharacteristics::BloodOxygenation => 0x08,
            DeviceCharacteristics::EngsSensor => 0x09,
            DeviceCharacteristics::Touch => 0x21,
            DeviceCharacteristics::GestureTouchLess => 0x22,
            DeviceCharacteristics::Grip => 0x23,
            DeviceCharacteristics::Fingerprint => 0x24,
            DeviceCharacteristics::Haptic => 0x25,
            DeviceCharacteristics::GestureAcousticUltrasonic => 0x26,
            DeviceCharacteristics::AudioAlarm => 0x27,
            DeviceCharacteristics::Hid => 0x28,
            DeviceCharacteristics::Accelerometer => 0x41,
            DeviceCharacteristics::Gyroscope => 0x42,
            DeviceCharacteristics::Magnetometer => 0x43,
            DeviceCharacteristics::AccelGyroCombo => 0x44,
            DeviceCharacteristics::AccelMagCombo => 0x45,
            DeviceCharacteristics::AccelGyroMagCombo => 0x46,
            DeviceCharacteristics::AmbientLight => 0x61,
            DeviceCharacteristics::Pressure => 0x62,
            DeviceCharacteristics::Temperature => 0x63,
            DeviceCharacteristics::Humidity => 0x64,
            DeviceCharacteristics::UvSensor => 0x65,
            DeviceCharacteristics::AirQuality => 0x66,
            DeviceCharacteristics::IrSensor => 0x67,
            DeviceCharacteristics::Proximity => 0x81,
            DeviceCharacteristics::Rgb => 0x82,
            DeviceCharacteristics::AccelerometerMechanicalShock => 0x83,
            DeviceCharacteristics::OxygenSensor => 0x84,
            DeviceCharacteristics::MassFlowSensor => 0x85,
            DeviceCharacteristics::SwitchSolenoidValveControl => 0x86,
            DeviceCharacteristics::Goniometer => 0x87,
            DeviceCharacteristics::PositionSensor => 0x88,
            DeviceCharacteristics::ThrottleControl => 0x89,
            DeviceCharacteristics::ForceStressSensor => 0x8A,
            DeviceCharacteristics::Nfc => 0xA1,
            DeviceCharacteristics::IrDataLink => 0xA2,
            DeviceCharacteristics::RfDataLink => 0xA3,
            DeviceCharacteristics::RfLinkRangingLocalization => 0xA4,
            DeviceCharacteristics::SecurityDeviceEtsiSsp => 0xBA,
            DeviceCharacteristics::SecurityDeviceEtsiUicc => 0xBB,
            DeviceCharacteristics::SecurityDeviceEse => 0xBC,
            DeviceCharacteristics::OcpRecovery => 0xBD,
            DeviceCharacteristics::Bridge => 0xC1,
            DeviceCharacteristics::Hub => 0xC2,
            DeviceCharacteristics::BusMonitor => 0xC3,
            DeviceCharacteristics::SecondaryMaster => 0xC4,
            DeviceCharacteristics::Memory => 0xC5,
            DeviceCharacteristics::Microcontroller => 0xC6,
            DeviceCharacteristics::Pmic => 0xC7,
            DeviceCharacteristics::IoExpander => 0xC8,
            DeviceCharacteristics::DebugTargetSystem => 0xC9,
            DeviceCharacteristics::DebugAndTestSystem => 0xCA,
            DeviceCharacteristics::DualRoleDebugSystem => 0xCB,
            DeviceCharacteristics::Mctp => 0xCC,
            DeviceCharacteristics::Retimer => 0xCD,
            DeviceCharacteristics::ThermalSensorFirst => 0xD2,
            DeviceCharacteristics::DifferentialDimmMemoryFirstBuffer => 0xD4,
            DeviceCharacteristics::DifferentialDimmMemorySecondBuffer => 0xD5,
            DeviceCharacteristics::ThermalSensorSecond => 0xD6,
            DeviceCharacteristics::Pmic2 => 0xD8,
            DeviceCharacteristics::Pmic1 => 0xD9,
            DeviceCharacteristics::SpdHub => 0xDA,
            DeviceCharacteristics::RegisteredClockDivider => 0xDB,
            DeviceCharacteristics::Pmic3 => 0xDC,
            DeviceCharacteristics::FpgaPldConfiguration => 0xE1,
            DeviceCharacteristics::CameraPhotometer => 0xE2,
            DeviceCharacteristics::CameraShutterControl => 0xE3,
            DeviceCharacteristics::CameraFocusControl => 0xE4,
        }
    }
}

pub(crate) mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = I3cConfig> {
    fn info() -> &'static Info;

    const CLOCK_INSTANCE: crate::clocks::periph_helpers::I3cInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
    const TX_DMA_REQUEST: DmaRequest;
    const RX_DMA_REQUEST: DmaRequest;

    fn bbq_state() -> &'static crate::i3c::target::BbqState;

    /// DMA-completion callback for the per-instance RX DMA channel.
    /// Pends the I3C interrupt so the IRQ rotates BBQ grants.
    fn dma_rx_complete_cb();
}

/// I3C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this I3C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pub(crate) struct Info {
    pub(crate) regs: pac::i3c::I3c,
    pub(crate) wait_cell: WaitCell,
    pub(crate) enable_interrupt: fn(),
    pub(crate) disable_interrupt: fn(),
    pub(crate) reset_peripheral: unsafe fn(),
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::i3c::I3c {
        self.regs
    }

    pub(crate) fn enable_interrupt<T: Instance>() {
        use crate::interrupt::typelevel::Interrupt;

        T::Interrupt::unpend();
        // SAFETY: The driver owns the instance and installed its binding.
        unsafe { T::Interrupt::enable() };
    }

    pub(crate) fn disable_interrupt<T: Instance>() {
        use crate::interrupt::typelevel::Interrupt;

        T::Interrupt::disable();
        T::Interrupt::unpend();
    }

    /// Pulse the MRCC reset.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own the I3C instance, stop all peripheral
    /// and DMA activity, and ensure SCL/SDA are inactive during reset release.
    pub(crate) unsafe fn reset_peripheral<T: Instance>() {
        unsafe { crate::clocks::pulse_reset::<T>() };
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i3c_instance {
    ($n:literal) => {
        paste::paste! {
            impl $crate::i3c::SealedInstance for $crate::peripherals::[<I3C $n>] {
                fn info() -> &'static $crate::i3c::Info {
                    static INFO: $crate::i3c::Info = $crate::i3c::Info {
                        regs: $crate::pac::[<I3C $n>],
                        wait_cell: maitake_sync::WaitCell::new(),
                        enable_interrupt: crate::i3c::Info::enable_interrupt::<crate::peripherals::[<I3C $n>]>,
                        disable_interrupt: crate::i3c::Info::disable_interrupt::<crate::peripherals::[<I3C $n>]>,
                        reset_peripheral: crate::i3c::Info::reset_peripheral::<crate::peripherals::[<I3C $n>]>,
                    };
                    &INFO
                }

                const TX_DMA_REQUEST: DmaRequest = DmaRequest::[<I3C $n Tx>];
                const RX_DMA_REQUEST: DmaRequest = DmaRequest::[<I3C $n Rx>];
                const CLOCK_INSTANCE: $crate::clocks::periph_helpers::I3cInstance = $crate::clocks::periph_helpers::I3cInstance::[<I3c $n>];
                const PERF_INT_INCR: fn() = $crate::perf_counters::[<incr_interrupt_i3c $n>];
                const PERF_INT_WAKE_INCR: fn() = $crate::perf_counters::[<incr_interrupt_i3c $n _wake>];

                fn bbq_state() -> &'static $crate::i3c::target::BbqState {
                    static STATE: $crate::i3c::target::BbqState = $crate::i3c::target::BbqState::new();
                    &STATE
                }

                fn dma_rx_complete_cb() {
                    use $crate::_generated::interrupt::typelevel::Interrupt;
                    use core::sync::atomic::Ordering;
                    Self::bbq_state()
                        .state
                        .fetch_or($crate::i3c::target::STATE_RXDMA_COMPLETE, Ordering::AcqRel);
                    <Self as $crate::i3c::Instance>::Interrupt::pend();
                }
            }

            impl $crate::i3c::Instance for $crate::peripherals::[<I3C $n>] {
                type Interrupt = $crate::interrupt::typelevel::[<I3C $n>];
            }
        }
    };
}

/// SCL pin trait.
pub trait SclPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA1 pin (for I3C multi-lane) trait.
pub trait Sda1Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA2 pin (for I3C multi-lane) trait.
pub trait Sda2Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA3 pin (for I3C multi-lane) trait.
pub trait Sda3Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// PUR pin trait. (Pull up resistance)
pub trait PurPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Async driver mode.
#[allow(private_bounds)]
pub trait AsyncMode: sealed::Sealed + Mode {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}
impl AsyncMode for Async {}

/// DMA mode.
pub struct Dma<'d> {
    tx_dma: DmaChannel<'d>,
    tx_request: DmaRequest,

    rx_dma: DmaChannel<'d>,
    rx_request: DmaRequest,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}
impl AsyncMode for Dma<'_> {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i3c_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        paste::paste! {
            impl $crate::i3c::sealed::Sealed for $crate::peripherals::$pin {}

            impl $crate::i3c::$trait<$crate::peripherals::$peri> for $crate::peripherals::$pin {
                fn mux(&self) {
                    use $crate::gpio::SealedPin;
                    self.set_pull($crate::gpio::Pull::Disabled);
                    self.set_slew_rate($crate::gpio::SlewRate::Fast.into());
                    self.set_drive_strength($crate::gpio::DriveStrength::Normal.into());
                    self.set_function($crate::pac::port::Mux::$fn);
                    self.set_enable_input_buffer(true);
                }
            }
        }
    };
}
