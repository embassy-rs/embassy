//! Minimal driver for the Sony IMX335 5 Mpx CMOS sensor on the MB1854
//! (B-CAMS-IMX) daughterboard, as wired to the STM32N6570-DK.
//!
//! Bus + GPIO mapping (UM3300 §7.16, §8.10):
//!   - I2C1: SCL=PH9, SDA=PC1; sensor at 7-bit address 0x1A.
//!   - PC8 / PD2: power-enable + reset lines, sequenced per ST BSP.
//!
//! Register tables and the power-up sequence are ported directly from
//! `Drivers/BSP/Components/imx335/imx335.c` and
//! `Drivers/BSP/STM32N6570-DK/stm32n6570_discovery_camera.c` in
//! STM32CubeN6 v1.3.0 — same byte sequences, just retyped in Rust. We only
//! support the single mode this example needs: 2592×1944 RAW10 RGGB Bayer
//! over 2 MIPI lanes at 24 MHz INCK, ~30 fps.

use embassy_stm32::gpio::Output;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Blocking;
use embassy_time::Timer;
use embedded_hal_1::i2c::Operation;

/// 7-bit I2C address of the IMX335. The BSP uses 0x34 as the 8-bit form.
pub const I2C_ADDRESS: u8 = 0x1A;

/// Register that returns the chip ID. Reset value is `0x00`, which the BSP
/// treats as the "IMX335 present" signature.
const REG_ID: u16 = 0x3912;
const CHIP_ID: u8 = 0x00;

const REG_MODE_SELECT: u16 = 0x3000;
const MODE_STREAMING: u8 = 0x00;
#[allow(dead_code)]
const MODE_STANDBY: u8 = 0x01;

// "Hold" register: writing 1 freezes the gain/shutter shadow registers so a
// burst of writes is committed atomically. Write 0 to unhold.
const REG_HOLD: u16 = 0x3001;
// Analog gain, 16-bit, units of 0.3 dB (LSB at 0x30E8).
const REG_GAIN: u16 = 0x30E8;
// Shutter, 24-bit lines (LSB at 0x3058). Effective exposure ≈ (VMAX − shutter).
const REG_SHUTTER: u16 = 0x3058;

/// Errors surfaced by the IMX335 driver. The variants stay coarse on
/// purpose — the example logs them via defmt and gives up.
#[derive(Debug, defmt::Format)]
pub enum Error {
    /// I2C transaction failed (NACK, bus error, arbitration loss, …).
    I2c,
    /// `REG_ID` did not return the expected `CHIP_ID`.
    WrongId(u8),
}

type Reg = (u16, u8);

/// 2592×1944 mode register block (BSP `res_2592_1944_regs`).
const RES_2592_1944: &[Reg] = &[
    (0x3000, 0x01),
    (0x3002, 0x00),
    (0x300c, 0x3b),
    (0x300d, 0x2a),
    (0x3018, 0x04),
    (0x302c, 0x3c),
    (0x302e, 0x20),
    (0x3056, 0x98),
    (0x3074, 0xc8),
    (0x3076, 0x30),
    (0x304c, 0x00),
    (0x314c, 0xc6),
    (0x315a, 0x02),
    (0x3168, 0xa0),
    (0x316a, 0x7e),
    (0x31a1, 0x00),
    (0x3288, 0x21),
    (0x328a, 0x02),
    (0x3414, 0x05),
    (0x3416, 0x18),
    (0x3648, 0x01),
    (0x364a, 0x04),
    (0x364c, 0x04),
    (0x3678, 0x01),
    (0x367c, 0x31),
    (0x367e, 0x31),
    (0x3706, 0x10),
    (0x3708, 0x03),
    (0x3714, 0x02),
    (0x3715, 0x02),
    (0x3716, 0x01),
    (0x3717, 0x03),
    (0x371c, 0x3d),
    (0x371d, 0x3f),
    (0x372c, 0x00),
    (0x372d, 0x00),
    (0x372e, 0x46),
    (0x372f, 0x00),
    (0x3730, 0x89),
    (0x3731, 0x00),
    (0x3732, 0x08),
    (0x3733, 0x01),
    (0x3734, 0xfe),
    (0x3735, 0x05),
    (0x3740, 0x02),
    (0x375d, 0x00),
    (0x375e, 0x00),
    (0x375f, 0x11),
    (0x3760, 0x01),
    (0x3768, 0x1b),
    (0x3769, 0x1b),
    (0x376a, 0x1b),
    (0x376b, 0x1b),
    (0x376c, 0x1a),
    (0x376d, 0x17),
    (0x376e, 0x0f),
    (0x3776, 0x00),
    (0x3777, 0x00),
    (0x3778, 0x46),
    (0x3779, 0x00),
    (0x377a, 0x89),
    (0x377b, 0x00),
    (0x377c, 0x08),
    (0x377d, 0x01),
    (0x377e, 0x23),
    (0x377f, 0x02),
    (0x3780, 0xd9),
    (0x3781, 0x03),
    (0x3782, 0xf5),
    (0x3783, 0x06),
    (0x3784, 0xa5),
    (0x3788, 0x0f),
    (0x378a, 0xd9),
    (0x378b, 0x03),
    (0x378c, 0xeb),
    (0x378d, 0x05),
    (0x378e, 0x87),
    (0x378f, 0x06),
    (0x3790, 0xf5),
    (0x3792, 0x43),
    (0x3794, 0x7a),
    (0x3796, 0xa1),
    (0x37b0, 0x36),
    (0x3a00, 0x01),
];

