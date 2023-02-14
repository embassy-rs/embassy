use core::future::Future;

use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::*;
use lorawan_device::async_device::radio::{PhyRxTx, RfConfig, RxQuality, TxConfig};
use lorawan_device::async_device::Timings;

mod sx126x_lora;
use sx126x_lora::LoRa;

use self::sx126x_lora::mod_params::RadioError;

/// Semtech Sx126x LoRa peripheral
pub struct Sx126xRadio<SPI, CTRL, WAIT, BUS>
where
    SPI: SpiBus<u8, Error = BUS> + 'static,
    CTRL: OutputPin + 'static,
    WAIT: Wait + 'static,
    BUS: Error + Format + 'static,
{
    pub lora: LoRa<SPI, CTRL, WAIT>,
}

impl<SPI, CTRL, WAIT, BUS> Sx126xRadio<SPI, CTRL, WAIT, BUS>
where
    SPI: SpiBus<u8, Error = BUS> + 'static,
    CTRL: OutputPin + 'static,
    WAIT: Wait + 'static,
    BUS: Error + Format + 'static,
{
    pub async fn new(
        spi: SPI,
        cs: CTRL,
        reset: CTRL,
        antenna_rx: Option<CTRL>,
        antenna_tx: Option<CTRL>,
        dio1: WAIT,
        busy: WAIT,
        enable_public_network: bool,
    ) -> Result<Self, RadioError<BUS>> {
        let mut lora = LoRa::new(spi, cs, reset, antenna_rx, antenna_tx, dio1, busy);
        lora.init().await?;
        lora.set_lora_modem(enable_public_network).await?;
        Ok(Self { lora })
    }
}

impl<SPI, CTRL, WAIT, BUS> Timings for Sx126xRadio<SPI, CTRL, WAIT, BUS>
where
    SPI: SpiBus<u8, Error = BUS> + 'static,
    CTRL: OutputPin + 'static,
    WAIT: Wait + 'static,
    BUS: Error + Format + 'static,
{
    fn get_rx_window_offset_ms(&self) -> i32 {
        -3
    }
    fn get_rx_window_duration_ms(&self) -> u32 {
        1003
    }
}

impl<SPI, CTRL, WAIT, BUS> PhyRxTx for Sx126xRadio<SPI, CTRL, WAIT, BUS>
where
    SPI: SpiBus<u8, Error = BUS> + 'static,
    CTRL: OutputPin + 'static,
    WAIT: Wait + 'static,
    BUS: Error + Format + 'static,
{
    type PhyError = RadioError<BUS>;

    type TxFuture<'m> = impl Future<Output = Result<u32, Self::PhyError>> + 'm
    where
        SPI: 'm,
        CTRL: 'm,
        WAIT: 'm,
        BUS: 'm;

    fn tx<'m>(&'m mut self, config: TxConfig, buffer: &'m [u8]) -> Self::TxFuture<'m> {
        trace!("TX START");
        async move {
            self.lora
                .set_tx_config(
                    config.pw,
                    config.rf.spreading_factor.into(),
                    config.rf.bandwidth.into(),
                    config.rf.coding_rate.into(),
                    8,
                    false,
                    true,
                    false,
                    0,
                    false,
                )
                .await?;
            self.lora.set_max_payload_length(buffer.len() as u8).await?;
            self.lora.set_channel(config.rf.frequency).await?;
            self.lora.send(buffer, 0xffffff).await?;
            self.lora.process_irq(None, None, None).await?;
            trace!("TX DONE");
            return Ok(0);
        }
    }

    type RxFuture<'m> = impl Future<Output = Result<(usize, RxQuality), Self::PhyError>> + 'm
    where
        SPI: 'm,
        CTRL: 'm,
        WAIT: 'm,
        BUS: 'm;
    fn rx<'m>(&'m mut self, config: RfConfig, receiving_buffer: &'m mut [u8]) -> Self::RxFuture<'m> {
        trace!("RX START");
        async move {
            self.lora
                .set_rx_config(
                    config.spreading_factor.into(),
                    config.bandwidth.into(),
                    config.coding_rate.into(),
                    8,
                    4,
                    false,
                    0u8,
                    true,
                    false,
                    0,
                    true,
                    true,
                )
                .await?;
            self.lora.set_max_payload_length(receiving_buffer.len() as u8).await?;
            self.lora.set_channel(config.frequency).await?;
            self.lora.rx(90 * 1000).await?;
            let mut received_len = 0u8;
            self.lora
                .process_irq(Some(receiving_buffer), Some(&mut received_len), None)
                .await?;
            trace!("RX DONE");

            let packet_status = self.lora.get_latest_packet_status();
            let mut rssi = 0i16;
            let mut snr = 0i8;
            if packet_status.is_some() {
                rssi = packet_status.unwrap().rssi as i16;
                snr = packet_status.unwrap().snr;
            }

            Ok((received_len as usize, RxQuality::new(rssi, snr)))
        }
    }
}
