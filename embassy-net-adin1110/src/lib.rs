#![cfg_attr(not(test), no_std)]
#![deny(clippy::pedantic)]
#![allow(async_fn_in_trait)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// must go first!
mod fmt;

mod crc32;
mod crc8;
mod mdio;
mod phy;
mod protocol;
mod regs;

use ch::driver::LinkState;
pub use crc32::ETH_FCS;
use embassy_futures::select::{Either, select};
use embassy_net_driver_channel as ch;
use embassy_time::Timer;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{Error, SpiDevice};
pub use mdio::MdioBus;
pub use phy::Phy10BaseT1x;
use phy::{RegsC22, RegsC45};
pub use protocol::Adin1110Protocol;
#[cfg(feature = "generic-spi")]
pub use protocol::GenericSpi;
#[cfg(feature = "tc6")]
pub use protocol::Tc6;
use regs::{Config0, Config2, SpiRegisters as sr, Status0, Status1};

use crate::regs::{LedCntrl, LedFunc, LedPol, LedPolarity};

/// ADIN1110 intern PHY ID
pub const PHYID: u32 = 0x0283_BC91;

/// Error values ADIN1110
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(non_camel_case_types)]
pub enum AdinError<E> {
    /// SPI-BUS Error
    Spi(E),
    /// Ethernet FCS error
    FCS,
    /// SPI Header CRC error
    SPI_CRC,
    /// Received or sended ethernet packet is too big
    PACKET_TOO_BIG,
    /// Received or sended ethernet packet is too small
    PACKET_TOO_SMALL,
    /// MDIO transaction timeout
    MDIO_ACC_TIMEOUT,
}

/// Type alias `Result` type with `AdinError` as error type.
pub type AEResult<T, SPIError> = core::result::Result<T, AdinError<SPIError>>;

/// Internet PHY address
pub const MDIO_PHY_ADDR: u8 = 0x01;

/// Maximum Transmission Unit
pub const MTU: usize = 1514;

/// Max SPI/Frame buffer size
pub const MAX_BUFF: usize = 2048;

const DONT_CARE_BYTE: u8 = 0x00;
const TURN_AROUND_BYTE: u8 = 0x00;

/// Packet minimal frame/packet length
const ETH_MIN_LEN: usize = 64;
/// Ethernet `Frame Check Sequence` length
const FCS_LEN: usize = 4;
/// Packet minimal frame/packet length without `Frame Check Sequence` length
const ETH_MIN_WITHOUT_FCS_LEN: usize = ETH_MIN_LEN - FCS_LEN;

/// SPI Header, contains SPI action and register id.
const SPI_HEADER_LEN: usize = 2;
/// SPI Header CRC length
const SPI_HEADER_CRC_LEN: usize = 1;
/// SPI Header Turn Around length
const SPI_HEADER_TA_LEN: usize = 1;
/// Frame Header length
const FRAME_HEADER_LEN: usize = 2;
/// Space for last bytes to create multipule 4 bytes on the end of a FIFO read/write.
const SPI_SPACE_MULTIPULE: usize = 3;

/// P1 = 0x00, P2 = 0x01
const PORT_ID_BYTE: u8 = 0x00;

/// Type alias for the embassy-net driver for ADIN1110
pub type Device<'d> = embassy_net_driver_channel::Device<'d, MTU>;

/// Internal state for the embassy-net integration.
pub struct State<const N_RX: usize, const N_TX: usize> {
    ch_state: ch::State<MTU, N_RX, N_TX>,
}
impl<const N_RX: usize, const N_TX: usize> State<N_RX, N_TX> {
    /// Create a new `State`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ch_state: ch::State::new(),
        }
    }
}

/// ADIN1110 embassy-net driver
#[derive(Debug)]
pub struct ADIN1110<P: Adin1110Protocol> {
    /// Generic or OPEN Alliance TC6 SPI protocol, wraps SPI device
    protocol: P,
}

impl<P: Adin1110Protocol> ADIN1110<P> {
    /// Create a new ADIN1110 instance.
    pub fn new(protocol: P) -> Self {
        Self { protocol }
    }

    /// Read a SPI register
    pub async fn read_reg(&mut self, reg: sr) -> AEResult<u32, P::SpiError> {
        self.protocol.read_reg(reg.into()).await
    }

    /// Write a SPI register
    pub async fn write_reg(&mut self, reg: sr, value: u32) -> AEResult<(), P::SpiError> {
        self.protocol.write_reg(reg.into(), value).await
    }

    /// helper function for write to `MDIO_ACC` register and wait for ready!
    async fn write_mdio_acc_reg(&mut self, mdio_acc_val: u32) -> AEResult<u32, P::SpiError> {
        self.write_reg(sr::MDIO_ACC, mdio_acc_val).await?;

        // TODO: Add proper timeout!
        for _ in 0..100_000 {
            let val = self.read_reg(sr::MDIO_ACC).await?;
            if val & 0x8000_0000 != 0 {
                return Ok(val);
            }
        }

        Err(AdinError::MDIO_ACC_TIMEOUT)
    }

    /// Read out fifo ethernet packet memory received via the wire.
    pub async fn read_fifo(&mut self, frame: &mut [u8]) -> AEResult<usize, P::SpiError> {
        self.protocol.read_fifo(frame).await
    }

