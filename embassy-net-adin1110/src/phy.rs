use crate::mdio::MdioBus;

#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(u8)]
/// Clause 22 Registers
pub enum RegsC22 {
    /// MII Control Register
    CONTROL = 0x00,
    /// MII Status Register
    STATUS = 0x01,
    /// PHY Identifier 1 Register
    PHY_ID1 = 0x02,
    /// PHY Identifier 2 Register.
    PHY_ID2 = 0x03,
}

/// Clause 45 Registers
#[allow(non_snake_case, dead_code)]
pub mod RegsC45 {
    /// Device Address: 0x01
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    #[repr(u16)]
    pub enum DA1 {
        /// PMA/PMD Control 1 Register
        PMA_PMD_CNTRL1 = 0x0000,
        /// PMA/PMD Status 1 Register
        PMA_PMD_STAT1 = 0x0001,
        /// MSE Value Register
        MSE_VAL = 0x830B,
    }

    impl DA1 {
        #[must_use]
        pub fn into(self) -> (u8, u16) {
            (0x01, self as u16)
        }
    }

    /// Device Address: 0x03
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    #[repr(u16)]
    pub enum DA3 {
        /// PCS Control 1 Register
        PCS_CNTRL1 = 0x0000,
        /// PCS Status 1 Register
        PCS_STAT1 = 0x0001,
        /// PCS Status 2 Register
        PCS_STAT2 = 0x0008,
    }

    impl DA3 {
        #[must_use]
        pub fn into(self) -> (u8, u16) {
            (0x03, self as u16)
        }
    }

    /// Device Address: 0x07
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    #[repr(u16)]
    pub enum DA7 {
        /// Extra Autonegotiation Status Register
        AN_STATUS_EXTRA = 0x8001,
    }

    impl DA7 {
        #[must_use]
        pub fn into(self) -> (u8, u16) {
            (0x07, self as u16)
        }
    }

    /// Device Address: 0x1E
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    #[repr(u16)]
    pub enum DA1E {
        /// System Interrupt Status Register
        CRSM_IRQ_STATUS = 0x0010,
        /// System Interrupt Mask Register
        CRSM_IRQ_MASK = 0x0020,
        /// Pin Mux Configuration 1 Register
        DIGIO_PINMUX = 0x8c56,
        /// LED Control Register.
        LED_CNTRL = 0x8C82,
        /// LED Polarity Register
        LED_POLARITY = 0x8C83,
    }

    impl DA1E {
        #[must_use]
        pub fn into(self) -> (u8, u16) {
            (0x1e, self as u16)
        }
    }

    /// Device Address: 0x1F
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    #[repr(u16)]
    pub enum DA1F {
        /// PHY Subsystem Interrupt Status Register
        PHY_SYBSYS_IRQ_STATUS = 0x0011,
        /// PHY Subsystem Interrupt Mask Register
        PHY_SYBSYS_IRQ_MASK = 0x0021,
    }

    impl DA1F {
        #[must_use]
        pub fn into(self) -> (u8, u16) {
            (0x1f, self as u16)
        }
    }
}

/// 10-BASE-T1x PHY functions.
pub struct Phy10BaseT1x(u8);

impl Default for Phy10BaseT1x {
    fn default() -> Self {
        Self(0x01)
    }
}

impl Phy10BaseT1x {
    /// Get the both parts of the PHYID.
    pub async fn get_id<MDIOBUS, MDE>(&self, mdiobus: &mut MDIOBUS) -> Result<u32, MDE>
    where
        MDIOBUS: MdioBus<Error = MDE>,
        MDE: core::fmt::Debug,
    {
        let mut phyid = u32::from(mdiobus.read_cl22(self.0, RegsC22::PHY_ID1 as u8).await?) << 16;
        phyid |= u32::from(mdiobus.read_cl22(self.0, RegsC22::PHY_ID2 as u8).await?);
        Ok(phyid)
    }

    /// Get the Mean Squared Error Value.
    pub async fn get_sqi<MDIOBUS, MDE>(&self, mdiobus: &mut MDIOBUS) -> Result<u16, MDE>
    where
        MDIOBUS: MdioBus<Error = MDE>,
        MDE: core::fmt::Debug,
    {
        mdiobus.read_cl45(self.0, RegsC45::DA1::MSE_VAL.into()).await
    }
}
