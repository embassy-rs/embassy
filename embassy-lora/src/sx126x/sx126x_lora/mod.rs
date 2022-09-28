#![allow(dead_code)]

use embassy_time::{Duration, Timer};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiBus;

mod board_specific;
pub mod mod_params;
mod subroutine;

use mod_params::RadioError::*;
use mod_params::*;

// Syncwords for public and private networks
const LORA_MAC_PUBLIC_SYNCWORD: u16 = 0x3444;
const LORA_MAC_PRIVATE_SYNCWORD: u16 = 0x1424;

// Maximum number of registers that can be added to the retention list
const MAX_NUMBER_REGS_IN_RETENTION: u8 = 4;

// Possible LoRa bandwidths
const LORA_BANDWIDTHS: [Bandwidth; 3] = [Bandwidth::_125KHz, Bandwidth::_250KHz, Bandwidth::_500KHz];

// Radio complete wakeup time with margin for temperature compensation [ms]
const RADIO_WAKEUP_TIME: u32 = 3;

/// Provides high-level access to Semtech SX126x-based boards
pub struct LoRa<SPI, CS, RESET, ANTRX, ANTTX, WAIT> {
    spi: SPI,
    cs: CS,
    reset: RESET,
    antenna_rx: ANTRX,
    antenna_tx: ANTTX,
    dio1: WAIT,
    busy: WAIT,
    operating_mode: RadioMode,
    rx_continuous: bool,
    max_payload_length: u8,
    modulation_params: Option<ModulationParams>,
    packet_type: PacketType,
    packet_params: Option<PacketParams>,
    packet_status: Option<PacketStatus>,
    image_calibrated: bool,
    frequency_error: u32,
}