    /// Write to fifo ethernet packet memory send over the wire.
    pub async fn write_fifo(&mut self, frame: &[u8]) -> AEResult<(), P::SpiError> {
        self.protocol.write_fifo(frame).await
    }

    /// Programs the mac address in the mac filters.
    /// Also set the boardcast address.
    /// The chip supports 2 priority queues but current code doesn't support this mode.
    pub async fn set_mac_addr(&mut self, mac: &[u8; 6]) -> AEResult<(), P::SpiError> {
        let mac_high_part = u16::from_be_bytes(mac[0..2].try_into().unwrap());
        let mac_low_part = u32::from_be_bytes(mac[2..6].try_into().unwrap());

        // program our mac address in the mac address filter
        self.write_reg(sr::ADDR_FILT_UPR0, (1 << 16) | (1 << 30) | u32::from(mac_high_part))
            .await?;
        self.write_reg(sr::ADDR_FILT_LWR0, mac_low_part).await?;

        self.write_reg(sr::ADDR_MSK_UPR0, u32::from(mac_high_part)).await?;
        self.write_reg(sr::ADDR_MSK_LWR0, mac_low_part).await?;

        // Also program broadcast address in the mac address filter
        self.write_reg(sr::ADDR_FILT_UPR1, (1 << 16) | (1 << 30) | 0xFFFF)
            .await?;
        self.write_reg(sr::ADDR_FILT_LWR1, 0xFFFF_FFFF).await?;
        self.write_reg(sr::ADDR_MSK_UPR1, 0xFFFF).await?;
        self.write_reg(sr::ADDR_MSK_LWR1, 0xFFFF_FFFF).await?;

        Ok(())
    }
}

impl<SPI: SpiDevice> mdio::MdioBus for ADIN1110<GenericSpi<SPI>> {
    type Error = AdinError<SPI::Error>;

    /// Read from the PHY Registers as Clause 22.
    async fn read_cl22(&mut self, phy_id: u8, reg: u8) -> Result<u16, Self::Error> {
        let mdio_acc_val: u32 =
            (0x1 << 28) | u32::from(phy_id & 0x1F) << 21 | u32::from(reg & 0x1F) << 16 | (0x3 << 26);

        // Result is in the lower half of the answer.
        #[allow(clippy::cast_possible_truncation)]
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|val| val as u16)
    }

    /// Read from the PHY Registers as Clause 45.
    async fn read_cl45(&mut self, phy_id: u8, regc45: (u8, u16)) -> Result<u16, Self::Error> {
        let mdio_acc_val = u32::from(phy_id & 0x1F) << 21 | u32::from(regc45.0 & 0x1F) << 16 | u32::from(regc45.1);

        self.write_mdio_acc_reg(mdio_acc_val).await?;

        let mdio_acc_val = u32::from(phy_id & 0x1F) << 21 | u32::from(regc45.0 & 0x1F) << 16 | (0x03 << 26);

        // Result is in the lower half of the answer.
        #[allow(clippy::cast_possible_truncation)]
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|val| val as u16)
    }

    /// Write to the PHY Registers as Clause 22.
    async fn write_cl22(&mut self, phy_id: u8, reg: u8, val: u16) -> Result<(), Self::Error> {
        let mdio_acc_val: u32 =
            (0x1 << 28) | u32::from(phy_id & 0x1F) << 21 | u32::from(reg & 0x1F) << 16 | (0x1 << 26) | u32::from(val);

        self.write_mdio_acc_reg(mdio_acc_val).await.map(|_| ())
    }

    /// Write to the PHY Registers as Clause 45.
    async fn write_cl45(&mut self, phy_id: u8, regc45: (u8, u16), value: u16) -> AEResult<(), SPI::Error> {
        let phy_id = u32::from(phy_id & 0x1F) << 21;
        let dev_addr = u32::from(regc45.0 & 0x1F) << 16;
        let reg = u32::from(regc45.1);

        let mdio_acc_val: u32 = phy_id | dev_addr | reg;
        self.write_mdio_acc_reg(mdio_acc_val).await?;

        let mdio_acc_val: u32 = phy_id | dev_addr | (0x01 << 26) | u32::from(value);
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|_| ())
    }
}

#[cfg(feature = "tc6")]
impl<SPI: SpiDevice> mdio::MdioBus for ADIN1110<Tc6<SPI>> {
    type Error = AdinError<SPI::Error>;

