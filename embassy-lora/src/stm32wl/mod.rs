//! A radio driver integration for the radio found on STM32WL family devices.
use core::future::{poll_fn, Future};
use core::task::Poll;

use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_stm32::dma::NoDma;
use embassy_stm32::interrupt::{Interrupt, InterruptExt, SUBGHZ_RADIO};
use embassy_stm32::subghz::{
    CalibrateImage, CfgIrq, CodingRate, Error, HeaderType, HseTrim, Irq, LoRaBandwidth, LoRaModParams,
    LoRaPacketParams, LoRaSyncWord, Ocp, PaConfig, PacketType, RegMode, RfFreq, SpreadingFactor as SF, StandbyClk,
    Status, SubGhz, TcxoMode, TcxoTrim, Timeout, TxParams,
};
use embassy_sync::waitqueue::AtomicWaker;
use lorawan_device::async_device::radio::{Bandwidth, PhyRxTx, RfConfig, RxQuality, SpreadingFactor, TxConfig};
use lorawan_device::async_device::Timings;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Idle,
    Txing,
    Rxing,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RadioError;

static IRQ_WAKER: AtomicWaker = AtomicWaker::new();

/// The radio peripheral keeping the radio state and owning the radio IRQ.
pub struct SubGhzRadio<'d, RS> {
    radio: SubGhz<'d, NoDma, NoDma>,
    switch: RS,
    irq: PeripheralRef<'d, SUBGHZ_RADIO>,
}

#[derive(Default)]
#[non_exhaustive]
pub struct SubGhzRadioConfig {
    pub reg_mode: RegMode,
    pub calibrate_image: CalibrateImage,
    pub pa_config: PaConfig,
    pub tx_params: TxParams,
}

impl<'d, RS: RadioSwitch> SubGhzRadio<'d, RS> {
    /// Create a new instance of a SubGhz radio for LoRaWAN.
    pub fn new(
        mut radio: SubGhz<'d, NoDma, NoDma>,
        switch: RS,
        irq: impl Peripheral<P = SUBGHZ_RADIO> + 'd,
        config: SubGhzRadioConfig,
    ) -> Result<Self, RadioError> {
        into_ref!(irq);

        radio.reset();

        irq.disable();
        irq.set_handler(|_| {
            IRQ_WAKER.wake();
            unsafe { SUBGHZ_RADIO::steal().disable() };
        });

        configure_radio(&mut radio, config)?;

        Ok(Self { radio, switch, irq })
    }

    /// Perform a transmission with the given parameters and payload. Returns any time adjustements needed form
    /// the upcoming RX window start.
    async fn do_tx(&mut self, config: TxConfig, buf: &[u8]) -> Result<u32, RadioError> {
        trace!("TX request: {:?}", config);
        self.switch.set_tx();

        self.radio
            .set_rf_frequency(&RfFreq::from_frequency(config.rf.frequency))?;

        self.set_lora_mod_params(config.rf)?;

        let packet_params = LoRaPacketParams::new()
            .set_preamble_len(8)
            .set_header_type(HeaderType::Variable)
            .set_payload_len(buf.len() as u8)
            .set_crc_en(true)
            .set_invert_iq(false);

        self.radio.set_lora_packet_params(&packet_params)?;

        let irq_cfg = CfgIrq::new().irq_enable_all(Irq::TxDone).irq_enable_all(Irq::Timeout);
        self.radio.set_irq_cfg(&irq_cfg)?;

        self.radio.set_buffer_base_address(0, 0)?;
        self.radio.write_buffer(0, buf)?;

        // The maximum airtime for any LoRaWAN package is 2793.5ms.
        // The value of 4000ms is copied from C driver and gives us a good safety margin.
        self.radio.set_tx(Timeout::from_millis_sat(4000))?;
        trace!("TX started");

        loop {
            let (_status, irq_status) = self.irq_wait().await;

            if irq_status & Irq::TxDone.mask() != 0 {
                trace!("TX done");
                return Ok(0);
            }

            if irq_status & Irq::Timeout.mask() != 0 {
                return Err(RadioError);
            }
        }
    }

    fn set_lora_mod_params(&mut self, config: RfConfig) -> Result<(), Error> {
        let mod_params = LoRaModParams::new()
            .set_sf(convert_spreading_factor(&config.spreading_factor))
            .set_bw(convert_bandwidth(&config.bandwidth))
            .set_cr(CodingRate::Cr45)
            .set_ldro_en(matches!(
                (config.spreading_factor, config.bandwidth),
                (SpreadingFactor::_12, Bandwidth::_125KHz)
                    | (SpreadingFactor::_12, Bandwidth::_250KHz)
                    | (SpreadingFactor::_11, Bandwidth::_125KHz)
            ));
        self.radio.set_lora_mod_params(&mod_params)
    }

