use core::future::Future;
use embassy::traits::gpio::WaitForRisingEdge;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use lorawan_device::async_device::{
    radio::{Bandwidth, PhyRxTx, RfConfig, RxQuality, SpreadingFactor, TxConfig},
    Timings,
};

mod sx127x_lora;
use sx127x_lora::{Error as RadioError, LoRa, RadioMode, IRQ};

/// Trait representing a radio switch for boards using the Sx127x radio. One some
/// boards, this will be a dummy implementation that does nothing.
pub trait RadioSwitch {
    fn set_tx(&mut self);
    fn set_rx(&mut self);
}

/// Semtech Sx127x radio peripheral
pub struct Sx127xRadio<SPI, CS, RESET, E, I, RFS>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + 'static,
    E: 'static,
    CS: OutputPin + 'static,
    RESET: OutputPin + 'static,
    I: WaitForRisingEdge + 'static,
    RFS: RadioSwitch + 'static,
{
    radio: LoRa<SPI, CS, RESET>,
    rfs: RFS,
    irq: I,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Idle,
    Txing,
    Rxing,
}

impl<SPI, CS, RESET, E, I, RFS> Sx127xRadio<SPI, CS, RESET, E, I, RFS>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + 'static,
    CS: OutputPin + 'static,
    RESET: OutputPin + 'static,
    I: WaitForRisingEdge + 'static,
    RFS: RadioSwitch + 'static,
{
    pub fn new<D: DelayMs<u32>>(
        spi: SPI,
        cs: CS,
        reset: RESET,
        irq: I,
        rfs: RFS,
        d: &mut D,
    ) -> Result<Self, RadioError<E, CS::Error, RESET::Error>> {
        let mut radio = LoRa::new(spi, cs, reset);
        radio.reset(d)?;
        Ok(Self { radio, irq, rfs })
    }
}

impl<SPI, CS, RESET, E, I, RFS> Timings for Sx127xRadio<SPI, CS, RESET, E, I, RFS>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + 'static,
    CS: OutputPin + 'static,
    RESET: OutputPin + 'static,
    I: WaitForRisingEdge + 'static,
    RFS: RadioSwitch + 'static,
{
    fn get_rx_window_offset_ms(&self) -> i32 {
        -500
    }
    fn get_rx_window_duration_ms(&self) -> u32 {
        800
    }
}

impl<SPI, CS, RESET, E, I, RFS> PhyRxTx for Sx127xRadio<SPI, CS, RESET, E, I, RFS>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + 'static,
    CS: OutputPin + 'static,
    E: 'static,
    RESET: OutputPin + 'static,
    I: WaitForRisingEdge + 'static,
    RFS: RadioSwitch + 'static,
{
    type PhyError = Sx127xError;

    #[rustfmt::skip]
    type TxFuture<'m> where SPI: 'm, CS: 'm, RESET: 'm, E: 'm, I: 'm, RFS: 'm = impl Future<Output = Result<u32, Self::PhyError>> + 'm;

    fn tx<'m>(&'m mut self, config: TxConfig, buf: &'m [u8]) -> Self::TxFuture<'m> {
        trace!("TX START");
        async move {
            self.rfs.set_tx();
            self.radio.set_tx_power(14, 0)?;
            self.radio.set_frequency(config.rf.frequency)?;
            // TODO: Modify radio to support other coding rates
            self.radio.set_coding_rate_4(5)?;
            self.radio
                .set_signal_bandwidth(bandwidth_to_i64(config.rf.bandwidth))?;
            self.radio
                .set_spreading_factor(spreading_factor_to_u8(config.rf.spreading_factor))?;

            self.radio.set_preamble_length(8)?;
            self.radio.set_lora_pa_ramp()?;
            self.radio.set_lora_sync_word()?;
            self.radio.set_invert_iq(false)?;
            self.radio.set_crc(true)?;

            self.radio.set_dio0_tx_done()?;
            self.radio.transmit_payload(buf)?;

            loop {
                self.irq.wait_for_rising_edge().await;
                self.radio.set_mode(RadioMode::Stdby).ok().unwrap();
                let irq = self.radio.clear_irq().ok().unwrap();
                if (irq & IRQ::IrqTxDoneMask.addr()) != 0 {
                    trace!("TX DONE");
                    return Ok(0);
                }
            }
        }
    }

    #[rustfmt::skip]
    type RxFuture<'m> where SPI: 'm, CS: 'm, RESET: 'm, E: 'm, I: 'm, RFS: 'm = impl Future<Output = Result<(usize, RxQuality), Self::PhyError>> + 'm;

    fn rx<'m>(&'m mut self, config: RfConfig, buf: &'m mut [u8]) -> Self::RxFuture<'m> {
        trace!("RX START");
        async move {
            self.rfs.set_rx();
            self.radio.reset_payload_length()?;
            self.radio.set_frequency(config.frequency)?;
            // TODO: Modify radio to support other coding rates
            self.radio.set_coding_rate_4(5)?;
            self.radio
                .set_signal_bandwidth(bandwidth_to_i64(config.bandwidth))?;
            self.radio
                .set_spreading_factor(spreading_factor_to_u8(config.spreading_factor))?;

            self.radio.set_preamble_length(8)?;
            self.radio.set_lora_sync_word()?;
            self.radio.set_invert_iq(true)?;
            self.radio.set_crc(true)?;

            self.radio.set_dio0_rx_done()?;
            self.radio.set_mode(RadioMode::RxContinuous)?;

            loop {
                self.irq.wait_for_rising_edge().await;
                self.radio.set_mode(RadioMode::Stdby).ok().unwrap();
                let irq = self.radio.clear_irq().ok().unwrap();
                if (irq & IRQ::IrqRxDoneMask.addr()) != 0 {
                    let rssi = self.radio.get_packet_rssi().unwrap_or(0) as i16;
                    let snr = self.radio.get_packet_snr().unwrap_or(0.0) as i8;
                    let response = if let Ok(size) = self.radio.read_packet_size() {
                        self.radio.read_packet(buf)?;
                        Ok((size, RxQuality::new(rssi, snr)))
                    } else {
                        Ok((0, RxQuality::new(rssi, snr)))
                    };
                    trace!("RX DONE");
                    return response;
                }
            }
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Sx127xError;

impl<A, B, C> From<sx127x_lora::Error<A, B, C>> for Sx127xError {
    fn from(_: sx127x_lora::Error<A, B, C>) -> Self {
        Sx127xError
    }
}

fn spreading_factor_to_u8(sf: SpreadingFactor) -> u8 {
    match sf {
        SpreadingFactor::_7 => 7,
        SpreadingFactor::_8 => 8,
        SpreadingFactor::_9 => 9,
        SpreadingFactor::_10 => 10,
        SpreadingFactor::_11 => 11,
        SpreadingFactor::_12 => 12,
    }
}

fn bandwidth_to_i64(bw: Bandwidth) -> i64 {
    match bw {
        Bandwidth::_125KHz => 125_000,
        Bandwidth::_250KHz => 250_000,
        Bandwidth::_500KHz => 500_000,
    }
}
