use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

/// Represents possible errors in ADC operations.
#[derive(Debug)]
pub enum AD4112Error<SpiError, PinError> {
    SpiError(SpiError),
    PinError(PinError),
    InvalidConfiguration,
}

/// AD4112 Commands
struct AD4112Commands;

impl AD4112Commands {
    const READ_REGISTER: u8 = 0x40;
    const WRITE_REGISTER: u8 = 0x00;
}

/// AD4112 Register Addresses
#[derive(Debug, Clone, Copy)]
pub enum Register {
    // General registers
    Status = 0x00,
    ADCMode = 0x01,
    IFMode = 0x02,
    RegCheck = 0x03,
    Data = 0x04,
    GPIOCon = 0x06,
    ID = 0x07,

    // Channel configuration registers
    CH0 = 0x10,
    CH1 = 0x11,
    CH2 = 0x12,
    CH3 = 0x13,
    CH4 = 0x14,
    CH5 = 0x15,
    CH6 = 0x16,
    CH7 = 0x17,
    CH8 = 0x18,
    CH9 = 0x19,
    CH10 = 0x1A,
    CH11 = 0x1B,
    CH12 = 0x1C,
    CH13 = 0x1D,
    CH14 = 0x1E,
    CH15 = 0x1F,

    // Setup configuration registers
    SetupCon0 = 0x20,
    SetupCon1 = 0x21,
    SetupCon2 = 0x22,
    SetupCon3 = 0x23,
    SetupCon4 = 0x24,
    SetupCon5 = 0x25,
    SetupCon6 = 0x26,
    SetupCon7 = 0x27,

    // Filter configuration registers
    FiltCon0 = 0x28,
    FiltCon1 = 0x29,
    FiltCon2 = 0x2A,
    FiltCon3 = 0x2B,
    FiltCon4 = 0x2C,
    FiltCon5 = 0x2D,
    FiltCon6 = 0x2E,
    FiltCon7 = 0x2F,

    // Offset and Gain registers
    Offset0 = 0x30,
    Offset1 = 0x31,
    Offset2 = 0x32,
    Offset3 = 0x33,
    Offset4 = 0x34,
    Offset5 = 0x35,
    Offset6 = 0x36,
    Offset7 = 0x37,
    Gain0 = 0x38,
    Gain1 = 0x39,
    Gain2 = 0x3A,
    Gain3 = 0x3B,
    Gain4 = 0x3C,
    Gain5 = 0x3D,
    Gain6 = 0x3E,
    Gain7 = 0x3F,
}

/// Enum for ADC Conversion Mode
#[derive(Debug, Clone, Copy)]
pub enum ADCMode {
    Continuous = 0b000,
    Single = 0b001,
    Standby = 0b010,
    PowerDown = 0b011,
    InternalOffsetCalibration = 0b100,
    InternalGainCalibration = 0b101,
    SystemOffsetCalibration = 0b110,
    SystemGainCalibration = 0b111,
}

/// Enum for enabling/disabling the internal reference.
#[derive(Debug, Clone, Copy)]
pub enum RefEnable {
    Disabled = 0b0 << 15,
    Enabled = 0b1 << 15,
}

/// Enum for single cycle mode.
#[derive(Debug, Clone, Copy)]
pub enum SingleCycle {
    Disabled = 0b0 << 13,
    Enabled = 0b1 << 13,
}

/// Enum for setting the delay after a channel switch.
#[derive(Debug, Clone, Copy)]
pub enum Delay {
    Delay0us = 0b000 << 8,
    Delay32us = 0b001 << 8,
    Delay128us = 0b010 << 8,
    Delay320us = 0b011 << 8,
    Delay800us = 0b100 << 8,
    Delay1600us = 0b101 << 8,
    Delay4000us = 0b110 << 8,
    Delay8000us = 0b111 << 8,
}

