/// Interrupt lines.
///
/// Argument of [`CfgIrq::irq_enable`] and [`CfgIrq::irq_disable`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IrqLine {
    /// Global interrupt.
    Global,
    /// Interrupt line 1.
    ///
    /// This will output to the [`RfIrq0`](crate::gpio::RfIrq0) pin.
    Line1,
    /// Interrupt line 2.
    ///
    /// This will output to the [`RfIrq1`](crate::gpio::RfIrq1) pin.
    Line2,
    /// Interrupt line 3.
    ///
    /// This will output to the [`RfIrq2`](crate::gpio::RfIrq2) pin.
    Line3,
}

impl IrqLine {
    pub(super) const fn offset(&self) -> usize {
        match self {
            IrqLine::Global => 1,
            IrqLine::Line1 => 3,
            IrqLine::Line2 => 5,
            IrqLine::Line3 => 7,
        }
    }
}

/// IRQ bit mapping
///
/// See table 37 "IRQ bit mapping and definition" in the reference manual for
/// more information.
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Irq {
    /// Packet transmission finished.
    ///
    /// * Packet type: LoRa and GFSK
    /// * Operation: TX
    TxDone = (1 << 0),
    /// Packet reception finished.
    ///
    /// * Packet type: LoRa and GFSK
    /// * Operation: RX
    RxDone = (1 << 1),
    /// Preamble detected.
    ///
    /// * Packet type: LoRa and GFSK
    /// * Operation: RX
    PreambleDetected = (1 << 2),
    /// Synchronization word valid.
    ///
    /// * Packet type: GFSK
    /// * Operation: RX
    SyncDetected = (1 << 3),
    /// Header valid.
    ///
    /// * Packet type: LoRa
    /// * Operation: RX
    HeaderValid = (1 << 4),
    /// Header CRC error.
    ///
    /// * Packet type: LoRa
    /// * Operation: RX
    HeaderErr = (1 << 5),
    /// Dual meaning error.
    ///
    /// For GFSK RX this indicates a preamble, syncword, address, CRC, or length
    /// error.
    ///
    /// For LoRa RX this indicates a CRC error.
    Err = (1 << 6),
    /// Channel activity detection finished.
    ///
    /// * Packet type: LoRa
    /// * Operation: CAD
    CadDone = (1 << 7),
    /// Channel activity detected.
    ///
    /// * Packet type: LoRa
    /// * Operation: CAD
    CadDetected = (1 << 8),
    /// RX or TX timeout.
    ///
    /// * Packet type: LoRa and GFSK
    /// * Operation: RX and TX
    Timeout = (1 << 9),
}

impl Irq {
    /// Get the bitmask for an IRQ.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::Irq;
    ///
    /// assert_eq!(Irq::TxDone.mask(), 0x0001);
    /// assert_eq!(Irq::Timeout.mask(), 0x0200);
    /// ```
    pub const fn mask(self) -> u16 {
        self as u16
    }
}

/// Argument for [`set_irq_cfg`].
///
/// [`set_irq_cfg`]: crate::subghz::SubGhz::set_irq_cfg
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CfgIrq {
    buf: [u8; 9],
}