    /// Perform a radio receive operation with the radio config and receive buffer. The receive buffer must
    /// be able to hold a single LoRaWAN packet.
    async fn do_rx(&mut self, config: RfConfig, buf: &mut [u8]) -> Result<(usize, RxQuality), RadioError> {
        assert!(buf.len() >= 255);
        trace!("RX request: {:?}", config);
        self.switch.set_rx();

        self.radio.set_rf_frequency(&RfFreq::from_frequency(config.frequency))?;

        self.set_lora_mod_params(config)?;

        let packet_params = LoRaPacketParams::new()
            .set_preamble_len(8)
            .set_header_type(HeaderType::Variable)
            .set_payload_len(0xFF)
            .set_crc_en(false)
            .set_invert_iq(true);
        self.radio.set_lora_packet_params(&packet_params)?;

        let irq_cfg = CfgIrq::new()
            .irq_enable_all(Irq::RxDone)
            .irq_enable_all(Irq::PreambleDetected)
            .irq_enable_all(Irq::HeaderValid)
            .irq_enable_all(Irq::HeaderErr)
            .irq_enable_all(Irq::Err)
            .irq_enable_all(Irq::Timeout);
        self.radio.set_irq_cfg(&irq_cfg)?;

        self.radio.set_buffer_base_address(0, 0)?;

        // NOTE: Upper layer handles timeout by cancelling the future
        self.radio.set_rx(Timeout::DISABLED)?;

        trace!("RX started");

        loop {
            let (_status, irq_status) = self.irq_wait().await;

            if irq_status & Irq::RxDone.mask() != 0 {
                let (_status, len, ptr) = self.radio.rx_buffer_status()?;
                let packet_status = self.radio.lora_packet_status()?;
                let rssi = packet_status.rssi_pkt().to_integer();
                let snr = packet_status.snr_pkt().to_integer();
                self.radio.read_buffer(ptr, &mut buf[..len as usize])?;
                self.radio.set_standby(StandbyClk::Rc)?;

                #[cfg(feature = "defmt")]
                trace!("RX done: {=[u8]:#02X}", &mut buf[..len as usize]);

                #[cfg(feature = "log")]
                trace!("RX done: {:02x?}", &mut buf[..len as usize]);
                return Ok((len as usize, RxQuality::new(rssi, snr as i8)));
            }

            if irq_status & Irq::Timeout.mask() != 0 {
                return Err(RadioError);
            }
        }
    }

    async fn irq_wait(&mut self) -> (Status, u16) {
        poll_fn(|cx| {
            self.irq.unpend();
            self.irq.enable();
            IRQ_WAKER.register(cx.waker());

            let (status, irq_status) = self.radio.irq_status().expect("error getting irq status");
            self.radio
                .clear_irq_status(irq_status)
                .expect("error clearing irq status");

            trace!("SUGHZ IRQ 0b{:016b}, {:?}", irq_status, status);

            if irq_status == 0 {
                Poll::Pending
            } else {
                Poll::Ready((status, irq_status))
            }
        })
        .await
    }
}

fn configure_radio(radio: &mut SubGhz<'_, NoDma, NoDma>, config: SubGhzRadioConfig) -> Result<(), RadioError> {
    trace!("Configuring STM32WL SUBGHZ radio");

    radio.set_regulator_mode(config.reg_mode)?;
    radio.set_standby(StandbyClk::Rc)?;

    let tcxo_mode = TcxoMode::new()
        .set_txco_trim(TcxoTrim::Volts1pt7)
        .set_timeout(Timeout::from_duration_sat(core::time::Duration::from_millis(100)));
    radio.set_tcxo_mode(&tcxo_mode)?;
    // Reduce input capacitance as shown in Reference Manual "Figure 23. HSE32 TCXO control".
    // The STM32CUBE C driver also does this.
    radio.set_hse_in_trim(HseTrim::MIN)?;

    // Re-calibrate everything after setting the TXCO config.
    radio.calibrate(0x7F)?;
    radio.calibrate_image(config.calibrate_image)?;

    radio.set_pa_config(&config.pa_config)?;
    radio.set_tx_params(&config.tx_params)?;
    radio.set_pa_ocp(Ocp::Max140m)?;

    radio.set_packet_type(PacketType::LoRa)?;
    radio.set_lora_sync_word(LoRaSyncWord::Public)?;

    trace!("Done initializing STM32WL SUBGHZ radio");
    Ok(())
}

impl<'d, RS: RadioSwitch> PhyRxTx for SubGhzRadio<'d, RS> {
    type PhyError = RadioError;

    type TxFuture<'m> = impl Future<Output = Result<u32, Self::PhyError>> + 'm where Self: 'm;
    fn tx<'m>(&'m mut self, config: TxConfig, buf: &'m [u8]) -> Self::TxFuture<'m> {
        async move { self.do_tx(config, buf).await }
    }

    type RxFuture<'m> = impl Future<Output = Result<(usize, RxQuality), Self::PhyError>> + 'm  where Self: 'm;
    fn rx<'m>(&'m mut self, config: RfConfig, buf: &'m mut [u8]) -> Self::RxFuture<'m> {
        async move { self.do_rx(config, buf).await }
    }
}

impl From<embassy_stm32::spi::Error> for RadioError {
    fn from(_: embassy_stm32::spi::Error) -> Self {
        RadioError
    }
}

impl<'d, RS> Timings for SubGhzRadio<'d, RS> {
    fn get_rx_window_offset_ms(&self) -> i32 {
        -3
    }
    fn get_rx_window_duration_ms(&self) -> u32 {
        1003
    }
}

pub trait RadioSwitch {
    fn set_rx(&mut self);
    fn set_tx(&mut self);
}

fn convert_spreading_factor(sf: &SpreadingFactor) -> SF {
    match sf {
        SpreadingFactor::_7 => SF::Sf7,
        SpreadingFactor::_8 => SF::Sf8,
        SpreadingFactor::_9 => SF::Sf9,
        SpreadingFactor::_10 => SF::Sf10,
        SpreadingFactor::_11 => SF::Sf11,
        SpreadingFactor::_12 => SF::Sf12,
    }
}

fn convert_bandwidth(bw: &Bandwidth) -> LoRaBandwidth {
    match bw {
        Bandwidth::_125KHz => LoRaBandwidth::Bw125,
        Bandwidth::_250KHz => LoRaBandwidth::Bw250,
        Bandwidth::_500KHz => LoRaBandwidth::Bw500,
    }
}
