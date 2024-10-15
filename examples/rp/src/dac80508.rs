use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

/// Represents possible errors in ADC operations.
#[derive(Debug)]
pub enum Dac80505Error<SpiError, PinError> {
    SpiError(SpiError),
    PinError(PinError),
    InvalidRead,
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
    Nop = 0x00,
    DeviceId = 0x01,
    Sync = 0x02,
    Config = 0x03,
    Gain = 0x04,
    Trigger = 0x05,
    Broadcast = 0x06,
    Status = 0x07,
    Dac0 = 0x08,
    Dac1 = 0x09,
    Dac2 = 0x0A,
    Dac3 = 0x0B,
    Dac4 = 0x0C,
    Dac5 = 0x0D,
    Dac6 = 0x0E,
    Dac7 = 0x0F,
}

pub struct DAC80508Driver;

impl DAC80508Driver {
    pub fn new() -> Self {
        Self {}
    }

    /// Helper to build the command word for writing/reading registers
    fn build_command(register: Register, is_read: bool) -> u8 {
        let mut command = register as u8;
        if is_read {
            command |= 0x80; // Set the read bit (bit 7)
        }
        command
    }

    /// Write data to a register
    pub async fn write_register<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        register: Register,
        data: u16,
    ) -> Result<(), Dac80505Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        let command = Self::build_command(register, false);
        let to_send = [command, (data >> 8) as u8, data as u8];

        cs.set_low().map_err(Dac80505Error::PinError)?;
        spi.write(&to_send).await.map_err(Dac80505Error::SpiError)?;
        cs.set_high().map_err(Dac80505Error::PinError)?;

        Ok(())
    }

    pub async fn read_register<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        register: Register,
    ) -> Result<(u8, u16), Dac80505Error<SPI::Error, CS::Error>>
    // Now returns echoed address and data
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        let command = Self::build_command(register, true); // Prepare the read command

        // Step 1: Send the read command (first access cycle)
        cs.set_low().map_err(Dac80505Error::PinError)?;
        spi.write(&[command, 0x00, 0x00])
            .await
            .map_err(Dac80505Error::SpiError)?;
        cs.set_high().map_err(Dac80505Error::PinError)?;

        // Step 2: Read 3 bytes (RW echo, reserved bits, address echo, and 16-bit data)
        let mut read_buffer = [0u8; 3]; // Buffer for receiving the 3-byte response
        cs.set_low().map_err(Dac80505Error::PinError)?;
        spi.read(&mut read_buffer).await.map_err(Dac80505Error::SpiError)?;
        cs.set_high().map_err(Dac80505Error::PinError)?;

        // Step 3: Verify that the response is a read response
        let rw_bit = (read_buffer[0] & 0x80) != 0; // Check if the RW bit is set to 1 for read
        let echoed_address = read_buffer[0] & 0x0F; // Extract the echoed address from bits [3:0]

        if !rw_bit {
            return Err(Dac80505Error::InvalidRead);
        }

        // Extract the 16-bit data from the last two bytes of the read buffer (DO[15:0])
        let result = ((read_buffer[1] as u16) << 8) | (read_buffer[2] as u16);

        // Return the echoed address and the 16-bit data
        Ok((echoed_address, result))
    }

    /// Write to a specific DAC channel
    pub async fn write_dac<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        channel: Register,
        value: u16,
    ) -> Result<(), Dac80505Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.write_register(spi, cs, channel, value).await
    }

    /// Read from a specific DAC channel
    pub async fn read_dac<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        channel: Register,
    ) -> Result<(u8, u16), Dac80505Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.read_register(spi, cs, channel).await
    }
}

macro_rules! bit_setter {
    ($name:ident, $bit:expr) => {
        pub fn $name(mut self, enabled: bool) -> Self {
            if enabled {
                self.value |= 1 << $bit;
            } else {
                self.value &= !(1 << $bit);
            }
            self
        }
    };
}

/// SYNC Register Builder
pub struct SyncRegisterBuilder {
    value: u16,
}

impl SyncRegisterBuilder {
    pub fn new() -> Self {
        // Start with the reset value 0xFF00
        Self { value: 0xFF00 }
    }

    /// Build the final 16-bit value for the register.
    pub fn build(self) -> u16 {
        self.value
    }

    bit_setter!(dac7_broadcast_en, 15);
    bit_setter!(dac6_broadcast_en, 14);
    bit_setter!(dac5_broadcast_en, 13);
    bit_setter!(dac4_broadcast_en, 12);
    bit_setter!(dac3_broadcast_en, 11);
    bit_setter!(dac2_broadcast_en, 10);
    bit_setter!(dac1_broadcast_en, 9);
    bit_setter!(dac0_broadcast_en, 8);
    bit_setter!(dac7_sync_en, 7);
    bit_setter!(dac6_sync_en, 6);
    bit_setter!(dac5_sync_en, 5);
    bit_setter!(dac4_sync_en, 4);
    bit_setter!(dac3_sync_en, 3);
    bit_setter!(dac2_sync_en, 2);
    bit_setter!(dac1_sync_en, 1);
    bit_setter!(dac0_sync_en, 0);
}

/// CONFIG Register Builder
pub struct ConfigRegisterBuilder {
    value: u16,
}

impl ConfigRegisterBuilder {
    pub fn new() -> Self {
        // Start with the reset value 0x0000
        Self { value: 0x0000 }
    }

    /// Build the final 16-bit value for the register.
    pub fn build(self) -> u16 {
        self.value
    }

    bit_setter!(alarm_select, 13);
    bit_setter!(alarm_enable, 12);
    bit_setter!(crc_enable, 11);
    bit_setter!(fsdo, 10);
    bit_setter!(dsdo, 9);
    bit_setter!(ref_pwdwn, 8);
    bit_setter!(dac7_pwdwn, 7);
    bit_setter!(dac6_pwdwn, 6);
    bit_setter!(dac5_pwdwn, 5);
    bit_setter!(dac4_pwdwn, 4);
    bit_setter!(dac3_pwdwn, 3);
    bit_setter!(dac2_pwdwn, 2);
    bit_setter!(dac1_pwdwn, 1);
    bit_setter!(dac0_pwdwn, 0);
}

pub struct TriggerRegisterBuilder {
    value: u16,
}

impl TriggerRegisterBuilder {
    pub fn new() -> Self {
        // Start with the reset value 0x0000
        Self { value: 0x0000 }
    }

    /// Build the final 16-bit value for the register.
    pub fn build(self) -> u16 {
        self.value
    }

    // Setter for LDAC bit (bit 4)
    bit_setter!(ldac, 4);

    /// Sets the soft reset value (4 bits) to the reserved reset code `1010`.
    pub fn soft_reset(mut self) -> Self {
        // Clear bits 0-3
        self.value &= !(0xF);
        // Set the reserved reset code `1010`
        self.value |= 0b1010;
        self
    }
}

pub struct GainRegisterBuilder {
    value: u16,
}

impl GainRegisterBuilder {
    pub fn new() -> Self {
        // Start with the default value, which could be different depending on the specific DAC.
        Self { value: 0x0000 }
    }

    /// Build the final 16-bit value for the register.
    pub fn build(self) -> u16 {
        self.value
    }

    bit_setter!(refdiv_en, 8);
    bit_setter!(buff7_gain, 7);
    bit_setter!(buff6_gain, 6);
    bit_setter!(buff5_gain, 5);
    bit_setter!(buff4_gain, 4);
    bit_setter!(buff3_gain, 3);
    bit_setter!(buff2_gain, 2);
    bit_setter!(buff1_gain, 1);
    bit_setter!(buff0_gain, 0);
}
