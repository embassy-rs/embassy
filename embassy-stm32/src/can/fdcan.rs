// Driver implementation for the FDCAN peripheral in STM32 MCUs.

// The FDCAN peripheral appears to be licensed IP from Bosch,
// named the M_CAN protocol controller.
// https://www.bosch-semiconductors.com/ip-modules/can-ip-modules/m-can/

pub(crate) mod fd;

use super::enums::*;
use super::frame::*;

pub use self::fd::{config, filter};
pub use super::common::{BufferedCanReceiver, BufferedCanSender};
pub use fd::configurator::CanConfigurator;
pub use fd::interrupt::{IT0InterruptHandler, IT1InterruptHandler};
pub use fd::peripheral::*;
pub use fd::{Can, RxFifo};

/// Timestamp for incoming packets. Use Embassy time when enabled.
#[cfg(feature = "time")]
pub type Timestamp = embassy_time::Instant;

/// Timestamp for incoming packets.
#[cfg(not(feature = "time"))]
pub type Timestamp = u16;

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