    /// Read from the PHY Registers as Clause 22.
    async fn read_cl22(&mut self, phy_id: u8, reg: u8) -> Result<u16, Self::Error> {
        let mdio_acc_val: u32 =
            (0x1 << 28) | u32::from(phy_id & 0x1F) << 21 | u32::from(reg & 0x1F) << 16 | (0x3 << 26);

        // Result is in the lower half of the answer.
        #[allow(clippy::cast_possible_truncation)]
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|val| val as u16)
    }

    /// Read from the PHY Registers as Clause 45.
    async fn read_cl45(&mut self, phy_id: u8, regc45: (u8, u16)) -> Result<u16, Self::Error> {
        let mdio_acc_val = u32::from(phy_id & 0x1F) << 21 | u32::from(regc45.0 & 0x1F) << 16 | u32::from(regc45.1);

        self.write_mdio_acc_reg(mdio_acc_val).await?;

        let mdio_acc_val = u32::from(phy_id & 0x1F) << 21 | u32::from(regc45.0 & 0x1F) << 16 | (0x03 << 26);

        // Result is in the lower half of the answer.
        #[allow(clippy::cast_possible_truncation)]
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|val| val as u16)
    }

    /// Write to the PHY Registers as Clause 22.
    async fn write_cl22(&mut self, phy_id: u8, reg: u8, val: u16) -> Result<(), Self::Error> {
        let mdio_acc_val: u32 =
            (0x1 << 28) | u32::from(phy_id & 0x1F) << 21 | u32::from(reg & 0x1F) << 16 | (0x1 << 26) | u32::from(val);

        self.write_mdio_acc_reg(mdio_acc_val).await.map(|_| ())
    }

    /// Write to the PHY Registers as Clause 45.
    async fn write_cl45(&mut self, phy_id: u8, regc45: (u8, u16), value: u16) -> AEResult<(), SPI::Error> {
        let phy_id = u32::from(phy_id & 0x1F) << 21;
        let dev_addr = u32::from(regc45.0 & 0x1F) << 16;
        let reg = u32::from(regc45.1);

        let mdio_acc_val: u32 = phy_id | dev_addr | reg;
        self.write_mdio_acc_reg(mdio_acc_val).await?;

        let mdio_acc_val: u32 = phy_id | dev_addr | (0x01 << 26) | u32::from(value);
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|_| ())
    }
}

/// Background runner for the ADIN1110.
///
/// You must call `.run()` in a background task for the ADIN1110 to operate.
pub struct Runner<'d, P: Adin1110Protocol, INT, RST> {
    mac: ADIN1110<P>,
    ch: ch::Runner<'d, MTU>,
    int: INT,
    is_link_up: bool,
    _reset: RST,
}

impl<'d, P: Adin1110Protocol, INT: Wait, RST: OutputPin> Runner<'d, P, INT, RST>
where
    ADIN1110<P>: MdioBus,
    <ADIN1110<P> as MdioBus>::Error: core::fmt::Debug,
{
    /// Run the driver.
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self) -> ! {
        loop {
            let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();

            loop {
                debug!("Waiting for interrupts");
                match select(self.int.wait_for_low(), tx_chan.tx_buf()).await {
                    Either::First(_) => {
                        let mut status1_clr = Status1(0);
                        let mut status1 = Status1(self.mac.read_reg(sr::STATUS1).await.unwrap());

                        while status1.p1_rx_rdy() {
                            debug!("alloc RX packet buffer");
                            match select(rx_chan.rx_buf(), tx_chan.tx_buf()).await {
                                // Handle frames that needs to transmit from the wire.
                                // Note: rx_chan.rx_buf() channel don´t accept new request
                                //       when the tx_chan is full. So these will be handled
                                //       automaticly.
                                Either::First(frame) => match self.mac.read_fifo(frame).await {
                                    Ok(n) => {
                                        rx_chan.rx_done(n);
                                    }
                                    Err(e) => match e {
                                        AdinError::PACKET_TOO_BIG => {
                                            error!("RX Packet too big, DROP");
                                            self.mac.write_reg(sr::FIFO_CLR, 1).await.unwrap();
                                        }
                                        AdinError::PACKET_TOO_SMALL => {
                                            error!("RX Packet too small, DROP");
                                            self.mac.write_reg(sr::FIFO_CLR, 1).await.unwrap();
                                        }
                                        AdinError::Spi(e) => {
                                            error!("RX Spi error {}", e.kind());
                                        }
                                        e => {
                                            error!("RX Error {:?}", e);
                                        }
                                    },
                                },
                                Either::Second(frame) => {
                                    // Handle frames that needs to transmit to the wire.
                                    self.mac.write_fifo(frame).await.unwrap();
                                    tx_chan.tx_done();
                                }
                            }
                            status1 = Status1(self.mac.read_reg(sr::STATUS1).await.unwrap());
                        }

                        let status0 = Status0(self.mac.read_reg(sr::STATUS0).await.unwrap());
                        if status1.0 & !0x1b != 0 {
                            error!("SPE CHIP STATUS 0:{:08x} 1:{:08x}", status0.0, status1.0);
                        }

                        if status1.tx_rdy() {
                            status1_clr.set_tx_rdy(true);
                            trace!("TX_DONE");
                        }

                        if status1.link_change() {
                            let link = status1.p1_link_status();
                            self.is_link_up = link;

                            if link {
                                let link_status = self
                                    .mac
                                    .read_cl45(MDIO_PHY_ADDR, RegsC45::DA7::AN_STATUS_EXTRA.into())
                                    .await
                                    .unwrap();

                                let volt = if link_status & (0b11 << 5) == (0b11 << 5) {
                                    "2.4"
                                } else {
                                    "1.0"
                                };

                                let mse = self
                                    .mac
                                    .read_cl45(MDIO_PHY_ADDR, RegsC45::DA1::MSE_VAL.into())
                                    .await
                                    .unwrap();

                                info!("LINK Changed: Link Up, Volt: {} V p-p, MSE: {:0004}", volt, mse);
                            } else {
                                info!("LINK Changed: Link Down");
                            }

                            state_chan.set_link_state(if link { LinkState::Up } else { LinkState::Down });
                            status1_clr.set_link_change(true);
                        }

                        if status1.tx_ecc_err() {
                            error!("SPI TX_ECC_ERR error, CLEAR TX FIFO");
                            self.mac.write_reg(sr::FIFO_CLR, 2).await.unwrap();
                            status1_clr.set_tx_ecc_err(true);
                        }

                        if status1.rx_ecc_err() {
                            error!("SPI RX_ECC_ERR error");
                            status1_clr.set_rx_ecc_err(true);
                        }

                        if status1.spi_err() {
                            error!("SPI SPI_ERR CRC error");
                            status1_clr.set_spi_err(true);
                        }

                        if status0.phyint() {
                            let crsm_irq_st = self
                                .mac
                                .read_cl45(MDIO_PHY_ADDR, RegsC45::DA1E::CRSM_IRQ_STATUS.into())
                                .await
                                .unwrap();

                            let phy_irq_st = self
                                .mac
                                .read_cl45(MDIO_PHY_ADDR, RegsC45::DA1F::PHY_SYBSYS_IRQ_STATUS.into())
                                .await
                                .unwrap();

                            warn!(
                                "SPE CHIP PHY CRSM_IRQ_STATUS {:04x} PHY_SUBSYS_IRQ_STATUS {:04x}",
                                crsm_irq_st, phy_irq_st
                            );
                        }

                        if status0.txfcse() {
                            error!("Ethernet Frame FCS and calc FCS don't match!");
                        }

                        // Clear status0
                        self.mac.write_reg(sr::STATUS0, 0xFFF).await.unwrap();
                        self.mac.write_reg(sr::STATUS1, status1_clr.0).await.unwrap();
                    }
                    Either::Second(packet) => {
                        // Handle frames that needs to transmit to the wire.
                        self.mac.write_fifo(packet).await.unwrap();
                        tx_chan.tx_done();
                    }
                }
            }
        }
    }
}

