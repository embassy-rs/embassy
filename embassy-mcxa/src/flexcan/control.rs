//! This module contains a bunch of random helpers for controlling/configuring the FlexCAN peripheral.

use embassy_time::Duration;
use nxp_pac::can as pac;

/// Contains a bunch of random helpers for controlling/configuring the FlexCAN peripheral.
#[allow(dead_code)]
pub(in crate::flexcan) struct Control {
    regs: pac::Can,
}

// u_Note: default MCR for CAN0: 11011000100100000000000000001111
// u_Note: default MCR for CAN1: 11011000100100000000010000001111

impl Control {
    pub(in crate::flexcan) const fn new(regs: pac::Can) -> Self {
        Self { regs }
    }

    /// Access the raw FlexCAN registers
    #[inline(always)]
    pub(in crate::flexcan) const fn regs(&self) -> pac::Can {
        self.regs
    }

    /// Sets the number of message buffers.
    /// The hardware deafults to having 16 message buffers (see page 1466 of the datasheet).
    pub(in crate::flexcan) fn set_number_of_message_buffers(&self, num: u8) {
        // We do -1 here since the register value is technically the "Number of the Last Message Buffer".
        // So, if this register stores `15`, for example, you'll have 16 message buffers.
        self.regs.mcr().modify(|m| m.set_maxmb(num - 1));
    }

    /// Takes the FlexCAN out of Disable (low-power) mode.
    /// 
    /// WARNING: This function is blocking! It doesn't return until the hardware confirms that the module has left low-power mode.
    /// Pass `None` as the timeout to wait indefinitely.
    pub(in crate::flexcan) fn enable(&self, timeout: Option<Duration>) -> Result<(), ControlError> {
        use embassy_time::Instant;

        // Request module enable by clearing MCR[MDIS].
        self.regs.mcr().modify(|m| m.set_mdis(pac::Mdis::FlexcanEnabled));

        // Busy-wait for the low-power-mode acknowledge to clear.
        let deadline = timeout.map(|t| Instant::now() + t);
        while self.regs.mcr().read().lpmack() != pac::Lpmack::LowPowerNo {
            if let Some(deadline) = deadline {
                if Instant::now() >= deadline {
                    return Err(ControlError::EnableTimeout);
                }
            }
        }

        Ok(())
    }

    /// Puts the FlexCAN into Freeze mode. If the FlexCAN is already in Freeze mode, returns Ok(()).
    /// 
    /// WARNING: This function is blocking! It doesn't return until the hardware confirms that we have entered freeze mode. 
    /// Pass `None` as the timeout to wait indefinitely.
    pub(in crate::flexcan) fn freeze(&self, timeout: Option<Duration>) -> Result<(), ControlError> {
        use embassy_time::Instant;

        // If we're already frozen, don't need to do anything.
        if self.is_frozen() {
            return Ok(());
        }

        // Request Freeze via MCR[FRZ]=1 and MCR[HALT]=1
        self.regs.mcr().modify(|m| {
            m.set_frz(pac::Frz::FreezeModeEnabled);
            m.set_halt(pac::Halt::HaltEnable);
        });

        // Busy-wait for the freeze acknowledge
        let deadline = timeout.map(|t| Instant::now() + t);
        while !self.is_frozen() {
            if let Some(deadline) = deadline {
                if Instant::now() >= deadline {
                    return Err(ControlError::FreezeTimeout);
                }
            }
        }

        Ok(())
    }

    /// Takes the FlexCAN out of Freeze mode.
    pub(in crate::flexcan) fn unfreeze(&self) {
        // Need to set MCR[HALT]=0 and MCR[FRZ]=0
        self.regs.mcr().modify(|m| { 
            m.set_halt(pac::Halt::HaltDisable); 
            m.set_frz(pac::Frz::FreezeModeDisabled);
        })
    }

    /// Checks whether or not FlexCAN is actively in Freeze mode.
    pub(in crate::flexcan) fn is_frozen(&self) -> bool {
        let mcr = self.regs.mcr().read();
        (mcr.halt() == pac::Halt::HaltEnable) && (mcr.frz() == pac::Frz::FreezeModeEnabled) && (mcr.frzack() == pac::Frzack::FreezeModeYes)
    }
}

/// Errors that can occur when controlling/configuring the FlexCAN,
/// ususally during init-time or when modifying its core operating mode(s).
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControlError {
    /// The hardware did not assert `MCR[FRZACK]` within the requested time bound.
    FreezeTimeout,

    /// The hardware did not clear `MCR[LPMACK]` within the requested time bound.
    EnableTimeout,
}