/// Enum for the ADC operating mode.
#[derive(Debug, Clone, Copy)]
pub enum ADCModeOperation {
    ContinuousConversion = 0b000 << 4,
    SingleConversion = 0b001 << 4,
    Standby = 0b010 << 4,
    PowerDown = 0b011 << 4,
    InternalOffsetCalibration = 0b100 << 4,
    InternalGainCalibration = 0b101 << 4,
    SystemOffsetCalibration = 0b110 << 4,
    SystemGainCalibration = 0b111 << 4,
}

/// Enum for clock selection.
#[derive(Debug, Clone, Copy)]
pub enum ClockSelect {
    Internal = 0b00 << 2,
    InternalOutput = 0b01 << 2,
    ExternalInput = 0b10 << 2,
    ExternalCrystal = 0b11 << 2,
}

/// Enum for enabling/disabling a channel.
#[derive(Debug, Clone, Copy)]
pub enum ChannelEnable {
    Disabled = 0b0 << 15,
    Enabled = 0b1 << 15,
}

/// Enum for selecting the setup configuration for a channel.
#[derive(Debug, Clone, Copy)]
pub enum SetupSelection {
    Setup0 = 0b000 << 12,
    Setup1 = 0b001 << 12,
    Setup2 = 0b010 << 12,
    Setup3 = 0b011 << 12,
    Setup4 = 0b100 << 12,
    Setup5 = 0b101 << 12,
    Setup6 = 0b110 << 12,
    Setup7 = 0b111 << 12,
}

/// Enum for selecting the input pair for a channel.
#[derive(Debug, Clone, Copy)]
pub enum InputPair {
    Vin0Vin1 = 0b0000000001,
    Vin0VinCom = 0b0000010000,
    Vin1Vin0 = 0b0000100000,
    Vin1VinCom = 0b0000110000,
    Vin2Vin3 = 0b0001000011,
    Vin2VinCom = 0b0001010000,
    Vin3Vin2 = 0b0001100010,
    Vin3VinCom = 0b0001110000,
    Vin4Vin5 = 0b0010000101,
    Vin4VinCom = 0b0010010000,
    Vin5Vin4 = 0b0010100100,
    Vin5VinCom = 0b0010110000,
    Vin6Vin7 = 0b0011000111,
    Vin6VinCom = 0b0011010000,
    Vin7Vin6 = 0b0011100110,
    Vin7VinCom = 0b0011110000,
    Iin3PlusIin3Minus = 0b0110001011,
    Iin2PlusIin2Minus = 0b0110101010,
    Iin1PlusIin1Minus = 0b0111001001,
    Iin0PlusIin0Minus = 0b0111101000,
    TemperatureSensor = 0b1000110010,
    Reference = 0b1010110110,
}

/// Enum for setting the output coding of the ADC (bipolar/unipolar).
#[derive(Debug, Clone, Copy)]
pub enum BipolarUnipolar {
    Unipolar = 0b0 << 12,
    Bipolar = 0b1 << 12,
}

/// Enum for enabling/disabling the REF+ buffer.
#[derive(Debug, Clone, Copy)]
pub enum RefBufPlus {
    Disabled = 0b0 << 11,
    Enabled = 0b1 << 11,
}

/// Enum for enabling/disabling the REF− buffer.
#[derive(Debug, Clone, Copy)]
pub enum RefBufMinus {
    Disabled = 0b0 << 10,
    Enabled = 0b1 << 10,
}

/// Enum for enabling/disabling input buffers.
#[derive(Debug, Clone, Copy)]
pub enum InputBuffer {
    Disabled = 0b00 << 8,
    Enabled = 0b11 << 8,
}

/// Enum for selecting the reference source.
#[derive(Debug, Clone, Copy)]
pub enum ReferenceSelect {
    ExternalRef = 0b00 << 4,
    Internal2V5 = 0b10 << 4,
    AvddAvss = 0b11 << 4,
}

/// Enum for selecting enhanced filter mode.
#[derive(Debug, Clone, Copy)]
pub enum EnhancedFilter {
    Disabled = 0b0 << 11,
    Enabled = 0b1 << 11,
}