impl<SPI, CS, RESET, ANTRX, ANTTX, WAIT, BUS> LoRa<SPI, CS, RESET, ANTRX, ANTTX, WAIT>
where
    SPI: SpiBus<u8, Error = BUS>,
    CS: OutputPin,
    RESET: OutputPin,
    ANTRX: OutputPin,
    ANTTX: OutputPin,
    WAIT: Wait,
{
    /// Builds and returns a new instance of the radio. Only one instance of the radio should exist at a time ()
    pub fn new(spi: SPI, cs: CS, reset: RESET, antenna_rx: ANTRX, antenna_tx: ANTTX, dio1: WAIT, busy: WAIT) -> Self {
        Self {
            spi,
            cs,
            reset,
            antenna_rx,
            antenna_tx,
            dio1,
            busy,
            operating_mode: RadioMode::Sleep,
            rx_continuous: false,
            max_payload_length: 0xFFu8,
            modulation_params: None,
            packet_type: PacketType::LoRa,
            packet_params: None,
            packet_status: None,
            image_calibrated: false,
            frequency_error: 0u32, // where is volatile FrequencyError modified ???
        }
    }

    /// Initialize the radio
    pub async fn init(&mut self) -> Result<(), RadioError<BUS>> {
        self.sub_init().await?;
        self.sub_set_standby(StandbyMode::RC).await?;
        self.sub_set_regulator_mode(RegulatorMode::UseDCDC).await?;
        self.sub_set_buffer_base_address(0x00u8, 0x00u8).await?;
        self.sub_set_tx_params(0i8, RampTime::Ramp200Us).await?;
        self.sub_set_dio_irq_params(
            IrqMask::All.value(),
            IrqMask::All.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
        )
        .await?;
        self.add_register_to_retention_list(Register::RxGain.addr()).await?;
        self.add_register_to_retention_list(Register::TxModulation.addr())
            .await?;
        Ok(())
    }

    /// Return current radio state
    pub fn get_status(&mut self) -> RadioState {
        match self.brd_get_operating_mode() {
            RadioMode::Transmit => RadioState::TxRunning,
            RadioMode::Receive => RadioState::RxRunning,
            RadioMode::ChannelActivityDetection => RadioState::ChannelActivityDetecting,
            _ => RadioState::Idle,
        }
    }

    /// Configure the radio for LoRa (FSK support should be provided in a separate driver, if desired)
    pub async fn set_lora_modem(&mut self, enable_public_network: bool) -> Result<(), RadioError<BUS>> {
        self.sub_set_packet_type(PacketType::LoRa).await?;
        if enable_public_network {
            self.brd_write_registers(
                Register::LoRaSyncword,
                &[
                    ((LORA_MAC_PUBLIC_SYNCWORD >> 8) & 0xFF) as u8,
                    (LORA_MAC_PUBLIC_SYNCWORD & 0xFF) as u8,
                ],
            )
            .await?;
        } else {
            self.brd_write_registers(
                Register::LoRaSyncword,
                &[
                    ((LORA_MAC_PRIVATE_SYNCWORD >> 8) & 0xFF) as u8,
                    (LORA_MAC_PRIVATE_SYNCWORD & 0xFF) as u8,
                ],
            )
            .await?;
        }

        Ok(())
    }

    /// Sets the channel frequency
    pub async fn set_channel(&mut self, frequency: u32) -> Result<(), RadioError<BUS>> {
        self.sub_set_rf_frequency(frequency).await?;
        Ok(())
    }

    /* Checks if the channel is free for the given time.  This is currently not implemented until a substitute
        for switching to the FSK modem is found.

    pub async fn is_channel_free(&mut self, frequency: u32, rxBandwidth: u32, rssiThresh: i16, maxCarrierSenseTime: u32) -> bool;
    */

    /// Generate a 32 bit random value based on the RSSI readings, after disabling all interrupts.   Ensure set_lora_modem() is called befrorehand.
    /// After calling this function either set_rx_config() or set_tx_config() must be called.
    pub async fn get_random_value(&mut self) -> Result<u32, RadioError<BUS>> {
        self.sub_set_dio_irq_params(
            IrqMask::None.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
        )
        .await?;

        let result = self.sub_get_random().await?;
        Ok(result)
    }

    /// Set the reception parameters for the LoRa modem (only).  Ensure set_lora_modem() is called befrorehand.
    ///   spreading_factor     [6: 64, 7: 128, 8: 256, 9: 512, 10: 1024, 11: 2048, 12: 4096 chips/symbol]
    ///   bandwidth            [0: 125 kHz, 1: 250 kHz, 2: 500 kHz, 3: Reserved]
    ///   coding_rate          [1: 4/5, 2: 4/6, 3: 4/7, 4: 4/8]
    ///   preamble_length      length in symbols (the hardware adds 4 more symbols)
    ///   symb_timeout         RxSingle timeout value in symbols
    ///   fixed_len            fixed length packets [0: variable, 1: fixed]
    ///   payload_len          payload length when fixed length is used
    ///   crc_on               [0: OFF, 1: ON]
    ///   freq_hop_on          intra-packet frequency hopping [0: OFF, 1: ON]
    ///   hop_period           number of symbols between each hop
    ///   iq_inverted          invert IQ signals [0: not inverted, 1: inverted]
    ///   rx_continuous        reception mode [false: single mode, true: continuous mode]
    pub async fn set_rx_config(
        &mut self,
        spreading_factor: SpreadingFactor,
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        preamble_length: u16,
        symb_timeout: u16,
        fixed_len: bool,
        payload_len: u8,
        crc_on: bool,
        _freq_hop_on: bool,
        _hop_period: u8,
        iq_inverted: bool,
        rx_continuous: bool,
    ) -> Result<(), RadioError<BUS>> {
        let mut symb_timeout_final = symb_timeout;

        self.rx_continuous = rx_continuous;
        if self.rx_continuous {
            symb_timeout_final = 0;
        }
        if fixed_len {
            self.max_payload_length = payload_len;
        } else {
            self.max_payload_length = 0xFFu8;
        }

        self.sub_set_stop_rx_timer_on_preamble_detect(false).await?;

        let mut low_data_rate_optimize = 0x00u8;
        if (((spreading_factor == SpreadingFactor::_11) || (spreading_factor == SpreadingFactor::_12))
            && (bandwidth == Bandwidth::_125KHz))
            || ((spreading_factor == SpreadingFactor::_12) && (bandwidth == Bandwidth::_250KHz))
        {
            low_data_rate_optimize = 0x01u8;
        }

        let modulation_params = ModulationParams {
            spreading_factor: spreading_factor,
            bandwidth: bandwidth,
            coding_rate: coding_rate,
            low_data_rate_optimize: low_data_rate_optimize,
        };

        let mut preamble_length_final = preamble_length;
        if ((spreading_factor == SpreadingFactor::_5) || (spreading_factor == SpreadingFactor::_6))
            && (preamble_length < 12)
        {
            preamble_length_final = 12;
        }

        let packet_params = PacketParams {
            preamble_length: preamble_length_final,
            implicit_header: fixed_len,
            payload_length: self.max_payload_length,
            crc_on: crc_on,
            iq_inverted: iq_inverted,
        };

        self.modulation_params = Some(modulation_params);
        self.packet_params = Some(packet_params);

        self.standby().await?;
        self.sub_set_modulation_params().await?;
        self.sub_set_packet_params().await?;
        self.sub_set_lora_symb_num_timeout(symb_timeout_final).await?;

        // Optimize the Inverted IQ Operation (see DS_SX1261-2_V1.2 datasheet chapter 15.4)
        let mut iq_polarity = [0x00u8];
        self.brd_read_registers(Register::IQPolarity, &mut iq_polarity).await?;
        if iq_inverted {
            self.brd_write_registers(Register::IQPolarity, &[iq_polarity[0] & (!(1 << 2))])
                .await?;
        } else {
            self.brd_write_registers(Register::IQPolarity, &[iq_polarity[0] | (1 << 2)])
                .await?;
        }
        Ok(())
    }

    /// Set the transmission parameters for the LoRa modem (only).
    ///   power                output power [dBm]
    ///   spreading_factor     [6: 64, 7: 128, 8: 256, 9: 512, 10: 1024, 11: 2048, 12: 4096 chips/symbol]
    ///   bandwidth            [0: 125 kHz, 1: 250 kHz, 2: 500 kHz, 3: Reserved]
    ///   coding_rate          [1: 4/5, 2: 4/6, 3: 4/7, 4: 4/8]
    ///   preamble_length      length in symbols (the hardware adds 4 more symbols)
    ///   fixed_len            fixed length packets [0: variable, 1: fixed]
    ///   crc_on               [0: OFF, 1: ON]
    ///   freq_hop_on          intra-packet frequency hopping [0: OFF, 1: ON]
    ///   hop_period           number of symbols between each hop
    ///   iq_inverted          invert IQ signals [0: not inverted, 1: inverted]
    pub async fn set_tx_config(
        &mut self,
        power: i8,
        spreading_factor: SpreadingFactor,
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        preamble_length: u16,
        fixed_len: bool,
        crc_on: bool,
        _freq_hop_on: bool,
        _hop_period: u8,
        iq_inverted: bool,
    ) -> Result<(), RadioError<BUS>> {
        let mut low_data_rate_optimize = 0x00u8;
        if (((spreading_factor == SpreadingFactor::_11) || (spreading_factor == SpreadingFactor::_12))
            && (bandwidth == Bandwidth::_125KHz))
            || ((spreading_factor == SpreadingFactor::_12) && (bandwidth == Bandwidth::_250KHz))
        {
            low_data_rate_optimize = 0x01u8;
        }

        let modulation_params = ModulationParams {
            spreading_factor: spreading_factor,
            bandwidth: bandwidth,
            coding_rate: coding_rate,
            low_data_rate_optimize: low_data_rate_optimize,
        };

        let mut preamble_length_final = preamble_length;
        if ((spreading_factor == SpreadingFactor::_5) || (spreading_factor == SpreadingFactor::_6))
            && (preamble_length < 12)
        {
            preamble_length_final = 12;
        }

        let packet_params = PacketParams {
            preamble_length: preamble_length_final,
            implicit_header: fixed_len,
            payload_length: self.max_payload_length,
            crc_on: crc_on,
            iq_inverted: iq_inverted,
        };

        self.modulation_params = Some(modulation_params);
        self.packet_params = Some(packet_params);

        self.standby().await?;
        self.sub_set_modulation_params().await?;
        self.sub_set_packet_params().await?;

        // Handle modulation quality with the 500 kHz LoRa bandwidth (see DS_SX1261-2_V1.2 datasheet chapter 15.1)

        let mut tx_modulation = [0x00u8];
        self.brd_read_registers(Register::TxModulation, &mut tx_modulation)
            .await?;
        if bandwidth == Bandwidth::_500KHz {
            self.brd_write_registers(Register::TxModulation, &[tx_modulation[0] & (!(1 << 2))])
                .await?;
        } else {
            self.brd_write_registers(Register::TxModulation, &[tx_modulation[0] | (1 << 2)])
                .await?;
        }

        self.brd_set_rf_tx_power(power).await?;
        Ok(())
    }

    /// Check if the given RF frequency is supported by the hardware [true: supported, false: unsupported]
    pub async fn check_rf_frequency(&mut self, frequency: u32) -> Result<bool, RadioError<BUS>> {
        Ok(self.brd_check_rf_frequency(frequency).await?)
    }

    /// Computes the packet time on air in ms for the given payload for a LoRa modem (can only be called once set_rx_config or set_tx_config have been called)
    ///   spreading_factor     [6: 64, 7: 128, 8: 256, 9: 512, 10: 1024, 11: 2048, 12: 4096 chips/symbol]
    ///   bandwidth            [0: 125 kHz, 1: 250 kHz, 2: 500 kHz, 3: Reserved]
    ///   coding_rate          [1: 4/5, 2: 4/6, 3: 4/7, 4: 4/8]
    ///   preamble_length      length in symbols (the hardware adds 4 more symbols)
    ///   fixed_len            fixed length packets [0: variable, 1: fixed]
    ///   payload_len          sets payload length when fixed length is used
    ///   crc_on               [0: OFF, 1: ON]
    pub fn get_time_on_air(
        &mut self,
        spreading_factor: SpreadingFactor,
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        preamble_length: u16,
        fixed_len: bool,
        payload_len: u8,
        crc_on: bool,
    ) -> Result<u32, RadioError<BUS>> {
        let numerator = 1000
            * Self::get_lora_time_on_air_numerator(
                spreading_factor,
                bandwidth,
                coding_rate,
                preamble_length,
                fixed_len,
                payload_len,
                crc_on,
            );
        let denominator = bandwidth.value_in_hz();
        if denominator == 0 {
            Err(RadioError::InvalidBandwidth)
        } else {
            Ok((numerator + denominator - 1) / denominator)
        }
    }

    /// Send the buffer of the given size. Prepares the packet to be sent and sets the radio in transmission [timeout in ms]
    pub async fn send(&mut self, buffer: &[u8], timeout: u32) -> Result<(), RadioError<BUS>> {
        if self.packet_params.is_some() {
            self.sub_set_dio_irq_params(
                IrqMask::TxDone.value() | IrqMask::RxTxTimeout.value(),
                IrqMask::TxDone.value() | IrqMask::RxTxTimeout.value(),
                IrqMask::None.value(),
                IrqMask::None.value(),
            )
            .await?;

            let mut packet_params = self.packet_params.as_mut().unwrap();
            packet_params.payload_length = buffer.len() as u8;
            self.sub_set_packet_params().await?;
            self.sub_send_payload(buffer, timeout).await?;
            Ok(())
        } else {
            Err(RadioError::PacketParamsMissing)
        }
    }

    /// Set the radio in sleep mode
    pub async fn sleep(&mut self) -> Result<(), RadioError<BUS>> {
        self.sub_set_sleep(SleepParams {
            wakeup_rtc: false,
            reset: false,
            warm_start: true,
        })
        .await?;
        Timer::after(Duration::from_millis(2)).await;
        Ok(())
    }

    /// Set the radio in standby mode
    pub async fn standby(&mut self) -> Result<(), RadioError<BUS>> {
        self.sub_set_standby(StandbyMode::RC).await?;
        Ok(())
    }

    /// Set the radio in reception mode for the given duration [0: continuous, others: timeout (ms)]       
    pub async fn rx(&mut self, timeout: u32) -> Result<(), RadioError<BUS>> {
        self.sub_set_dio_irq_params(
            IrqMask::All.value(),
            IrqMask::All.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
        )
        .await?;

        if self.rx_continuous {
            self.sub_set_rx(0xFFFFFF).await?;
        } else {
            self.sub_set_rx(timeout << 6).await?;
        }

        Ok(())
    }

    /// Start a Channel Activity Detection
    pub async fn start_cad(&mut self) -> Result<(), RadioError<BUS>> {
        self.sub_set_dio_irq_params(
            IrqMask::CADDone.value() | IrqMask::CADActivityDetected.value(),
            IrqMask::CADDone.value() | IrqMask::CADActivityDetected.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
        )
        .await?;
        self.sub_set_cad().await?;
        Ok(())
    }

    /// Sets the radio in continuous wave transmission mode
    ///   frequency    channel RF frequency
    ///   power        output power [dBm]
    ///   timeout      transmission mode timeout [s]
    pub async fn set_tx_continuous_wave(
        &mut self,
        frequency: u32,
        power: i8,
        _timeout: u16,
    ) -> Result<(), RadioError<BUS>> {
        self.sub_set_rf_frequency(frequency).await?;
        self.brd_set_rf_tx_power(power).await?;
        self.sub_set_tx_continuous_wave().await?;

        Ok(())
    }

    /// Read the current RSSI value for the LoRa modem (only) [dBm]
    pub async fn get_rssi(&mut self) -> Result<i16, RadioError<BUS>> {
        let value = self.sub_get_rssi_inst().await?;
        Ok(value as i16)
    }

    /// Write one or more radio registers with a buffer of a given size, starting at the first register address
    pub async fn write_registers_from_buffer(
        &mut self,
        start_register: Register,
        buffer: &[u8],
    ) -> Result<(), RadioError<BUS>> {
        self.brd_write_registers(start_register, buffer).await?;
        Ok(())
    }

    /// Read one or more radio registers into a buffer of a given size, starting at the first register address
    pub async fn read_registers_into_buffer(
        &mut self,
        start_register: Register,
        buffer: &mut [u8],
    ) -> Result<(), RadioError<BUS>> {
        self.brd_read_registers(start_register, buffer).await?;
        Ok(())
    }

    /// Set the maximum payload length (in bytes) for a LoRa modem (only).
    pub async fn set_max_payload_length(&mut self, max: u8) -> Result<(), RadioError<BUS>> {
        if self.packet_params.is_some() {
            let packet_params = self.packet_params.as_mut().unwrap();
            self.max_payload_length = max;
            packet_params.payload_length = max;
            self.sub_set_packet_params().await?;
            Ok(())
        } else {
            Err(RadioError::PacketParamsMissing)
        }
    }

    /// Get the time required for the board plus radio to get out of sleep [ms]
    pub fn get_wakeup_time(&mut self) -> u32 {
        self.brd_get_board_tcxo_wakeup_time() + RADIO_WAKEUP_TIME
    }

    /// Process the radio irq
    pub async fn process_irq(
        &mut self,
        receiving_buffer: Option<&mut [u8]>,
        received_len: Option<&mut u8>,
        cad_activity_detected: Option<&mut bool>,
    ) -> Result<(), RadioError<BUS>> {
        loop {
            trace!("process_irq loop entered");

            let de = self.sub_get_device_errors().await?;
            trace!("device_errors: rc_64khz_calibration = {}, rc_13mhz_calibration = {}, pll_calibration = {}, adc_calibration = {}, image_calibration = {}, xosc_start = {}, pll_lock = {}, pa_ramp = {}",
                               de.rc_64khz_calibration, de.rc_13mhz_calibration, de.pll_calibration, de.adc_calibration, de.image_calibration, de.xosc_start, de.pll_lock, de.pa_ramp);
            let st = self.sub_get_status().await?;
            trace!(
                "radio status: cmd_status: {:x}, chip_mode: {:x}",
                st.cmd_status,
                st.chip_mode
            );

            self.dio1.wait_for_high().await.map_err(|_| DIO1)?;
            let operating_mode = self.brd_get_operating_mode();
            let irq_flags = self.sub_get_irq_status().await?;
            self.sub_clear_irq_status(irq_flags).await?;
            trace!("process_irq DIO1 satisfied: irq_flags = {:x}", irq_flags);

            // check for errors and unexpected interrupt masks (based on operation mode)
            if (irq_flags & IrqMask::HeaderError.value()) == IrqMask::HeaderError.value() {
                if !self.rx_continuous {
                    self.brd_set_operating_mode(RadioMode::StandbyRC);
                }
                return Err(RadioError::HeaderError);
            } else if (irq_flags & IrqMask::CRCError.value()) == IrqMask::CRCError.value() {
                if operating_mode == RadioMode::Receive {
                    if !self.rx_continuous {
                        self.brd_set_operating_mode(RadioMode::StandbyRC);
                    }
                    return Err(RadioError::CRCErrorOnReceive);
                } else {
                    return Err(RadioError::CRCErrorUnexpected);
                }
            } else if (irq_flags & IrqMask::RxTxTimeout.value()) == IrqMask::RxTxTimeout.value() {
                if operating_mode == RadioMode::Transmit {
                    self.brd_set_operating_mode(RadioMode::StandbyRC);
                    return Err(RadioError::TransmitTimeout);
                } else if operating_mode == RadioMode::Receive {
                    self.brd_set_operating_mode(RadioMode::StandbyRC);
                    return Err(RadioError::ReceiveTimeout);
                } else {
                    return Err(RadioError::TimeoutUnexpected);
                }
            } else if ((irq_flags & IrqMask::TxDone.value()) == IrqMask::TxDone.value())
                && (operating_mode != RadioMode::Transmit)
            {
                return Err(RadioError::TransmitDoneUnexpected);
            } else if ((irq_flags & IrqMask::RxDone.value()) == IrqMask::RxDone.value())
                && (operating_mode != RadioMode::Receive)
            {
                return Err(RadioError::ReceiveDoneUnexpected);
            } else if (((irq_flags & IrqMask::CADActivityDetected.value()) == IrqMask::CADActivityDetected.value())
                || ((irq_flags & IrqMask::CADDone.value()) == IrqMask::CADDone.value()))
                && (operating_mode != RadioMode::ChannelActivityDetection)
            {
                return Err(RadioError::CADUnexpected);
            }

            if (irq_flags & IrqMask::HeaderValid.value()) == IrqMask::HeaderValid.value() {
                trace!("HeaderValid");
            } else if (irq_flags & IrqMask::PreambleDetected.value()) == IrqMask::PreambleDetected.value() {
                trace!("PreambleDetected");
            } else if (irq_flags & IrqMask::SyncwordValid.value()) == IrqMask::SyncwordValid.value() {
                trace!("SyncwordValid");
            }

            // handle completions
            if (irq_flags & IrqMask::TxDone.value()) == IrqMask::TxDone.value() {
                self.brd_set_operating_mode(RadioMode::StandbyRC);
                return Ok(());
            } else if (irq_flags & IrqMask::RxDone.value()) == IrqMask::RxDone.value() {
                if !self.rx_continuous {
                    self.brd_set_operating_mode(RadioMode::StandbyRC);

                    // implicit header mode timeout behavior (see DS_SX1261-2_V1.2 datasheet chapter 15.3)
                    self.brd_write_registers(Register::RTCCtrl, &[0x00]).await?;
                    let mut evt_clr = [0x00u8];
                    self.brd_read_registers(Register::EvtClr, &mut evt_clr).await?;
                    evt_clr[0] |= 1 << 1;
                    self.brd_write_registers(Register::EvtClr, &evt_clr).await?;
                }

                if receiving_buffer.is_some() && received_len.is_some() {
                    *(received_len.unwrap()) = self.sub_get_payload(receiving_buffer.unwrap()).await?;
                }
                self.packet_status = self.sub_get_packet_status().await?.into();
                return Ok(());
            } else if (irq_flags & IrqMask::CADDone.value()) == IrqMask::CADDone.value() {
                if cad_activity_detected.is_some() {
                    *(cad_activity_detected.unwrap()) =
                        (irq_flags & IrqMask::CADActivityDetected.value()) == IrqMask::CADActivityDetected.value();
                }
                self.brd_set_operating_mode(RadioMode::StandbyRC);
                return Ok(());
            }

            // if DIO1 was driven high for reasons other than an error or operation completion (currently, PreambleDetected, SyncwordValid, and HeaderValid
            // are in that category), loop to wait again
        }
    }

    // SX126x-specific functions

    /// Set the radio in reception mode with Max LNA gain for the given time (SX126x radios only) [0: continuous, others timeout in ms]
    pub async fn set_rx_boosted(&mut self, timeout: u32) -> Result<(), RadioError<BUS>> {
        self.sub_set_dio_irq_params(
            IrqMask::All.value(),
            IrqMask::All.value(),
            IrqMask::None.value(),
            IrqMask::None.value(),
        )
        .await?;

        if self.rx_continuous {
            self.sub_set_rx_boosted(0xFFFFFF).await?; // Rx continuous
        } else {
            self.sub_set_rx_boosted(timeout << 6).await?;
        }

        Ok(())
    }

    /// Set the Rx duty cycle management parameters (SX126x radios only)
    ///   rx_time       structure describing reception timeout value
    ///   sleep_time    structure describing sleep timeout value
    pub async fn set_rx_duty_cycle(&mut self, rx_time: u32, sleep_time: u32) -> Result<(), RadioError<BUS>> {
        self.sub_set_rx_duty_cycle(rx_time, sleep_time).await?;
        Ok(())
    }

    pub fn get_latest_packet_status(&mut self) -> Option<PacketStatus> {
        self.packet_status
    }

    // Utilities

    async fn add_register_to_retention_list(&mut self, register_address: u16) -> Result<(), RadioError<BUS>> {
        let mut buffer = [0x00u8; (1 + (2 * MAX_NUMBER_REGS_IN_RETENTION)) as usize];

        // Read the address and registers already added to the list
        self.brd_read_registers(Register::RetentionList, &mut buffer).await?;

        let number_of_registers = buffer[0];
        for i in 0..number_of_registers {
            if register_address
                == ((buffer[(1 + (2 * i)) as usize] as u16) << 8) | (buffer[(2 + (2 * i)) as usize] as u16)
            {
                return Ok(()); // register already in list
            }
        }

        if number_of_registers < MAX_NUMBER_REGS_IN_RETENTION {
            buffer[0] += 1; // increment number of registers

            buffer[(1 + (2 * number_of_registers)) as usize] = ((register_address >> 8) & 0xFF) as u8;
            buffer[(2 + (2 * number_of_registers)) as usize] = (register_address & 0xFF) as u8;
            self.brd_write_registers(Register::RetentionList, &buffer).await?;

            Ok(())
        } else {
            Err(RadioError::RetentionListExceeded)
        }
    }

    fn get_lora_time_on_air_numerator(
        spreading_factor: SpreadingFactor,
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        preamble_length: u16,
        fixed_len: bool,
        payload_len: u8,
        crc_on: bool,
    ) -> u32 {
        let cell_denominator;
        let cr_denominator = (coding_rate.value() as i32) + 4;

        // Ensure that the preamble length is at least 12 symbols when using SF5 or SF6
        let mut preamble_length_final = preamble_length;
        if ((spreading_factor == SpreadingFactor::_5) || (spreading_factor == SpreadingFactor::_6))
            && (preamble_length < 12)
        {
            preamble_length_final = 12;
        }

        let mut low_data_rate_optimize = false;
        if (((spreading_factor == SpreadingFactor::_11) || (spreading_factor == SpreadingFactor::_12))
            && (bandwidth == Bandwidth::_125KHz))
            || ((spreading_factor == SpreadingFactor::_12) && (bandwidth == Bandwidth::_250KHz))
        {
            low_data_rate_optimize = true;
        }

        let mut cell_numerator = ((payload_len as i32) << 3) + (if crc_on { 16 } else { 0 })
            - (4 * spreading_factor.value() as i32)
            + (if fixed_len { 0 } else { 20 });

        if spreading_factor.value() <= 6 {
            cell_denominator = 4 * (spreading_factor.value() as i32);
        } else {
            cell_numerator += 8;
            if low_data_rate_optimize {
                cell_denominator = 4 * ((spreading_factor.value() as i32) - 2);
            } else {
                cell_denominator = 4 * (spreading_factor.value() as i32);
            }
        }

        if cell_numerator < 0 {
            cell_numerator = 0;
        }

        let mut intermediate: i32 = (((cell_numerator + cell_denominator - 1) / cell_denominator) * cr_denominator)
            + (preamble_length_final as i32)
            + 12;

        if spreading_factor.value() <= 6 {
            intermediate = intermediate + 2;
        }

        (((4 * intermediate) + 1) * (1 << (spreading_factor.value() - 2))) as u32
    }
}
