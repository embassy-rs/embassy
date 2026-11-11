//! Radio Frequency (RF) front-end switch abstractions.
//!
//! Sub-GHz radio devices require an external RF switch to direct the RF path
//! between receive (`RFI`) and transmit (`RFO_LP` or `RFO_HP`).

use crate::gpio::Output;

/// Operating mode of the RF front-end switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RfMode {
    /// RF switch disconnected/disabled (lowest power).
    Off,
    /// RF switch configured for receive path (RFI).
    Rx,
    /// RF switch configured for Low-Power transmit path (RFO_LP, up to +15 dBm).
    TxLowPower,
    /// RF switch configured for High-Power transmit path (RFO_HP, up to +22 dBm).
    TxHighPower,
}

/// Trait implemented by RF front-end switches.
pub trait RfSwitch {
    /// Configure the RF front-end switch to the given mode.
    fn set_mode(&mut self, mode: RfMode);

    /// Switch to receive mode.
    fn set_rx(&mut self) {
        self.set_mode(RfMode::Rx);
    }

    /// Switch to low-power transmit mode.
    fn set_tx_lp(&mut self) {
        self.set_mode(RfMode::TxLowPower);
    }

    /// Switch to high-power transmit mode.
    fn set_tx_hp(&mut self) {
        self.set_mode(RfMode::TxHighPower);
    }

    /// Disable the RF switch (power down).
    fn set_off(&mut self) {
        self.set_mode(RfMode::Off);
    }
}

/// RF switch driver for the ST Nucleo-WL55JC development board.
///
/// On Nucleo-WL55JC:
/// - `ctrl1`: PC4 (FE_CTRL1)
/// - `ctrl2`: PC5 (FE_CTRL2)
/// - `ctrl3`: PC3 (FE_CTRL3)
pub struct NucleoRfSwitch<'d> {
    ctrl1: Output<'d>,
    ctrl2: Output<'d>,
    ctrl3: Output<'d>,
}

impl<'d> NucleoRfSwitch<'d> {
    /// Create a new RF switch driver for Nucleo-WL55JC.
    pub fn new(ctrl1: Output<'d>, ctrl2: Output<'d>, ctrl3: Output<'d>) -> Self {
        let mut this = Self { ctrl1, ctrl2, ctrl3 };
        this.set_mode(RfMode::Off);
        this
    }
}

impl<'d> RfSwitch for NucleoRfSwitch<'d> {
    fn set_mode(&mut self, mode: RfMode) {
        match mode {
            RfMode::Off => {
                self.ctrl1.set_low();
                self.ctrl2.set_low();
                self.ctrl3.set_low();
            }
            RfMode::Rx => {
                self.ctrl1.set_high();
                self.ctrl2.set_low();
                self.ctrl3.set_high();
            }
            RfMode::TxLowPower => {
                self.ctrl1.set_high();
                self.ctrl2.set_high();
                self.ctrl3.set_high();
            }
            RfMode::TxHighPower => {
                self.ctrl1.set_low();
                self.ctrl2.set_high();
                self.ctrl3.set_high();
            }
        }
    }
}

/// RF switch driver for Seeed Studio LoRa-E5 / Grove LoRa-E5 / RAK3172 modules.
///
/// These modules commonly use two control lines:
/// - `ctrl1`: High for RX, Low for TX.
/// - `ctrl2`: Low for RX, High for TX.
pub struct LoraE5RfSwitch<'d> {
    ctrl1: Output<'d>,
    ctrl2: Output<'d>,
}

impl<'d> LoraE5RfSwitch<'d> {
    /// Create a new RF switch driver for LoRa-E5 / RAK3172.
    pub fn new(ctrl1: Output<'d>, ctrl2: Output<'d>) -> Self {
        let mut this = Self { ctrl1, ctrl2 };
        this.set_mode(RfMode::Off);
        this
    }
}

impl<'d> RfSwitch for LoraE5RfSwitch<'d> {
    fn set_mode(&mut self, mode: RfMode) {
        match mode {
            RfMode::Off => {
                self.ctrl1.set_low();
                self.ctrl2.set_low();
            }
            RfMode::Rx => {
                self.ctrl1.set_high();
                self.ctrl2.set_low();
            }
            RfMode::TxLowPower | RfMode::TxHighPower => {
                self.ctrl1.set_low();
                self.ctrl2.set_high();
            }
        }
    }
}

/// Dummy RF switch that performs no operations. Useful for hardware with automatic RF switches.
pub struct DummyRfSwitch;

impl RfSwitch for DummyRfSwitch {
    fn set_mode(&mut self, _mode: RfMode) {}
}