/// 2-lane RAW10 mode (BSP `mode_2l_10b_regs`).
const MODE_2LANE_10BIT: &[Reg] = &[
    (0x3050, 0x00),
    (0x319D, 0x00),
    (0x341c, 0xff),
    (0x341d, 0x01),
    (0x3a01, 0x01),
];

/// 24 MHz INCK trim block (BSP `inck_24Mhz_regs`). The MB1854
/// daughterboard runs the sensor off its own 24 MHz oscillator.
const INCK_24MHZ: &[Reg] = &[
    (0x300c, 0x3B),
    (0x300d, 0x2A),
    (0x314c, 0xC6),
    (0x314d, 0x00),
    (0x315a, 0x02),
    (0x3168, 0xA0),
    (0x316a, 0x7E),
];

/// 30 fps frame-length / line-length pair (BSP `framerate_30fps_regs`).
const FRAMERATE_30FPS: &[Reg] = &[(0x3030, 0x94), (0x3031, 0x11)];

/// IMX335 sensor driver.
///
/// `pwr_en` and `nrst` are owned outputs the driver pulses during the
/// power-up sequence. Naming follows the working BSP semantics rather
/// than the silk-screened net names on the connector — empirically the
/// signals on PC8 / PD2 swap roles vs. the names in the UM3300 connector
/// table.
pub struct Imx335<'d> {
    i2c: I2c<'d, Blocking, Master>,
    pwr_en: Output<'d>, // PC8 — daughterboard 2V8 enable, active high.
    nrst: Output<'d>,   // PD2 — sensor NRST, active low.
}

impl<'d> Imx335<'d> {
    /// Wrap the I2C bus and the two control pins.
    pub fn new(i2c: I2c<'d, Blocking, Master>, pwr_en: Output<'d>, nrst: Output<'d>) -> Self {
        Self { i2c, pwr_en, nrst }
    }

    /// Run the BSP power-up sequence and probe the chip ID. Leaves the
    /// sensor in standby — call [`init`] / [`start_streaming`] next.
    ///
    /// [`init`]: Self::init
    /// [`start_streaming`]: Self::start_streaming
    pub async fn power_on(&mut self) -> Result<(), Error> {
        // BSP pre-power: drive both lines low, wait, then bring them up.
        self.pwr_en.set_low();
        Timer::after_millis(100).await;
        self.nrst.set_low();
        Timer::after_millis(100).await;
        self.pwr_en.set_high();
        Timer::after_millis(100).await;
        self.nrst.set_high();
        Timer::after_millis(100).await;

        let id = self.read_reg(REG_ID)?;
        if id != CHIP_ID {
            return Err(Error::WrongId(id));
        }
        Ok(())
    }

