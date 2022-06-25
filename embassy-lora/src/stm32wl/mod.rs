//! A radio driver integration for the radio found on STM32WL family devices.
use core::future::Future;
use core::task::Poll;

use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_stm32::dma::NoDma;
use embassy_stm32::interrupt::{Interrupt, InterruptExt, SUBGHZ_RADIO};
use embassy_stm32::subghz::{
    CalibrateImage, CfgIrq, CodingRate, Error, HeaderType, Irq, LoRaBandwidth, LoRaModParams, LoRaPacketParams,
    LoRaSyncWord, Ocp, PaConfig, PaSel, PacketType, RampTime, RegMode, RfFreq, SpreadingFactor as SF, StandbyClk,
    Status, SubGhz, TcxoMode, TcxoTrim, Timeout, TxParams,
};
use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;
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

        Self { radio, switch, irq }
    }

    /// Configure radio settings in preparation for TX or RX
    pub(crate) fn configure(&mut self) -> Result<(), RadioError> {
        trace!("Configuring STM32WL SUBGHZ radio");
        self.radio.set_standby(StandbyClk::Rc)?;
        let tcxo_mode = TcxoMode::new()
            .set_txco_trim(TcxoTrim::Volts1pt7)
            .set_timeout(Timeout::from_duration_sat(core::time::Duration::from_millis(40)));

        self.radio.set_tcxo_mode(&tcxo_mode)?;
        self.radio.set_regulator_mode(RegMode::Ldo)?;

        self.radio.calibrate_image(CalibrateImage::ISM_863_870)?;

        self.radio.set_buffer_base_address(0, 0)?;

        self.radio
            .set_pa_config(&PaConfig::new().set_pa_duty_cycle(0x1).set_hp_max(0x0).set_pa(PaSel::Lp))?;

        self.radio.set_pa_ocp(Ocp::Max140m)?;

        //        let tx_params = TxParams::LP_14.set_ramp_time(RampTime::Micros40);
        self.radio
            .set_tx_params(&TxParams::new().set_ramp_time(RampTime::Micros40).set_power(0x0A))?;

        self.radio.set_packet_type(PacketType::LoRa)?;
        self.radio.set_lora_sync_word(LoRaSyncWord::Public)?;
        trace!("Done initializing STM32WL SUBGHZ radio");
        Ok(())
    }

    /// Perform a transmission with the given parameters and payload. Returns any time adjustements needed form
    /// the upcoming RX window start.
    async fn do_tx(&mut self, config: TxConfig, buf: &[u8]) -> Result<u32, RadioError> {
        //trace!("TX Request: {}", config);
        trace!("TX START");
        self.switch.set_tx();
        self.configure()?;

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

        let irq_cfg = CfgIrq::new()
            .irq_enable_all(Irq::TxDone)
            .irq_enable_all(Irq::RxDone)
            .irq_enable_all(Irq::Timeout);
        self.radio.set_irq_cfg(&irq_cfg)?;

        self.radio.set_buffer_base_address(0, 0)?;
        self.radio.write_buffer(0, buf)?;

        self.radio.set_tx(Timeout::DISABLED)?;

        loop {
            let (_status, irq_status) = self.irq_wait().await;

            if irq_status & Irq::TxDone.mask() != 0 {
                let stats = self.radio.lora_stats()?;
                let (status, error_mask) = self.radio.op_error()?;
                trace!(
                    "TX done. Stats: {:?}. OP error: {:?}, mask {:?}",
                    stats,
                    status,
                    error_mask
                );

                return Ok(0);
            } else if irq_status & Irq::Timeout.mask() != 0 {
                trace!("TX timeout");
                return Err(RadioError);
            }
        }
    }

    fn set_lora_mod_params(&mut self, config: RfConfig) -> Result<(), Error> {
        let mod_params = LoRaModParams::new()
            .set_sf(convert_spreading_factor(config.spreading_factor))
            .set_bw(convert_bandwidth(config.bandwidth))
            .set_cr(CodingRate::Cr45)
            .set_ldro_en(true);
        self.radio.set_lora_mod_params(&mod_params)
    }

    /// Perform a radio receive operation with the radio config and receive buffer. The receive buffer must
    /// be able to hold a single LoRaWAN packet.
    async fn do_rx(&mut self, config: RfConfig, buf: &mut [u8]) -> Result<(usize, RxQuality), RadioError> {
        assert!(buf.len() >= 255);
        trace!("RX START");
        // trace!("Starting RX: {}", config);
        self.switch.set_rx();
        self.configure()?;

        self.radio.set_rf_frequency(&RfFreq::from_frequency(config.frequency))?;

        self.set_lora_mod_params(config)?;

        let packet_params = LoRaPacketParams::new()
            .set_preamble_len(8)
            .set_header_type(HeaderType::Variable)
            .set_payload_len(0xFF)
            .set_crc_en(true)
            .set_invert_iq(true);
        self.radio.set_lora_packet_params(&packet_params)?;

        let irq_cfg = CfgIrq::new()
            .irq_enable_all(Irq::RxDone)
            .irq_enable_all(Irq::PreambleDetected)
            .irq_enable_all(Irq::HeaderErr)
            .irq_enable_all(Irq::Timeout)
            .irq_enable_all(Irq::Err);
        self.radio.set_irq_cfg(&irq_cfg)?;

        self.radio.set_rx(Timeout::DISABLED)?;
        trace!("RX started");

        loop {
            let (status, irq_status) = self.irq_wait().await;

            trace!("RX IRQ {:?}, {:?}", status, irq_status);
            if irq_status & Irq::RxDone.mask() != 0 {
                let (status, len, ptr) = self.radio.rx_buffer_status()?;

                let packet_status = self.radio.lora_packet_status()?;
                let rssi = packet_status.rssi_pkt().to_integer();
                let snr = packet_status.snr_pkt().to_integer();
                trace!(
                    "RX done. Received {} bytes. RX status: {:?}. Pkt status: {:?}",
                    len,
                    status.cmd(),
                    packet_status,
                );
                self.radio.read_buffer(ptr, &mut buf[..len as usize])?;
                self.radio.set_standby(StandbyClk::Rc)?;
                return Ok((len as usize, RxQuality::new(rssi, snr as i8)));
            } else if irq_status & (Irq::Timeout.mask() | Irq::TxDone.mask()) != 0 {
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
            trace!("IRQ status: {=u16:b}", irq_status);
            if irq_status == 0 {
                Poll::Pending
            } else {
                Poll::Ready((status, irq_status))
            }
        })
        .await
    }
}

