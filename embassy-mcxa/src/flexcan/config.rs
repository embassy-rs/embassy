use crate::flexcan::can::Info;
use core::marker::PhantomData;
use embassy_time::Duration;
use nxp_pac::can as pac;

/// Allows you to configure the CAN peripheral.
#[allow(dead_code)]
pub(in crate::flexcan) struct Config<'a> {
    info: &'static Info,
    _lt: PhantomData<&'a mut ()>,
}

// u_Note: default MCR for CAN0: 11011000100100000000000000001111
// u_Note: default MCR for CAN1: 11011000100100000000010000001111

impl<'a> Config<'a> {
    pub(in crate::flexcan) fn new(info: &'static Info) -> Self {
        Self { info, _lt: PhantomData }
    }

    /// Sets the number of message buffers.
    /// The hardware deafults to having 16 message buffers (see page 1466 of the datasheet).
    pub(in crate::flexcan) fn set_number_of_message_buffers(&mut self, num: u8) {
        // We do -1 here since the register value is technically the "Number of the Last Message Buffer".
        // So, if this register stores `15`, for example, you'll have 16 message buffers.
        self.info.regs.mcr().modify(|m| m.set_maxmb(num - 1));
    }

    /// Takes the FlexCAN out of Disable (low-power) mode.
    /// WARNING: This function is blocking! It doesn't return until the hardware confirms that the module has left low-power mode.
    /// Pass `None` as the timeout to wait indefinitely.
    pub(in crate::flexcan) fn enable(&mut self, timeout: Option<Duration>) -> Result<(), ConfigError> {
        use embassy_time::Instant;

        // Request module enable by clearing MCR[MDIS].
        self.info.regs.mcr().modify(|m| m.set_mdis(pac::Mdis::FlexcanEnabled));

        // Busy-wait for the low-power-mode acknowledge to clear.
        let deadline = timeout.map(|t| Instant::now() + t);
        while self.info.regs.mcr().read().lpmack() != pac::Lpmack::LowPowerNo {
            if let Some(deadline) = deadline {
                if Instant::now() >= deadline {
                    return Err(ConfigError::EnableTimeout);
                }
            }
        }

        Ok(())
    }

    /// Puts the FlexCAN into Freeze mode.
    /// WARNING: This function is blocking! It doesn't return until the hardware confirms that we have entered freeze mode. 
    /// Pass `None` as the timeout to wait indefinitely.
    pub(in crate::flexcan) fn freeze(&mut self, timeout: Option<Duration>) -> Result<(), ConfigError> {
        use embassy_time::Instant;

        // Request Freeze via MCR[FRZ]=1 and MCR[HALT]=1
        self.info.regs.mcr().modify(|m| {
            m.set_frz(pac::Frz::FreezeModeEnabled);
            m.set_halt(pac::Halt::HaltEnable);
        });

        // Busy-wait for the freeze acknowledge
        let deadline = timeout.map(|t| Instant::now() + t);
        while self.info.regs.mcr().read().frzack() != pac::Frzack::FreezeModeYes {
            if let Some(deadline) = deadline {
                if Instant::now() >= deadline {
                    return Err(ConfigError::FreezeTimeout);
                }
            }
        }

        Ok(())
    }

    /// Takes the FlexCAN out of Freeze mode.
    pub(in crate::flexcan) fn unfreeze(&mut self) {
        // Need to set MCR[HALT]=0 and MCR[FRZ]=0
        self.info.regs.mcr().modify(|m| { 
            m.set_halt(pac::Halt::HaltDisable); 
            m.set_frz(pac::Frz::FreezeModeDisabled);
        })
    }
}

/// Errors that can occur when configuring stuff.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(in crate::flexcan) enum ConfigError {
    /// The hardware did not assert `MCR[FRZACK]` within the requested time bound.
    FreezeTimeout,

    /// The hardware did not clear `MCR[LPMACK]` within the requested time bound.
    EnableTimeout,
}