    /// Program the sensor for 2592×1944 RAW10 RGGB at 30 fps over 2 MIPI
    /// lanes. Leaves the sensor in standby; [`start_streaming`] enables
    /// output.
    ///
    /// [`start_streaming`]: Self::start_streaming
    pub async fn init(&mut self) -> Result<(), Error> {
        self.write_table(RES_2592_1944).await?;
        self.write_table(MODE_2LANE_10BIT).await?;
        self.write_table(INCK_24MHZ).await?;
        self.write_table(FRAMERATE_30FPS).await?;
        Ok(())
    }

    /// Take the sensor out of standby. Per the BSP a 20 ms post-write
    /// settle is required before the first frame is reliable.
    pub async fn start_streaming(&mut self) -> Result<(), Error> {
        self.write_reg(REG_MODE_SELECT, MODE_STREAMING)?;
        Timer::after_millis(20).await;
        Ok(())
    }

    /// Set analog gain in 0.1 dB units (0..720). The IMX335's native unit
    /// is 0.3 dB; we divide by 3 internally. Writes happen inside a HOLD
    /// envelope so the value is committed atomically.
    pub fn set_gain_db_x10(&mut self, gain_db_x10: u16) -> Result<(), Error> {
        let raw = (gain_db_x10 / 3).min(240); // max ~72 dB
        self.write_reg(REG_HOLD, 1)?;
        self.write_reg(REG_GAIN, raw as u8)?;
        self.write_reg(REG_GAIN + 1, (raw >> 8) as u8)?;
        self.write_reg(REG_HOLD, 0)?;
        Ok(())
    }

    /// Set the shutter ("storage") line count. Effective integration time
    /// is `(VMAX − shutter)` lines; lower values = longer exposure. The
    /// minimum the IMX335 accepts is 9.
    pub fn set_shutter_lines(&mut self, lines: u32) -> Result<(), Error> {
        let v = lines.max(9);
        self.write_reg(REG_HOLD, 1)?;
        self.write_reg(REG_SHUTTER, v as u8)?;
        self.write_reg(REG_SHUTTER + 1, (v >> 8) as u8)?;
        self.write_reg(REG_SHUTTER + 2, (v >> 16) as u8)?;
        self.write_reg(REG_HOLD, 0)?;
        Ok(())
    }

    /// Put the sensor back into standby (stops the CSI output).
    #[allow(dead_code)]
    pub fn stop_streaming(&mut self) -> Result<(), Error> {
        self.write_reg(REG_MODE_SELECT, MODE_STANDBY)
    }

    async fn write_table(&mut self, regs: &[Reg]) -> Result<(), Error> {
        let mut nacks = 0u32;
        for (i, &(addr, val)) in regs.iter().enumerate() {
            match self.write_reg(addr, val) {
                Ok(()) => {}
                Err(_) => {
                    defmt::error!("imx335: NACK at table[{}] reg=0x{:04x} val=0x{:02x}", i, addr, val);
                    nacks += 1;
                }
            }
            // Empirically the IMX335 stretches SCL for tens of ms during
            // back-to-back writes; a small breather keeps the sequence flowing.
            Timer::after_micros(500).await;
        }
        if nacks > 0 {
            defmt::warn!("imx335: {} NACKs in table of {} entries", nacks, regs.len());
        }
        Ok(())
    }

    fn write_reg(&mut self, reg: u16, val: u8) -> Result<(), Error> {
        // Match the wire pattern HAL_I2C_Mem_Write produces: address phase
        // then data phase as two chunks within one I2C transaction. The
        // embassy I2C v2 driver uses NBYTES + RELOAD + NBYTES + AUTOEND
        // for this, which inserts a small SCL pause between the memaddr
        // bytes and the data byte. Using a single 3-byte `blocking_write`
        // skips that pause and the IMX335 NACKs partway through the init
        // table — empirically reproducible at register 0x3678.
        let addr = [(reg >> 8) as u8, reg as u8];
        let data = [val];
        let mut ops = [Operation::Write(&addr), Operation::Write(&data)];
        self.i2c
            .blocking_transaction(I2C_ADDRESS, &mut ops)
            .map_err(|_| Error::I2c)
    }

    fn read_reg(&mut self, reg: u16) -> Result<u8, Error> {
        let addr = [(reg >> 8) as u8, reg as u8];
        let mut data = [0u8; 1];
        self.i2c
            .blocking_write_read(I2C_ADDRESS, &addr, &mut data)
            .map_err(|_| Error::I2c)?;
        Ok(data[0])
    }
}