impl CfgIrq {
    /// Create a new `CfgIrq`.
    ///
    /// This is the same as `default`, but in a `const` function.
    ///
    /// The default value has all interrupts disabled on all lines.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CfgIrq;
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new();
    /// ```
    pub const fn new() -> CfgIrq {
        CfgIrq {
            buf: [
                super::OpCode::CfgDioIrq as u8,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        }
    }

    /// Enable an interrupt.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CfgIrq, Irq, IrqLine};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable(IrqLine::Global, Irq::TxDone)
    ///     .irq_enable(IrqLine::Global, Irq::Timeout);
    /// # assert_eq!(IRQ_CFG.as_slice()[1], 0x02);
    /// # assert_eq!(IRQ_CFG.as_slice()[2], 0x01);
    /// # assert_eq!(IRQ_CFG.as_slice()[3], 0x00);
    /// ```
    #[must_use = "irq_enable returns a modified CfgIrq"]
    pub const fn irq_enable(mut self, line: IrqLine, irq: Irq) -> CfgIrq {
        let mask: u16 = irq as u16;
        let offset: usize = line.offset();
        self.buf[offset] |= ((mask >> 8) & 0xFF) as u8;
        self.buf[offset + 1] |= (mask & 0xFF) as u8;
        self
    }

    /// Enable an interrupt on all lines.
    ///
    /// As far as I can tell with empirical testing all IRQ lines need to be
    /// enabled for the internal interrupt to be pending in the NVIC.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CfgIrq, Irq};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable_all(Irq::TxDone)
    ///     .irq_enable_all(Irq::Timeout);
    /// # assert_eq!(IRQ_CFG.as_slice()[1], 0x02);
    /// # assert_eq!(IRQ_CFG.as_slice()[2], 0x01);
    /// # assert_eq!(IRQ_CFG.as_slice()[3], 0x02);
    /// # assert_eq!(IRQ_CFG.as_slice()[4], 0x01);
    /// # assert_eq!(IRQ_CFG.as_slice()[5], 0x02);
    /// # assert_eq!(IRQ_CFG.as_slice()[6], 0x01);
    /// # assert_eq!(IRQ_CFG.as_slice()[7], 0x02);
    /// # assert_eq!(IRQ_CFG.as_slice()[8], 0x01);
    /// ```
    #[must_use = "irq_enable_all returns a modified CfgIrq"]
    pub const fn irq_enable_all(mut self, irq: Irq) -> CfgIrq {
        let mask: [u8; 2] = irq.mask().to_be_bytes();

        self.buf[1] |= mask[0];
        self.buf[2] |= mask[1];
        self.buf[3] |= mask[0];
        self.buf[4] |= mask[1];
        self.buf[5] |= mask[0];
        self.buf[6] |= mask[1];
        self.buf[7] |= mask[0];
        self.buf[8] |= mask[1];

        self
    }

    /// Disable an interrupt.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CfgIrq, Irq, IrqLine};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable(IrqLine::Global, Irq::TxDone)
    ///     .irq_enable(IrqLine::Global, Irq::Timeout)
    ///     .irq_disable(IrqLine::Global, Irq::TxDone)
    ///     .irq_disable(IrqLine::Global, Irq::Timeout);
    /// # assert_eq!(IRQ_CFG.as_slice()[1], 0x00);
    /// # assert_eq!(IRQ_CFG.as_slice()[2], 0x00);
    /// # assert_eq!(IRQ_CFG.as_slice()[3], 0x00);
    /// ```
    #[must_use = "irq_disable returns a modified CfgIrq"]
    pub const fn irq_disable(mut self, line: IrqLine, irq: Irq) -> CfgIrq {
        let mask: u16 = !(irq as u16);
        let offset: usize = line.offset();
        self.buf[offset] &= ((mask >> 8) & 0xFF) as u8;
        self.buf[offset + 1] &= (mask & 0xFF) as u8;
        self
    }

    /// Disable an interrupt on all lines.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CfgIrq, Irq};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable_all(Irq::TxDone)
    ///     .irq_enable_all(Irq::Timeout)
    ///     .irq_disable_all(Irq::TxDone)
    ///     .irq_disable_all(Irq::Timeout);
    /// # assert_eq!(IRQ_CFG, CfgIrq::new());
    /// ```
    #[must_use = "irq_disable_all returns a modified CfgIrq"]
    pub const fn irq_disable_all(mut self, irq: Irq) -> CfgIrq {
        let mask: [u8; 2] = (!irq.mask()).to_be_bytes();

        self.buf[1] &= mask[0];
        self.buf[2] &= mask[1];
        self.buf[3] &= mask[0];
        self.buf[4] &= mask[1];
        self.buf[5] &= mask[0];
        self.buf[6] &= mask[1];
        self.buf[7] &= mask[0];
        self.buf[8] &= mask[1];

        self
    }

    /// Extracts a slice containing the packet.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CfgIrq, Irq};
    ///
    /// const IRQ_CFG: CfgIrq = CfgIrq::new()
    ///     .irq_enable_all(Irq::TxDone)
    ///     .irq_enable_all(Irq::Timeout);
    ///
    /// assert_eq!(
    ///     IRQ_CFG.as_slice(),
    ///     &[0x08, 0x02, 0x01, 0x02, 0x01, 0x02, 0x01, 0x02, 0x01]
    /// );
    /// ```
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for CfgIrq {
    fn default() -> Self {
        Self::new()
    }
}
