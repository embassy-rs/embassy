pub mod classic;
pub mod filter;
pub(crate) mod control;

use embassy_hal_internal::PeripheralType;
use crate::gpio::AnyPin;
use crate::interrupt::typelevel::Interrupt;

/// Shared, mode-agnostic peripheral identity. Each FlexCAN instance implements this
/// regardless of whether it is used in Classic or (future) FD mode.
pub trait Instance: PeripheralType + 'static + Send {
    type Interrupt: Interrupt;
}

pub(crate) mod sealed {
    /// Seal the pin traits so only the HAL can implement them.
    pub trait Sealed {}
}

// Every GPIO pin is allowed to be (potentially) sealed as a FlexCAN pin. The actual
// `TxPin`/`RxPin` impls (generated in the build script) restrict which concrete pins
// are valid for which CAN instance.
impl<T: crate::gpio::SealedPin> sealed::Sealed for T {}

/// CAN TX pin trait. Implemented (via codegen) for each pin that can be muxed to a
/// given FlexCAN instance's TXD function.
pub trait TxPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// The port mux setting that selects the TXD function for this pin.
    const MUX: crate::pac::port::Mux;
    /// Configure the pin for FlexCAN TXD usage.
    fn as_tx(&self);
}

/// CAN RX pin trait. Implemented (via codegen) for each pin that can be muxed to a
/// given FlexCAN instance's RXD function.
pub trait RxPin<T: Instance>: Into<AnyPin> + sealed::Sealed + PeripheralType {
    /// The port mux setting that selects the RXD function for this pin.
    const MUX: crate::pac::port::Mux;
    /// Configure the pin for FlexCAN RXD usage.
    fn as_rx(&self);
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_flexcan_pin {
    ($inst:ident, $pin:ident, $alt:ident, TXD) => {
        impl crate::flexcan::TxPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            const MUX: crate::pac::port::Mux = crate::pac::port::Mux::$alt;
            fn as_tx(&self) {
                use crate::gpio::SealedPin;
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
                self.set_function(<Self as crate::flexcan::TxPin<crate::peripherals::$inst>>::MUX);
                self.set_enable_input_buffer(false);
            }
        }
    };
    ($inst:ident, $pin:ident, $alt:ident, RXD) => {
        impl crate::flexcan::RxPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            const MUX: crate::pac::port::Mux = crate::pac::port::Mux::$alt;
            fn as_rx(&self) {
                use crate::gpio::SealedPin;
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_function(<Self as crate::flexcan::RxPin<crate::peripherals::$inst>>::MUX);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_can_instance {
    ($n:expr) => {
        paste::paste! {
            // mode-agnostic peripheral identity
            impl crate::flexcan::Instance for crate::peripherals::[<CAN $n>] {
                type Interrupt = crate::interrupt::typelevel::[<CAN $n>];
            }

            // stuff for classic CAN mode
            impl crate::flexcan::classic::SealedInstance for crate::peripherals::[<CAN $n>] {
                fn info() -> &'static crate::flexcan::classic::Info {
                    static INFO: crate::flexcan::classic::Info = crate::flexcan::classic::Info {
                        control: crate::flexcan::control::Control::new(crate::pac::[<CAN $n>]),
                        tx_available: core::sync::atomic::AtomicU32::new(0),
                        tx_remote: core::sync::atomic::AtomicU32::new(0),
                        tx_waker: embassy_sync::waitqueue::AtomicWaker::new(),
                        prexcen_supported: $n == 0, // Protocol Exception is only supported on CAN0.
                        rx_channel: embassy_sync::channel::Channel::new(),
                    };
                    &INFO
                }

                const CLOCK_INSTANCE: crate::clocks::periph_helpers::CanInstance
                    = crate::clocks::periph_helpers::CanInstance::[<Can $n>];
            }
            impl crate::flexcan::classic::Instance for crate::peripherals::[<CAN $n>] {}

            // u_TODO FDCAN mode: uncomment this block once a `fdcan` module exists alongside `classic`
            // impl crate::flexcan::fdcan::SealedInstance for crate::peripherals::[<CAN $n>] {
            //     fn info() -> &'static crate::flexcan::fdcan::Info {
            //         static INFO: crate::flexcan::fdcan::Info = crate::flexcan::fdcan::Info {
            //             control: crate::flexcan::control::Control::new(crate::pac::[<CAN $n>]),
            //             // other FDCAN stuff
            //         };
            //         &INFO
            //     }
            // }
            // impl crate::flexcan::fdcan::Instance for crate::peripherals::[<CAN $n>] {}
        }
    };
}

// u_TODO: I still need to edit the PAC (just on my fork for now) to add the gates
// according to Claude the changes would be somethnig like
// ```
// nxp-pac/src/chips/mcxa256/metadata.rs (CAN0 ~line 1130, CAN1 ~line 1216):


// gate: Some(Gate {
//     enable: "mrcc_glb_cc1",
//     reset: Some("mrcc_glb_rst1"),
//     config: Some("CanConfig"),
// }),
// nxp-pac/src/chips/mcxa577/metadata.rs (CAN0 ~line 1253, CAN1 ~line 1339) — note cc2/rst2 here, not cc1/rst1:


// gate: Some(Gate {
//     enable: "mrcc_glb_cc2",
//     reset: Some("mrcc_glb_rst2"),
//     config: Some("CanConfig"),
// }),
//```
// so probably look into that tomorrow