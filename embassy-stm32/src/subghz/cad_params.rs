use super::Timeout;

/// Number of symbols used for channel activity detection scans.
///
/// Argument of [`CadParams::set_num_symbol`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum NbCadSymbol {
    /// 1 symbol.
    S1 = 0x0,
    /// 2 symbols.
    S2 = 0x1,
    /// 4 symbols.
    S4 = 0x2,
    /// 8 symbols.
    S8 = 0x3,
    /// 16 symbols.
    S16 = 0x4,
}

/// Mode to enter after a channel activity detection scan is finished.
///
/// Argument of [`CadParams::set_exit_mode`].
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ExitMode {
    /// Standby with RC 13 MHz mode entry after CAD.
    Standby = 0,
    /// Standby with RC 13 MHz mode after CAD if no LoRa symbol is detected
    /// during the CAD scan.
    /// If a LoRa symbol is detected, the sub-GHz radio stays in RX mode
    /// until a packet is received or until the CAD timeout is reached.
    StandbyLoRa = 1,
}

/// Channel activity detection (CAD) parameters.
///
/// Argument of [`set_cad_params`].
///
/// # Recommended CAD settings
///
/// This is taken directly from the datasheet.
///
/// "The correct values selected in the table below must be carefully tested to
/// ensure a good detection at sensitivity level and to limit the number of
/// false detections"
///
/// | SF (Spreading Factor) | [`set_det_peak`] | [`set_det_min`] |
/// |-----------------------|------------------|-----------------|
/// |                     5 |             0x18 |            0x10 |
/// |                     6 |             0x19 |            0x10 |
/// |                     7 |             0x20 |            0x10 |
/// |                     8 |             0x21 |            0x10 |
/// |                     9 |             0x22 |            0x10 |
/// |                    10 |             0x23 |            0x10 |
/// |                    11 |             0x24 |            0x10 |
/// |                    12 |             0x25 |            0x10 |
///
/// [`set_cad_params`]: crate::subghz::SubGhz::set_cad_params
/// [`set_det_peak`]: crate::subghz::CadParams::set_det_peak
/// [`set_det_min`]: crate::subghz::CadParams::set_det_min
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CadParams {
    buf: [u8; 8],
}

impl CadParams {
    /// Create a new `CadParams`.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CadParams;
    ///
    /// const CAD_PARAMS: CadParams = CadParams::new();
    /// assert_eq!(CAD_PARAMS, CadParams::default());
    /// ```
    pub const fn new() -> CadParams {
        CadParams {
            buf: [super::OpCode::SetCadParams as u8, 0, 0, 0, 0, 0, 0, 0],
        }
        .set_num_symbol(NbCadSymbol::S1)
        .set_det_peak(0x18)
        .set_det_min(0x10)
        .set_exit_mode(ExitMode::Standby)
    }

    /// Number of symbols used for a CAD scan.
    ///
    /// # Example
    ///
    /// Set the number of symbols to 4.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CadParams, NbCadSymbol};
    ///
    /// const CAD_PARAMS: CadParams = CadParams::new().set_num_symbol(NbCadSymbol::S4);
    /// # assert_eq!(CAD_PARAMS.as_slice()[1], 0x2);
    /// ```
    #[must_use = "set_num_symbol returns a modified CadParams"]
    pub const fn set_num_symbol(mut self, nb: NbCadSymbol) -> CadParams {
        self.buf[1] = nb as u8;
        self
    }

    /// Used with [`set_det_min`] to correlate the LoRa symbol.
    ///
    /// See the table in [`CadParams`] docs for recommended values.
    ///
    /// # Example
    ///
    /// Setting the recommended value for a spreading factor of 7.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CadParams;
    ///
    /// const CAD_PARAMS: CadParams = CadParams::new().set_det_peak(0x20).set_det_min(0x10);
    /// # assert_eq!(CAD_PARAMS.as_slice()[2], 0x20);
    /// # assert_eq!(CAD_PARAMS.as_slice()[3], 0x10);
    /// ```
    ///
    /// [`set_det_min`]: crate::subghz::CadParams::set_det_min
    #[must_use = "set_det_peak returns a modified CadParams"]
    pub const fn set_det_peak(mut self, peak: u8) -> CadParams {
        self.buf[2] = peak;
        self
    }