#[cfg(feature = "generic-spi")]
impl<SPI: SpiDevice> ADIN1110<GenericSpi<SPI>> {
    /// Create driver with Generic SPI protocol (existing behavior)
    pub fn new_generic(spi: SPI, crc_enabled: bool, append_fcs_on_tx: bool) -> Self {
        let protocol = GenericSpi::new(spi, crc_enabled, append_fcs_on_tx);
        Self::new(protocol)
    }
}

#[cfg(feature = "tc6")]
impl<SPI: SpiDevice> ADIN1110<Tc6<SPI>> {
    /// Create driver with OPEN Alliance TC6 SPI protocol
    pub fn new_tc6(spi: SPI) -> Self {
        let protocol = Tc6::new(spi);
        Self::new(protocol)
    }
}
/// Obtain a driver for using the ADIN1110 with [`embassy-net`](crates.io/crates/embassy-net).
#[cfg(feature = "generic-spi")]
pub async fn new<const N_RX: usize, const N_TX: usize, SPI: SpiDevice, INT: Wait, RST: OutputPin>(
    mac_addr: [u8; 6],
    state: &'_ mut State<N_RX, N_TX>,
    spi_dev: SPI,
    int: INT,
    mut reset: RST,
    spi_crc: bool,
    append_fcs_on_tx: bool,
) -> (Device<'_>, Runner<'_, GenericSpi<SPI>, INT, RST>) {
    use crate::regs::{IMask0, IMask1};

    info!("INIT ADIN1110");

    // Reset sequence
    reset.set_low().unwrap();

    // Wait t1: 20-43mS
    Timer::after_millis(30).await;

    reset.set_high().unwrap();

    // Wait t3: 50mS
    Timer::after_millis(50).await;

    // Create device
    let mut mac = ADIN1110::new_generic(spi_dev, spi_crc, append_fcs_on_tx);

    // Check PHYID
    let id = mac.read_reg(sr::PHYID).await.unwrap();
    assert_eq!(id, PHYID);

    debug!("SPE: CHIP MAC/ID: {:08x}", id);

    #[cfg(any(feature = "defmt", feature = "log"))]
    {
        let adin_phy = Phy10BaseT1x::default();
        let phy_id = adin_phy.get_id(&mut mac).await.unwrap();
        debug!("SPE: CHIP: PHY ID: {:08x}", phy_id);
    }

    let mi_control = mac.read_cl22(MDIO_PHY_ADDR, RegsC22::CONTROL as u8).await.unwrap();
    debug!("SPE CHIP PHY MI_CONTROL {:04x}", mi_control);
    if mi_control & 0x0800 != 0 {
        let val = mi_control & !0x0800;
        debug!("SPE CHIP PHY MI_CONTROL Disable PowerDown");
        mac.write_cl22(MDIO_PHY_ADDR, RegsC22::CONTROL as u8, val)
            .await
            .unwrap();
    }

    // Config0
    let mut config0 = Config0(0x0000_0006);
    config0.set_txfcsve(append_fcs_on_tx);
    mac.write_reg(sr::CONFIG0, config0.0).await.unwrap();

    // Config2
    let mut config2 = Config2(0x0000_0800);
    // crc_append must be disable if tx_fcs_validation_enable is true!
    config2.set_crc_append(!append_fcs_on_tx);
    mac.write_reg(sr::CONFIG2, config2.0).await.unwrap();

    // Pin Mux Config 1
    let led_val = (0b11 << 6) | (0b11 << 4); // | (0b00 << 1);
    mac.write_cl45(MDIO_PHY_ADDR, RegsC45::DA1E::DIGIO_PINMUX.into(), led_val)
        .await
        .unwrap();

    let mut led_pol = LedPolarity(0);
    led_pol.set_led1_polarity(LedPol::ActiveLow);
    led_pol.set_led0_polarity(LedPol::ActiveLow);

    // Led Polarity Regisgere Active Low
    mac.write_cl45(MDIO_PHY_ADDR, RegsC45::DA1E::LED_POLARITY.into(), led_pol.0)
        .await
        .unwrap();

    // Led Both On
    let mut led_cntr = LedCntrl(0x0);

    // LED1: Yellow
    led_cntr.set_led1_en(true);
    led_cntr.set_led1_function(LedFunc::TxLevel2P4);
    // LED0: Green
    led_cntr.set_led0_en(true);
    led_cntr.set_led0_function(LedFunc::LinkupTxRxActicity);

    mac.write_cl45(MDIO_PHY_ADDR, RegsC45::DA1E::LED_CNTRL.into(), led_cntr.0)
        .await
        .unwrap();

    // Set ADIN1110 Interrupts, RX_READY and LINK_CHANGE
    // Enable interrupts LINK_CHANGE, TX_RDY, RX_RDY(P1), SPI_ERR
    // Have to clear the mask the enable it.
    let mut imask0_val = IMask0(0x0000_1FBF);
    imask0_val.set_txfcsem(false);
    imask0_val.set_phyintm(false);
    imask0_val.set_txboem(false);
    imask0_val.set_rxboem(false);
    imask0_val.set_txpem(false);

    mac.write_reg(sr::IMASK0, imask0_val.0).await.unwrap();

    // Set ADIN1110 Interrupts, RX_READY and LINK_CHANGE
    // Enable interrupts LINK_CHANGE, TX_RDY, RX_RDY(P1), SPI_ERR
    // Have to clear the mask the enable it.
    let mut imask1_val = IMask1(0x43FA_1F1A);
    imask1_val.set_link_change_mask(false);
    imask1_val.set_p1_rx_rdy_mask(false);
    imask1_val.set_spi_err_mask(false);
    imask1_val.set_tx_ecc_err_mask(false);
    imask1_val.set_rx_ecc_err_mask(false);

    mac.write_reg(sr::IMASK1, imask1_val.0).await.unwrap();

    // Program mac address but also sets mac filters.
    mac.set_mac_addr(&mac_addr).await.unwrap();

    let (runner, device) = ch::new(&mut state.ch_state, ch::driver::HardwareAddress::Ethernet(mac_addr));
    (
        device,
        Runner {
            ch: runner,
            mac,
            int,
            is_link_up: false,
            _reset: reset,
        },
    )
}