/// Enum for selecting the enhanced filter rejection options.
#[derive(Debug, Clone, Copy)]
pub enum EnhancedFilterSelection {
    Rejection27sps = 0b010 << 8,
    Rejection25sps = 0b011 << 8,
    Rejection20sps = 0b101 << 8,
    Rejection16_67sps = 0b110 << 8,
}

/// Enum for selecting the digital filter order.
#[derive(Debug, Clone, Copy)]
pub enum FilterOrder {
    Sinc5Sinc1 = 0b00 << 5,
    Sinc3 = 0b11 << 5,
}

/// Enum for setting the output data rate (ODR).
#[derive(Debug, Clone, Copy)]
pub enum OutputDataRate {
    Sps31250 = 0b00000,
    Sps15625 = 0b00110,
    Sps10417 = 0b00111,
    Sps5208 = 0b01000,
    Sps2597 = 0b01001,  // 2604.2 for sinc3
    Sps1007 = 0b01010,  // 1008.1 for sinc3
    Sps503_8 = 0b01011, // 504 for sinc3
    Sps381 = 0b01100,   // 401 for sinc3
    Sps200_3 = 0b01101,
    Sps100_2 = 0b01110,
    Sps59_52 = 0b01111, // 59.98 for sinc3
    Sps49_68 = 0b10000, // 50 for sinc3
    Sps20_01 = 0b10001,
    Sps16_63 = 0b10010, // 16.67 for sinc3
    Sps10 = 0b10011,
    Sps5 = 0b10100,
    Sps2_5 = 0b10101,
    Sps1_25 = 0b10110,
}

/// AD4112 Driver
pub struct AD4112Driver;

impl AD4112Driver {
    const N: u32 = 24;
    const MAX_CODE_UNIPOLAR: f64 = (1u64 << Self::N) as f64; // 2^N
    const MAX_CODE_BIPOLAR: f64 = (1u64 << (Self::N - 1)) as f64; // 2^(N-1)
    const CURRENT_RESISTANCE: f64 = 50.0; // 50Ω resistance for current inputs

    /// Creates a new instance of the AD4112 driver.
    pub fn new() -> Self {
        Self {}
    }

    pub fn convert_unipolar_voltage(code: u32, vref: f64) -> f64 {
        (code as f64 * vref) / (Self::MAX_CODE_UNIPOLAR * 0.1)
    }

    /// Build the communications register command byte.
    fn build_comms_byte(register: Register, read: bool) -> u8 {
        let mut cmd = (register as u8) & 0x3F; // RA bits to select register
        if read {
            cmd |= 0x40; // Set R/W bit to 1 for read
        }
        cmd
    }