    /// Used with [`set_det_peak`] to correlate the LoRa symbol.
    ///
    /// See the table in [`CadParams`] docs for recommended values.
    ///
    /// # Example
    ///
    /// Setting the recommended value for a spreading factor of 6.
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CadParams;
    ///
    /// const CAD_PARAMS: CadParams = CadParams::new().set_det_peak(0x18).set_det_min(0x10);
    /// # assert_eq!(CAD_PARAMS.as_slice()[2], 0x18);
    /// # assert_eq!(CAD_PARAMS.as_slice()[3], 0x10);
    /// ```
    ///
    /// [`set_det_peak`]: crate::subghz::CadParams::set_det_peak
    #[must_use = "set_det_min returns a modified CadParams"]
    pub const fn set_det_min(mut self, min: u8) -> CadParams {
        self.buf[3] = min;
        self
    }

    /// Mode to enter after a channel activity detection scan is finished.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CadParams, ExitMode};
    ///
    /// const CAD_PARAMS: CadParams = CadParams::new().set_exit_mode(ExitMode::Standby);
    /// # assert_eq!(CAD_PARAMS.as_slice()[4], 0x00);
    /// # assert_eq!(CAD_PARAMS.set_exit_mode(ExitMode::StandbyLoRa).as_slice()[4], 0x01);
    /// ```
    #[must_use = "set_exit_mode returns a modified CadParams"]
    pub const fn set_exit_mode(mut self, mode: ExitMode) -> CadParams {
        self.buf[4] = mode as u8;
        self
    }

    /// Set the timeout.
    ///
    /// This is only used with [`ExitMode::StandbyLoRa`].
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CadParams, ExitMode, Timeout};
    ///
    /// const TIMEOUT: Timeout = Timeout::from_raw(0x123456);
    /// const CAD_PARAMS: CadParams = CadParams::new()
    ///     .set_exit_mode(ExitMode::StandbyLoRa)
    ///     .set_timeout(TIMEOUT);
    /// # assert_eq!(CAD_PARAMS.as_slice()[4], 0x01);
    /// # assert_eq!(CAD_PARAMS.as_slice()[5], 0x12);
    /// # assert_eq!(CAD_PARAMS.as_slice()[6], 0x34);
    /// # assert_eq!(CAD_PARAMS.as_slice()[7], 0x56);
    /// ```
    #[must_use = "set_timeout returns a modified CadParams"]
    pub const fn set_timeout(mut self, to: Timeout) -> CadParams {
        let to_bytes: [u8; 3] = to.as_bytes();
        self.buf[5] = to_bytes[0];
        self.buf[6] = to_bytes[1];
        self.buf[7] = to_bytes[2];
        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CadParams, ExitMode, NbCadSymbol, Timeout};
    ///
    /// const TIMEOUT: Timeout = Timeout::from_raw(0x123456);
    /// const CAD_PARAMS: CadParams = CadParams::new()
    ///     .set_num_symbol(NbCadSymbol::S4)
    ///     .set_det_peak(0x18)
    ///     .set_det_min(0x10)
    ///     .set_exit_mode(ExitMode::StandbyLoRa)
    ///     .set_timeout(TIMEOUT);
    ///
    /// assert_eq!(
    ///     CAD_PARAMS.as_slice(),
    ///     &[0x88, 0x02, 0x18, 0x10, 0x01, 0x12, 0x34, 0x56]
    /// );
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for CadParams {
    fn default() -> Self {
        Self::new()
    }
}
