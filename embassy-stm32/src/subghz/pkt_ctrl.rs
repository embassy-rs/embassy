/// Generic packet infinite sequence selection.
///
/// Argument of [`PktCtrl::set_inf_seq_sel`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InfSeqSel {
    /// Preamble `0x5555`.
    Five = 0b00,
    /// Preamble `0x0000`.
    Zero = 0b01,
    /// Preamble `0xFFFF`.
    One = 0b10,
    /// PRBS9.
    Prbs9 = 0b11,
}

impl Default for InfSeqSel {
    fn default() -> Self {
        InfSeqSel::Five
    }
}

/// Generic packet control.
///
/// Argument of [`set_pkt_ctrl`](super::SubGhz::set_pkt_ctrl).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PktCtrl {
    val: u8,
}

impl PktCtrl {
    /// Reset value of the packet control register.
    pub const RESET: PktCtrl = PktCtrl { val: 0x21 };

    /// Create a new [`PktCtrl`] structure from a raw value.
    ///
    /// Reserved bits will be masked.
    pub const fn from_raw(raw: u8) -> Self {
        Self { val: raw & 0x3F }
    }

    /// Get the raw value of the [`PktCtrl`] register.
    pub const fn as_bits(&self) -> u8 {
        self.val
    }

    /// Generic packet synchronization word detection enable.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// const PKT_CTRL: PktCtrl = PktCtrl::RESET.set_sync_det_en(true);
    /// ```
    #[must_use = "set_sync_det_en returns a modified PktCtrl"]
    pub const fn set_sync_det_en(mut self, en: bool) -> PktCtrl {
        if en {
            self.val |= 1 << 5;
        } else {
            self.val &= !(1 << 5);
        }
        self
    }

    /// Returns `true` if generic packet synchronization word detection is
    /// enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// let pc: PktCtrl = PktCtrl::RESET;
    /// assert_eq!(pc.sync_det_en(), true);
    /// let pc: PktCtrl = pc.set_sync_det_en(false);
    /// assert_eq!(pc.sync_det_en(), false);
    /// let pc: PktCtrl = pc.set_sync_det_en(true);
    /// assert_eq!(pc.sync_det_en(), true);
    /// ```
    pub const fn sync_det_en(&self) -> bool {
        self.val & (1 << 5) != 0
    }

    /// Generic packet continuous transmit enable.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// const PKT_CTRL: PktCtrl = PktCtrl::RESET.set_cont_tx_en(true);
    /// ```
    #[must_use = "set_cont_tx_en returns a modified PktCtrl"]
    pub const fn set_cont_tx_en(mut self, en: bool) -> PktCtrl {
        if en {
            self.val |= 1 << 4;
        } else {
            self.val &= !(1 << 4);
        }
        self
    }

    /// Returns `true` if generic packet continuous transmit is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// let pc: PktCtrl = PktCtrl::RESET;
    /// assert_eq!(pc.cont_tx_en(), false);
    /// let pc: PktCtrl = pc.set_cont_tx_en(true);
    /// assert_eq!(pc.cont_tx_en(), true);
    /// let pc: PktCtrl = pc.set_cont_tx_en(false);
    /// assert_eq!(pc.cont_tx_en(), false);
    /// ```
    pub const fn cont_tx_en(&self) -> bool {
        self.val & (1 << 4) != 0
    }

    /// Set the continuous sequence type.
    #[must_use = "set_inf_seq_sel returns a modified PktCtrl"]
    pub const fn set_inf_seq_sel(mut self, sel: InfSeqSel) -> PktCtrl {
        self.val &= !(0b11 << 2);
        self.val |= (sel as u8) << 2;
        self
    }

    /// Get the continuous sequence type.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{InfSeqSel, PktCtrl};
    ///
    /// let pc: PktCtrl = PktCtrl::RESET;
    /// assert_eq!(pc.inf_seq_sel(), InfSeqSel::Five);
    ///
    /// let pc: PktCtrl = pc.set_inf_seq_sel(InfSeqSel::Zero);
    /// assert_eq!(pc.inf_seq_sel(), InfSeqSel::Zero);
    ///
    /// let pc: PktCtrl = pc.set_inf_seq_sel(InfSeqSel::One);
    /// assert_eq!(pc.inf_seq_sel(), InfSeqSel::One);
    ///
    /// let pc: PktCtrl = pc.set_inf_seq_sel(InfSeqSel::Prbs9);
    /// assert_eq!(pc.inf_seq_sel(), InfSeqSel::Prbs9);
    ///
    /// let pc: PktCtrl = pc.set_inf_seq_sel(InfSeqSel::Five);
    /// assert_eq!(pc.inf_seq_sel(), InfSeqSel::Five);
    /// ```
    pub const fn inf_seq_sel(&self) -> InfSeqSel {
        match (self.val >> 2) & 0b11 {
            0b00 => InfSeqSel::Five,
            0b01 => InfSeqSel::Zero,
            0b10 => InfSeqSel::One,
            _ => InfSeqSel::Prbs9,
        }
    }

    /// Enable infinite sequence generation.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// const PKT_CTRL: PktCtrl = PktCtrl::RESET.set_inf_seq_en(true);
    /// ```
    #[must_use = "set_inf_seq_en returns a modified PktCtrl"]
    pub const fn set_inf_seq_en(mut self, en: bool) -> PktCtrl {
        if en {
            self.val |= 1 << 1;
        } else {
            self.val &= !(1 << 1);
        }
        self
    }

    /// Returns `true` if infinite sequence generation is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// let pc: PktCtrl = PktCtrl::RESET;
    /// assert_eq!(pc.inf_seq_en(), false);
    /// let pc: PktCtrl = pc.set_inf_seq_en(true);
    /// assert_eq!(pc.inf_seq_en(), true);
    /// let pc: PktCtrl = pc.set_inf_seq_en(false);
    /// assert_eq!(pc.inf_seq_en(), false);
    /// ```
    pub const fn inf_seq_en(&self) -> bool {
        self.val & (1 << 1) != 0
    }

    /// Set the value of bit-8 (9th bit) for generic packet whitening.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// const PKT_CTRL: PktCtrl = PktCtrl::RESET.set_whitening_init(true);
    /// ```
    #[must_use = "set_whitening_init returns a modified PktCtrl"]
    pub const fn set_whitening_init(mut self, val: bool) -> PktCtrl {
        if val {
            self.val |= 1;
        } else {
            self.val &= !1;
        }
        self
    }

    /// Returns `true` if bit-8 of the generic packet whitening is set.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::PktCtrl;
    ///
    /// let pc: PktCtrl = PktCtrl::RESET;
    /// assert_eq!(pc.whitening_init(), true);
    /// let pc: PktCtrl = pc.set_whitening_init(false);
    /// assert_eq!(pc.whitening_init(), false);
    /// let pc: PktCtrl = pc.set_whitening_init(true);
    /// assert_eq!(pc.whitening_init(), true);
    /// ```
    pub const fn whitening_init(&self) -> bool {
        self.val & 0b1 != 0
    }
}

impl From<PktCtrl> for u8 {
    fn from(pc: PktCtrl) -> Self {
        pc.val
    }
}

impl Default for PktCtrl {
    fn default() -> Self {
        Self::RESET
    }
}
