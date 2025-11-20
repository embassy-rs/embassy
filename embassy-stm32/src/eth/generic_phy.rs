//! Generic SMI Ethernet PHY

use core::task::Context;

#[cfg(feature = "time")]
use embassy_time::{Duration, Timer};
#[cfg(feature = "time")]
use futures_util::FutureExt;

use super::{Phy, StationManagement};
use crate::block_for_us as blocking_delay_us;

#[allow(dead_code)]
mod phy_consts {
    pub const PHY_REG_BCR: u8 = 0x00;
    pub const PHY_REG_BSR: u8 = 0x01;
    pub const PHY_REG_ID1: u8 = 0x02;
    pub const PHY_REG_ID2: u8 = 0x03;
    pub const PHY_REG_ANTX: u8 = 0x04;
    pub const PHY_REG_ANRX: u8 = 0x05;
    pub const PHY_REG_ANEXP: u8 = 0x06;
    pub const PHY_REG_ANNPTX: u8 = 0x07;
    pub const PHY_REG_ANNPRX: u8 = 0x08;
    pub const PHY_REG_CTL: u8 = 0x0D; // Ethernet PHY Register Control
    pub const PHY_REG_ADDAR: u8 = 0x0E; // Ethernet PHY Address or Data

    pub const PHY_REG_WUCSR: u16 = 0x8010;

    pub const PHY_REG_BCR_COLTEST: u16 = 1 << 7;
    pub const PHY_REG_BCR_FD: u16 = 1 << 8;
    pub const PHY_REG_BCR_ANRST: u16 = 1 << 9;
    pub const PHY_REG_BCR_ISOLATE: u16 = 1 << 10;
    pub const PHY_REG_BCR_POWERDN: u16 = 1 << 11;
    pub const PHY_REG_BCR_AN: u16 = 1 << 12;
    pub const PHY_REG_BCR_100M: u16 = 1 << 13;
    pub const PHY_REG_BCR_LOOPBACK: u16 = 1 << 14;
    pub const PHY_REG_BCR_RESET: u16 = 1 << 15;

    pub const PHY_REG_BSR_JABBER: u16 = 1 << 1;
    pub const PHY_REG_BSR_UP: u16 = 1 << 2;
    pub const PHY_REG_BSR_FAULT: u16 = 1 << 4;
    pub const PHY_REG_BSR_ANDONE: u16 = 1 << 5;
}
use self::phy_consts::*;

/// Generic SMI Ethernet PHY implementation
pub struct GenericPhy<SM: StationManagement> {
    phy_addr: u8,
    sm: SM,
    #[cfg(feature = "time")]
    poll_interval: Duration,
}

impl<SM: StationManagement> GenericPhy<SM> {
    /// Construct the PHY. It assumes the address `phy_addr` in the SMI communication
    ///
    /// # Panics
    /// `phy_addr` must be in range `0..32`
    pub fn new(sm: SM, phy_addr: u8) -> Self {
        assert!(phy_addr < 32);
        Self {
            phy_addr,
            sm,
            #[cfg(feature = "time")]
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Construct the PHY. Try to probe all addresses from 0 to 31 during initialization
    ///
    /// # Panics
    /// Initialization panics if PHY didn't respond on any address
    pub fn new_auto(sm: SM) -> Self {
        Self {
            sm,
            phy_addr: 0xFF,
            #[cfg(feature = "time")]
            poll_interval: Duration::from_millis(500),
        }
    }
}

impl<SM: StationManagement> Phy for GenericPhy<SM> {
    fn phy_reset(&mut self) {
        // Detect SMI address
        if self.phy_addr == 0xFF {
            for addr in 0..32 {
                self.sm.smi_write(addr, PHY_REG_BCR, PHY_REG_BCR_RESET);
                for _ in 0..10 {
                    if self.sm.smi_read(addr, PHY_REG_BCR) & PHY_REG_BCR_RESET != PHY_REG_BCR_RESET {
                        trace!("Found ETH PHY on address {}", addr);
                        self.phy_addr = addr;
                        return;
                    }
                    // Give PHY a total of 100ms to respond
                    blocking_delay_us(10000);
                }
            }
            panic!("PHY did not respond");
        }

        self.sm.smi_write(self.phy_addr, PHY_REG_BCR, PHY_REG_BCR_RESET);
        while self.sm.smi_read(self.phy_addr, PHY_REG_BCR) & PHY_REG_BCR_RESET == PHY_REG_BCR_RESET {}
    }

    fn phy_init(&mut self) {
        // Clear WU CSR
        self.smi_write_ext(PHY_REG_WUCSR, 0);

        // Enable auto-negotiation
        self.sm.smi_write(
            self.phy_addr,
            PHY_REG_BCR,
            PHY_REG_BCR_AN | PHY_REG_BCR_ANRST | PHY_REG_BCR_100M,
        );
    }

    fn poll_link(&mut self, cx: &mut Context) -> bool {
        #[cfg(not(feature = "time"))]
        cx.waker().wake_by_ref();

        #[cfg(feature = "time")]
        let _ = Timer::after(self.poll_interval).poll_unpin(cx);

        let bsr = self.sm.smi_read(self.phy_addr, PHY_REG_BSR);

        // No link without autonegotiate
        if bsr & PHY_REG_BSR_ANDONE == 0 {
            return false;
        }
        // No link if link is down
        if bsr & PHY_REG_BSR_UP == 0 {
            return false;
        }

        // Got link
        true
    }
}

/// Public functions for the PHY
impl<SM: StationManagement> GenericPhy<SM> {
    /// Set the SMI polling interval.
    #[cfg(feature = "time")]
    pub fn set_poll_interval(&mut self, poll_interval: Duration) {
        self.poll_interval = poll_interval
    }

    // Writes a value to an extended PHY register in MMD address space
    fn smi_write_ext(&mut self, reg_addr: u16, reg_data: u16) {
        self.sm.smi_write(self.phy_addr, PHY_REG_CTL, 0x0003); // set address
        self.sm.smi_write(self.phy_addr, PHY_REG_ADDAR, reg_addr);
        self.sm.smi_write(self.phy_addr, PHY_REG_CTL, 0x4003); // set data
        self.sm.smi_write(self.phy_addr, PHY_REG_ADDAR, reg_data);
    }

    /// Access the underlying station management.
    pub fn station_management(&mut self) -> &mut SM {
        &mut self.sm
    }
}