impl<RS: RadioSwitch> PhyRxTx for SubGhzRadio<'static, RS> {
    type PhyError = RadioError;

    type TxFuture<'m> = impl Future<Output = Result<u32, Self::PhyError>> + 'm where RS: 'm;
    fn tx<'m>(&'m mut self, config: TxConfig, buf: &'m [u8]) -> Self::TxFuture<'m> {
        async move { self.do_tx(config, buf).await }
    }

    type RxFuture<'m> = impl Future<Output = Result<(usize, RxQuality), Self::PhyError>> + 'm  where RS: 'm;
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
        -200
    }
    fn get_rx_window_duration_ms(&self) -> u32 {
        800
    }
}

pub trait RadioSwitch {
    fn set_rx(&mut self);
    fn set_tx(&mut self);
}

fn convert_spreading_factor(sf: SpreadingFactor) -> SF {
    match sf {
        SpreadingFactor::_7 => SF::Sf7,
        SpreadingFactor::_8 => SF::Sf8,
        SpreadingFactor::_9 => SF::Sf9,
        SpreadingFactor::_10 => SF::Sf10,
        SpreadingFactor::_11 => SF::Sf11,
        SpreadingFactor::_12 => SF::Sf12,
    }
}

fn convert_bandwidth(bw: Bandwidth) -> LoRaBandwidth {
    match bw {
        Bandwidth::_125KHz => LoRaBandwidth::Bw125,
        Bandwidth::_250KHz => LoRaBandwidth::Bw250,
        Bandwidth::_500KHz => LoRaBandwidth::Bw500,
    }
}