    /// Transfer function to handle different data sizes (8, 16, 24, or 32 bits) in a single SPI transaction.
    async fn transfer_data<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        register: Register,
        data_size: usize,
        data: Option<u32>, // If it's a write operation, provide data
    ) -> Result<u32, AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        let comms_byte = Self::build_comms_byte(register, data.is_none());

        // Fixed-size buffers for SPI transfer (1 byte for comms + up to 4 bytes for data)
        let mut read_buffer = [0u8; 5]; // Maximum 5 bytes (1 byte comms + 4 bytes data)
        let mut write_buffer = [0u8; 5]; // Same size for writing

        write_buffer[0] = comms_byte; // First byte is the comms byte

        if let Some(data) = data {
            // For writing, populate the write buffer with the provided data
            for i in 0..data_size {
                write_buffer[data_size - i] = (data >> (8 * i)) as u8;
            }
        }

        // Perform SPI transfer: send the comms byte and the data in a single transaction
        cs.set_low().map_err(AD4112Error::PinError)?;
        spi.transfer(&mut read_buffer[..data_size + 1], &write_buffer[..data_size + 1])
            .await
            .map_err(AD4112Error::SpiError)?;
        cs.set_high().map_err(AD4112Error::PinError)?;

        // Combine the received bytes into a single u32 result (excluding the first byte)
        let mut result = 0u32;
        for i in 0..data_size {
            result |= (read_buffer[i + 1] as u32) << (8 * (data_size - 1 - i));
        }

        Ok(result)
    }

    /// Read from a register (with varying data sizes).
    pub async fn read_register<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        register: Register,
        data_size: usize,
    ) -> Result<u32, AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.transfer_data(spi, cs, register, data_size, None).await
    }

    /// Write to a register (with varying data sizes).
    pub async fn write_register<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        register: Register,
        data: u32,
        data_size: usize,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.transfer_data(spi, cs, register, data_size, Some(data)).await?;
        Ok(())
    }

    /// Reads the ADC data register (24 bits).
    pub async fn read_data<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Result<u32, AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        self.read_register(spi, cs, Register::Data, 3).await
    }

    /// Reads the device ID register (16 bits).
    pub async fn read_device_id<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Result<u16, AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        let id = self.read_register(spi, cs, Register::ID, 2).await?;
        Ok((id & 0xFFFF) as u16)
    }

    /// Reset communication in case of interface loss.
    pub async fn reset_communication<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // Send at least 64 clock cycles with DIN high (0xFF)
        let reset_bytes = [0xFFu8; 8];
        let mut response = [0u8; 8]; // Dummy read buffer for the transfer
        cs.set_low().map_err(AD4112Error::PinError)?;
        spi.transfer(&mut response, &reset_bytes)
            .await
            .map_err(AD4112Error::SpiError)?;
        cs.set_high().map_err(AD4112Error::PinError)?;

        Ok(())
    }

    /// Write to the ADCMODE register to configure the ADC operating mode and settings.
    pub async fn write_adc_mode<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        ref_en: RefEnable,
        single_cycle: SingleCycle,
        delay: Delay,
        mode: ADCModeOperation,
        clock: ClockSelect,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // Compose the 16-bit value for the ADCMODE register
        let adc_mode_value = (ref_en as u16) | (single_cycle as u16) | (delay as u16) | (mode as u16) | (clock as u16);

        // Write the value to the ADCMODE register (16 bits)
        self.write_register(spi, cs, Register::ADCMode, adc_mode_value as u32, 2)
            .await
    }

    /// Enable a channel and configure its input pair and setup selection.
    pub async fn enable_channel<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        channel: Register, // The channel register (e.g., CH0, CH1, etc.)
        enable: ChannelEnable,
        setup: SetupSelection,
        input_pair: InputPair,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // Compose the 16-bit value for the channel register
        let channel_value = (enable as u16) | (setup as u16) | (input_pair as u16);

        // Write the value to the channel register (16 bits)
        self.write_register(spi, cs, channel, channel_value as u32, 2).await
    }

    /// Configure a setup (SETUPCONx register).
    pub async fn configure_setup<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        setup: Register, // The setup register (e.g., SetupCon0, SetupCon1, etc.)
        bipolar: BipolarUnipolar,
        refbuf_plus: RefBufPlus,
        refbuf_minus: RefBufMinus,
        input_buffer: InputBuffer,
        ref_select: ReferenceSelect,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // Compose the 16-bit value for the setup register
        let setup_value = (bipolar as u16)
            | (refbuf_plus as u16)
            | (refbuf_minus as u16)
            | (input_buffer as u16)
            | (ref_select as u16);

        // Write the value to the setup register (16 bits)
        self.write_register(spi, cs, setup, setup_value as u32, 2).await
    }

    /// Configure a filter (FILTCONx register).
    pub async fn configure_filter<SPI, CS>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        filter: Register, // The filter register (e.g., FiltCon0, FiltCon1, etc.)
        enhanced_filter: EnhancedFilter,
        enhanced_filter_selection: EnhancedFilterSelection,
        filter_order: FilterOrder,
        output_data_rate: OutputDataRate,
    ) -> Result<(), AD4112Error<SPI::Error, CS::Error>>
    where
        SPI: SpiBus<u8>,
        CS: OutputPin,
    {
        // Compose the 16-bit value for the filter register
        let filter_value = (enhanced_filter as u16)
            | (enhanced_filter_selection as u16)
            | (filter_order as u16)
            | (output_data_rate as u16);

        // Write the value to the filter register (16 bits)
        self.write_register(spi, cs, filter, filter_value as u32, 2).await
    }
}
