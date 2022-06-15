/// Bit synchronization.
///
/// This must be cleared to `0x00` (the reset value) when using packet types
/// other than LoRa.
///
/// Argument of [`set_bit_sync`](crate::subghz::SubGhz::set_bit_sync).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitSync {
    val: u8,
}

impl BitSync {
    /// Bit synchronization register reset value.
    pub const RESET: BitSync = BitSync { val: 0x00 };

    /// Create a new [`BitSync`] structure from a raw value.
    ///
    /// Reserved bits will be masked.
    pub const fn from_raw(raw: u8) -> Self {
        Self { val: raw & 0x70 }
    }

    /// Get the raw value of the [`BitSync`] register.
    pub const fn as_bits(&self) -> u8 {
        self.val
    }

    /// LoRa simple bit synchronization enable.
    ///
    /// # Example
    ///
    /// Enable simple bit synchronization.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// const BIT_SYNC: BitSync = BitSync::RESET.set_simple_bit_sync_en(true);
    /// # assert_eq!(u8::from(BIT_SYNC), 0x40u8);
    /// ```
    #[must_use = "set_simple_bit_sync_en returns a modified BitSync"]
    pub const fn set_simple_bit_sync_en(mut self, en: bool) -> BitSync {
        if en {
            self.val |= 1 << 6;
        } else {
            self.val &= !(1 << 6);
        }
        self
    }

    /// Returns `true` if simple bit synchronization is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// let bs: BitSync = BitSync::RESET;
    /// assert_eq!(bs.simple_bit_sync_en(), false);
    /// let bs: BitSync = bs.set_simple_bit_sync_en(true);
    /// assert_eq!(bs.simple_bit_sync_en(), true);
    /// let bs: BitSync = bs.set_simple_bit_sync_en(false);
    /// assert_eq!(bs.simple_bit_sync_en(), false);
    /// ```
    pub const fn simple_bit_sync_en(&self) -> bool {
        self.val & (1 << 6) != 0
    }

    /// LoRa RX data inversion.
    ///
    /// # Example
    ///
    /// Invert receive data.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// const BIT_SYNC: BitSync = BitSync::RESET.set_rx_data_inv(true);
    /// # assert_eq!(u8::from(BIT_SYNC), 0x20u8);
    /// ```
    #[must_use = "set_rx_data_inv returns a modified BitSync"]
    pub const fn set_rx_data_inv(mut self, inv: bool) -> BitSync {
        if inv {
            self.val |= 1 << 5;
        } else {
            self.val &= !(1 << 5);
        }
        self
    }

    /// Returns `true` if LoRa RX data is inverted.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// let bs: BitSync = BitSync::RESET;
    /// assert_eq!(bs.rx_data_inv(), false);
    /// let bs: BitSync = bs.set_rx_data_inv(true);
    /// assert_eq!(bs.rx_data_inv(), true);
    /// let bs: BitSync = bs.set_rx_data_inv(false);
    /// assert_eq!(bs.rx_data_inv(), false);
    /// ```
    pub const fn rx_data_inv(&self) -> bool {
        self.val & (1 << 5) != 0
    }

    /// LoRa normal bit synchronization enable.
    ///
    /// # Example
    ///
    /// Enable normal bit synchronization.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// const BIT_SYNC: BitSync = BitSync::RESET.set_norm_bit_sync_en(true);
    /// # assert_eq!(u8::from(BIT_SYNC), 0x10u8);
    /// ```
    #[must_use = "set_norm_bit_sync_en returns a modified BitSync"]
    pub const fn set_norm_bit_sync_en(mut self, en: bool) -> BitSync {
        if en {
            self.val |= 1 << 4;
        } else {
            self.val &= !(1 << 4);
        }
        self
    }

    /// Returns `true` if normal bit synchronization is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::BitSync;
    ///
    /// let bs: BitSync = BitSync::RESET;
    /// assert_eq!(bs.norm_bit_sync_en(), false);
    /// let bs: BitSync = bs.set_norm_bit_sync_en(true);
    /// assert_eq!(bs.norm_bit_sync_en(), true);
    /// let bs: BitSync = bs.set_norm_bit_sync_en(false);
    /// assert_eq!(bs.norm_bit_sync_en(), false);
    /// ```
    pub const fn norm_bit_sync_en(&self) -> bool {
        self.val & (1 << 4) != 0
    }
}

impl From<BitSync> for u8 {
    fn from(bs: BitSync) -> Self {
        bs.val
    }
}

impl Default for BitSync {
    fn default() -> Self {
        Self::RESET
    }
}