#[allow(clippy::similar_names)]
#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use embedded_hal_1::digital::{ErrorType, OutputPin};
    use embedded_hal_async::delay::DelayNs;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use embedded_hal_mock::common::Generic;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    use crate::crc8::crc8;

    #[derive(Debug, Default)]
    struct CsPinMock {
        pub high: u32,
        pub low: u32,
    }
    impl OutputPin for CsPinMock {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.low += 1;
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.high += 1;
            Ok(())
        }
    }
    impl ErrorType for CsPinMock {
        type Error = Infallible;
    }

    use super::*;

    // TODO: This is currently a workaround unit `ExclusiveDevice` is moved to `embedded-hal-bus`
    // see https://github.com/rust-embedded/embedded-hal/pull/462#issuecomment-1560014426
    struct MockDelay {}

    impl DelayNs for MockDelay {
        async fn delay_ns(&mut self, _ns: u32) {
            todo!()
        }

        async fn delay_us(&mut self, _us: u32) {
            todo!()
        }

        async fn delay_ms(&mut self, _ms: u32) {
            todo!()
        }
    }

    struct TestHarnass {
        spe: ADIN1110<
            GenericSpi<ExclusiveDevice<embedded_hal_mock::common::Generic<SpiTransaction<u8>>, CsPinMock, MockDelay>>,
        >,
        spi: Generic<SpiTransaction<u8>>,
    }

    impl TestHarnass {
        pub fn new(expectations: &[SpiTransaction<u8>], spi_crc: bool, append_fcs_on_tx: bool) -> Self {
            let cs = CsPinMock::default();
            let delay = MockDelay {};
            let spi = SpiMock::new(expectations);
            let spi_dev: ExclusiveDevice<embedded_hal_mock::common::Generic<SpiTransaction<u8>>, CsPinMock, MockDelay> =
                ExclusiveDevice::new(spi.clone(), cs, delay);
            let spe = ADIN1110::new_generic(spi_dev, spi_crc, append_fcs_on_tx);

            Self { spe, spi }
        }

        pub fn done(&mut self) {
            self.spi.done();
        }
    }

    #[futures_test::test]
    async fn mac_read_registers_without_crc() {
        // Configure expectations
        let expectations = [
            // 1st
            SpiTransaction::write_vec(vec![0x80, 0x01, TURN_AROUND_BYTE]),
            SpiTransaction::read_vec(vec![0x02, 0x83, 0xBC, 0x91]),
            SpiTransaction::flush(),
            // 2nd
            SpiTransaction::write_vec(vec![0x80, 0x02, TURN_AROUND_BYTE]),
            SpiTransaction::read_vec(vec![0x00, 0x00, 0x06, 0xC3]),
            SpiTransaction::flush(),
        ];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, false, true);

        // Read PHIID
        let val = th.spe.read_reg(sr::PHYID).await.expect("Error");
        assert_eq!(val, 0x0283_BC91);

        // Read CAPAVILITY
        let val = th.spe.read_reg(sr::CAPABILITY).await.expect("Error");
        assert_eq!(val, 0x0000_06C3);

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn mac_read_registers_with_crc() {
        // Configure expectations
        let expectations = [
            // 1st
            SpiTransaction::write_vec(vec![0x80, 0x01, 177, TURN_AROUND_BYTE]),
            SpiTransaction::read_vec(vec![0x02, 0x83, 0xBC, 0x91, 215]),
            SpiTransaction::flush(),
            // 2nd
            SpiTransaction::write_vec(vec![0x80, 0x02, 184, TURN_AROUND_BYTE]),
            SpiTransaction::read_vec(vec![0x00, 0x00, 0x06, 0xC3, 57]),
            SpiTransaction::flush(),
        ];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        assert_eq!(crc8(0x0283_BC91_u32.to_be_bytes().as_slice()), 215);
        assert_eq!(crc8(0x0000_06C3_u32.to_be_bytes().as_slice()), 57);

        // Read PHIID
        let val = th.spe.read_reg(sr::PHYID).await.expect("Error");
        assert_eq!(val, 0x0283_BC91);

        // Read CAPAVILITY
        let val = th.spe.read_reg(sr::CAPABILITY).await.expect("Error");
        assert_eq!(val, 0x0000_06C3);

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn mac_write_registers_without_crc() {
        // Configure expectations
        let expectations = [
            SpiTransaction::write_vec(vec![0xA0, 0x09, 0x12, 0x34, 0x56, 0x78]),
            SpiTransaction::flush(),
        ];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, false, true);

        // Write reg: 0x1FFF
        assert!(th.spe.write_reg(sr::STATUS1, 0x1234_5678).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn mac_write_registers_with_crc() {
        // Configure expectations
        let expectations = [
            SpiTransaction::write_vec(vec![0xA0, 0x09, 39, 0x12, 0x34, 0x56, 0x78, 28]),
            SpiTransaction::flush(),
        ];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        // Write reg: 0x1FFF
        assert!(th.spe.write_reg(sr::STATUS1, 0x1234_5678).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_minimal_with_crc() {
        // Configure expectations
        let mut expectations = vec![];

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 136, 0, 0, 0, 66, 201]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // SPI Header + optional CRC + Frame Header
        expectations.push(SpiTransaction::write_vec(vec![160, 49, 143, 0, 0]));
        // Packet data
        let packet = [0xFF_u8; 60];
        expectations.push(SpiTransaction::write_vec(packet.to_vec()));

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);
        // Padding
        if let Some(padding_len) = (ETH_MIN_LEN - FCS_LEN).checked_sub(packet.len()) {
            tail.resize(padding_len, 0x00);
        }
        // Packet FCS + optinal padding
        tail.extend_from_slice(&[77, 241, 140, 244, DONT_CARE_BYTE, DONT_CARE_BYTE]);

        expectations.push(SpiTransaction::write_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        assert!(th.spe.write_fifo(&packet).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_minimal_with_crc_without_fcs() {
        // Configure expectations
        let mut expectations = vec![];

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 136, 0, 0, 0, 62, 186]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // SPI Header + optional CRC + Frame Header
        expectations.push(SpiTransaction::write_vec(vec![160, 49, 143, 0, 0]));
        // Packet data
        let packet = [0xFF_u8; 60];
        expectations.push(SpiTransaction::write_vec(packet.to_vec()));

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);
        // Padding
        if let Some(padding_len) = (ETH_MIN_LEN - FCS_LEN).checked_sub(packet.len()) {
            tail.resize(padding_len, 0x00);
        }
        // Packet FCS + optinal padding
        tail.extend_from_slice(&[DONT_CARE_BYTE, DONT_CARE_BYTE]);

        expectations.push(SpiTransaction::write_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, false);

        assert!(th.spe.write_fifo(&packet).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_max_mtu_with_crc() {
        assert_eq!(MTU, 1514);
        // Configure expectations
        let mut expectations = vec![];

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 136, 0, 0, 5, 240, 159]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // SPI Header + optional CRC + Frame Header
        expectations.push(SpiTransaction::write_vec(vec![160, 49, 143, 0, 0]));
        // Packet data
        let packet = [0xAA_u8; MTU];
        expectations.push(SpiTransaction::write_vec(packet.to_vec()));

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);
        // Padding
        if let Some(padding_len) = (ETH_MIN_LEN - FCS_LEN).checked_sub(packet.len()) {
            tail.resize(padding_len, 0x00);
        }
        // Packet FCS + optinal padding
        tail.extend_from_slice(&[49, 196, 205, 160]);

        expectations.push(SpiTransaction::write_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        assert!(th.spe.write_fifo(&packet).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_invalid_lengths() {
        assert_eq!(MTU, 1514);

        // Configure expectations
        let expectations = vec![];

        // Max packet size = MAX_BUFF - FRAME_HEADER_LEN
        let packet = [0xAA_u8; MAX_BUFF - FRAME_HEADER_LEN + 1];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        // minimal
        assert!(matches!(
            th.spe.write_fifo(&packet[0..(6 + 6 + 2 - 1)]).await,
            Err(AdinError::PACKET_TOO_SMALL)
        ));

        // max + 1
        assert!(matches!(
            th.spe.write_fifo(&packet).await,
            Err(AdinError::PACKET_TOO_BIG)
        ));

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_arp_46bytes_with_crc() {
        // Configure expectations
        let mut expectations = vec![];

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 136, 0, 0, 0, 66, 201]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // Header
        expectations.push(SpiTransaction::write_vec(vec![160, 49, 143, 0, 0]));
        // Packet data
        let packet = [
            34, 51, 68, 85, 102, 119, 18, 52, 86, 120, 154, 188, 8, 6, 0, 1, 8, 0, 6, 4, 0, 2, 18, 52, 86, 120, 154,
            188, 192, 168, 16, 4, 34, 51, 68, 85, 102, 119, 192, 168, 16, 1,
        ];
        expectations.push(SpiTransaction::write_vec(packet.to_vec()));

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);
        // Padding
        if let Some(padding_len) = (ETH_MIN_LEN - FCS_LEN).checked_sub(packet.len()) {
            tail.resize(padding_len, 0x00);
        }
        // Packet FCS + optinal padding
        tail.extend_from_slice(&[147, 149, 213, 68, DONT_CARE_BYTE, DONT_CARE_BYTE]);

        expectations.push(SpiTransaction::write_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        assert!(th.spe.write_fifo(&packet).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn write_packet_to_fifo_arp_46bytes_without_crc() {
        // Configure expectations
        let mut expectations = vec![];

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 0, 0, 0, 66]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // SPI Header + Frame Header
        expectations.push(SpiTransaction::write_vec(vec![160, 49, 0, 0]));
        // Packet data
        let packet = [
            34, 51, 68, 85, 102, 119, 18, 52, 86, 120, 154, 188, 8, 6, 0, 1, 8, 0, 6, 4, 0, 2, 18, 52, 86, 120, 154,
            188, 192, 168, 16, 4, 34, 51, 68, 85, 102, 119, 192, 168, 16, 1,
        ];
        expectations.push(SpiTransaction::write_vec(packet.to_vec()));

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);
        // Padding
        if let Some(padding_len) = (ETH_MIN_LEN - FCS_LEN).checked_sub(packet.len()) {
            tail.resize(padding_len, 0x00);
        }
        // Packet FCS + optinal padding
        tail.extend_from_slice(&[147, 149, 213, 68, DONT_CARE_BYTE, DONT_CARE_BYTE]);

        expectations.push(SpiTransaction::write_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, false, true);

        assert!(th.spe.write_fifo(&packet).await.is_ok());

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn read_packet_from_fifo_packet_too_big_for_frame_buffer() {
        // Configure expectations
        let mut expectations = vec![];

        // Read RX_SIZE reg
        let rx_size: u32 = u32::try_from(ETH_MIN_LEN + FRAME_HEADER_LEN + FCS_LEN).unwrap();
        let mut rx_size_vec = rx_size.to_be_bytes().to_vec();
        rx_size_vec.push(crc8(&rx_size_vec));

        expectations.push(SpiTransaction::write_vec(vec![128, 144, 79, TURN_AROUND_BYTE]));
        expectations.push(SpiTransaction::read_vec(rx_size_vec));
        expectations.push(SpiTransaction::flush());

        let mut frame = [0; MTU];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        let ret = th.spe.read_fifo(&mut frame[0..ETH_MIN_LEN - 1]).await;
        assert!(matches!(dbg!(ret), Err(AdinError::PACKET_TOO_BIG)));

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn read_packet_from_fifo_packet_too_small() {
        // Configure expectations
        let mut expectations = vec![];

        // This value is importen for this test!
        assert_eq!(ETH_MIN_LEN, 64);

        // Packet data, size = `ETH_MIN_LEN` - `FCS_LEN` - 1
        let packet = [0; 64 - FCS_LEN - 1];

        // Read RX_SIZE reg
        let rx_size: u32 = u32::try_from(packet.len() + FRAME_HEADER_LEN + FCS_LEN).unwrap();
        let mut rx_size_vec = rx_size.to_be_bytes().to_vec();
        rx_size_vec.push(crc8(&rx_size_vec));

        expectations.push(SpiTransaction::write_vec(vec![128, 144, 79, TURN_AROUND_BYTE]));
        expectations.push(SpiTransaction::read_vec(rx_size_vec));
        expectations.push(SpiTransaction::flush());

        let mut frame = [0; MTU];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, true);

        let ret = th.spe.read_fifo(&mut frame).await;
        assert!(matches!(dbg!(ret), Err(AdinError::PACKET_TOO_SMALL)));

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn read_packet_from_fifo_packet_corrupted_fcs() {
        let mut frame = [0; MTU];
        // Configure expectations
        let mut expectations = vec![];

        let packet = [0xDE; 60];
        let crc_en = true;

        // Read RX_SIZE reg
        let rx_size: u32 = u32::try_from(packet.len() + FRAME_HEADER_LEN + FCS_LEN).unwrap();
        let mut rx_size_vec = rx_size.to_be_bytes().to_vec();
        if crc_en {
            rx_size_vec.push(crc8(&rx_size_vec));
        }

        // SPI Header with CRC
        let mut rx_fsize = vec![128, 144, 79, TURN_AROUND_BYTE];
        if !crc_en {
            // remove the CRC on idx 2
            rx_fsize.swap_remove(2);
        }
        expectations.push(SpiTransaction::write_vec(rx_fsize));
        expectations.push(SpiTransaction::read_vec(rx_size_vec));
        expectations.push(SpiTransaction::flush());

        // Read RX reg, SPI Header with CRC
        let mut rx_reg = vec![128, 145, 72, TURN_AROUND_BYTE];
        if !crc_en {
            // remove the CRC on idx 2
            rx_reg.swap_remove(2);
        }
        expectations.push(SpiTransaction::write_vec(rx_reg));
        // Frame Header
        expectations.push(SpiTransaction::read_vec(vec![0, 0]));
        // Packet data
        expectations.push(SpiTransaction::read_vec(packet.to_vec()));

        let packet_crc = ETH_FCS::new(&packet);

        let mut tail = std::vec::Vec::<u8>::with_capacity(100);

        tail.extend_from_slice(&packet_crc.hton_bytes());
        // increase last byte with 1.
        if let Some(crc) = tail.last_mut() {
            *crc = crc.wrapping_add(1);
        }

        // Need extra bytes?
        let pad = (packet.len() + FCS_LEN + FRAME_HEADER_LEN) & 0x03;
        if pad != 0 {
            // Packet FCS + optinal padding
            tail.resize(tail.len() + pad, DONT_CARE_BYTE);
        }

        expectations.push(SpiTransaction::read_vec(tail));
        expectations.push(SpiTransaction::flush());

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, crc_en, false);

        let ret = th.spe.read_fifo(&mut frame).await.expect_err("Error!");
        assert!(matches!(ret, AdinError::FCS));

        // Mark end of the SPI test.
        th.done();
    }

    #[futures_test::test]
    async fn read_packet_to_fifo_check_spi_read_multipule_of_u32_valid_lengths() {
        let packet_buffer = [0; MTU];
        let mut frame = [0; MTU];
        let mut expectations = std::vec::Vec::with_capacity(16);

        // Packet data, size = `ETH_MIN_LEN` - `FCS_LEN`
        for packet_size in [60, 61, 62, 63, 64, MTU - 4, MTU - 3, MTU - 2, MTU - 1, MTU] {
            for crc_en in [false, true] {
                expectations.clear();

                let packet = &packet_buffer[0..packet_size];

                // Read RX_SIZE reg
                let rx_size: u32 = u32::try_from(packet.len() + FRAME_HEADER_LEN + FCS_LEN).unwrap();
                let mut rx_size_vec = rx_size.to_be_bytes().to_vec();
                if crc_en {
                    rx_size_vec.push(crc8(&rx_size_vec));
                }

                // SPI Header with CRC
                let mut rx_fsize = vec![128, 144, 79, TURN_AROUND_BYTE];
                if !crc_en {
                    // remove the CRC on idx 2
                    rx_fsize.swap_remove(2);
                }
                expectations.push(SpiTransaction::write_vec(rx_fsize));
                expectations.push(SpiTransaction::read_vec(rx_size_vec));
                expectations.push(SpiTransaction::flush());

                // Read RX reg, SPI Header with CRC
                let mut rx_reg = vec![128, 145, 72, TURN_AROUND_BYTE];
                if !crc_en {
                    // remove the CRC on idx 2
                    rx_reg.swap_remove(2);
                }
                expectations.push(SpiTransaction::write_vec(rx_reg));
                // Frame Header
                expectations.push(SpiTransaction::read_vec(vec![0, 0]));
                // Packet data
                expectations.push(SpiTransaction::read_vec(packet.to_vec()));

                let packet_crc = ETH_FCS::new(packet);

                let mut tail = std::vec::Vec::<u8>::with_capacity(100);

                tail.extend_from_slice(&packet_crc.hton_bytes());

                // Need extra bytes?
                let pad = (packet_size + FCS_LEN + FRAME_HEADER_LEN) & 0x03;
                if pad != 0 {
                    // Packet FCS + optinal padding
                    tail.resize(tail.len() + pad, DONT_CARE_BYTE);
                }

                expectations.push(SpiTransaction::read_vec(tail));
                expectations.push(SpiTransaction::flush());

                // Create TestHarnass
                let mut th = TestHarnass::new(&expectations, crc_en, false);

                let ret = th.spe.read_fifo(&mut frame).await.expect("Error!");
                assert_eq!(ret, packet_size);

                // Mark end of the SPI test.
                th.done();
            }
        }
    }

    #[futures_test::test]
    async fn spi_crc_error() {
        // Configure expectations
        let expectations = vec![
            SpiTransaction::write_vec(vec![128, 144, 79, TURN_AROUND_BYTE]),
            SpiTransaction::read_vec(vec![0x00, 0x00, 0x00, 0x00, 0xDD]),
            SpiTransaction::flush(),
        ];

        // Create TestHarnass
        let mut th = TestHarnass::new(&expectations, true, false);

        let ret = th.spe.read_reg(sr::RX_FSIZE).await;
        assert!(matches!(dbg!(ret), Err(AdinError::SPI_CRC)));

        // Mark end of the SPI test.
        th.done();
    }
}
