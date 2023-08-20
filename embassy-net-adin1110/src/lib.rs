#![deny(clippy::pedantic)]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod crc32;
mod crc8;
mod mdio;
mod phy;
mod regs;

use ch::driver::LinkState;
pub use crc32::ETH_FSC;
use crc8::crc8;
use embassy_futures::select::{select, Either};
use embassy_net_driver_channel as ch;
use embassy_time::{Duration, Timer};
use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{Operation, SpiDevice};
use heapless::Vec;
pub use mdio::MdioBus;
pub use phy::{Phy10BaseT1x, RegsC22, RegsC45};
pub use regs::{Config0, Config2, SpiRegisters as sr, Status0, Status1};

use crate::regs::{LedCntrl, LedFunc, LedPol, LedPolarity, SpiHeader};

pub const PHYID: u32 = 0x0283_BC91;

/// Error values ADIN1110
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(non_camel_case_types)]
pub enum AdinError<E> {
    Spi(E),
    SENDERROR,
    READERROR,
    CRC,
    PACKET_TOO_BIG,
    PACKET_TOO_SMALL,
    MDIO_ACC_TIMEOUT,
}

pub type AEResult<T, SPIE> = core::result::Result<T, AdinError<SPIE>>;
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
const FSC_LEN: usize = 4;
const FRAME_HEADER_LEN: usize = 2;
const WR_HEADER_LEN: usize = 2;

// P1 = 0x00, P2 = 0x01
const PORT_ID_BYTE: u8 = 0x00;

pub type Packet = Vec<u8, { MTU + FSC_LEN + WR_HEADER_LEN }>;

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

#[derive(Debug)]
pub struct ADIN1110<SPI> {
    /// SPI bus
    spi: SPI,
    /// Enable CRC on SPI transfer.
    /// This must match with the hardware pin `SPI_CFG0` were low = CRC enable, high = CRC disabled.
    crc: bool,
}

/// Round size up the N u32;
pub(crate) fn size_align_u32(size: u32) -> u32 {
    (size + 3) & 0xFFFF_FFFC
}

impl<SpiE, SPI> ADIN1110<SPI>
where
    SPI: SpiDevice<u8, Error = SpiE>,
    SpiE: core::fmt::Debug,
{
    pub fn new(spi: SPI, crc: bool) -> Self {
        Self { spi, crc }
    }

    pub async fn read_reg(&mut self, reg: sr) -> AEResult<u32, SpiE> {
        let mut tx_buf = Vec::<u8, 16>::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_addr(reg);
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        // Turn around byte, TODO: Unknown that this is.
        let _ = tx_buf.push(TURN_AROUND_BYTE);

        let mut rx_buf = [0; 5];

        let spi_read_len = if self.crc { rx_buf.len() } else { rx_buf.len() - 1 };

        let mut spi_op = [Operation::Write(&tx_buf), Operation::Read(&mut rx_buf[0..spi_read_len])];

        self.spi.transaction(&mut spi_op).await.map_err(AdinError::Spi)?;

        if self.crc {
            let crc = crc8(&rx_buf[0..4]);
            if crc != rx_buf[4] {
                return Err(AdinError::CRC);
            }
        }

        let value = u32::from_be_bytes(rx_buf[0..4].try_into().unwrap());

        #[cfg(feature = "defmt")]
        defmt::trace!("REG Read {} = {:08x} SPI {:02x}", reg, value, &tx_buf);

        Ok(value)
    }

    pub async fn write_reg(&mut self, reg: sr, value: u32) -> AEResult<(), SpiE> {
        let mut tx_buf = Vec::<u8, 16>::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_write(true);
        spi_hdr.set_addr(reg);
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        let val = value.to_be_bytes();
        let _ = tx_buf.extend_from_slice(val.as_slice());

        if self.crc {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(val.as_slice()));
        }

        #[cfg(feature = "defmt")]
        defmt::trace!("REG Write {} = {:08x} SPI {:02x}", reg, value, &tx_buf);

        self.spi.write(&tx_buf).await.map_err(AdinError::Spi)
    }

    /// helper function for write to `MDIO_ACC` register and wait for ready!
    async fn write_mdio_acc_reg(&mut self, mdio_acc_val: u32) -> AEResult<u32, SpiE> {
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
    pub async fn read_fifo(&mut self, packet: &mut [u8]) -> AEResult<usize, SpiE> {
        let mut tx_buf = Vec::<u8, 16>::new();

        // Size of the frame, also includes the appednded header.
        let packet_size = self.read_reg(sr::RX_FSIZE).await?;

        // Packet read of write to the MAC packet buffer must be a multipul of 4!
        let read_size = size_align_u32(packet_size);

        if packet_size < u32::try_from(FRAME_HEADER_LEN + FSC_LEN).unwrap()
            || read_size > u32::try_from(packet.len()).unwrap()
        {
            return Err(AdinError::PACKET_TOO_BIG);
        }

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_addr(sr::RX);
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        // Turn around byte, TODO: Unknown that this is.
        let _ = tx_buf.push(TURN_AROUND_BYTE);

        let spi_packet = &mut packet[0..read_size as usize];

        assert_eq!(spi_packet.len() & 0x03, 0x00);

        let mut pkt_header = [0, 0];

        let mut spi_op = [
            Operation::Write(&tx_buf),
            Operation::Read(&mut pkt_header),
            Operation::Read(spi_packet),
        ];

        self.spi.transaction(&mut spi_op).await.map_err(AdinError::Spi)?;

        Ok(packet_size as usize)
    }

    /// Write to fifo ethernet packet memory send over the wire.
    pub async fn write_fifo(&mut self, frame: &[u8]) -> AEResult<(), SpiE> {
        let header_len = self.header_write_len();

        let mut packet = Packet::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_write(true);
        spi_hdr.set_addr(sr::TX);

        packet
            .extend_from_slice(spi_hdr.0.to_be_bytes().as_slice())
            .map_err(|_| AdinError::PACKET_TOO_BIG)?;

        if self.crc {
            // Add CRC for header data
            packet
                .push(crc8(&packet[0..2]))
                .map_err(|_| AdinError::PACKET_TOO_BIG)?;
        }

        // Add port number, ADIN1110 its fixed to zero/P1, but for ADIN2111 has two ports.
        packet
            .extend_from_slice(u16::from(PORT_ID_BYTE).to_be_bytes().as_slice())
            .map_err(|_| AdinError::PACKET_TOO_BIG)?;

        // Copy packet data to spi buffer.
        packet.extend_from_slice(frame).map_err(|_| AdinError::PACKET_TOO_BIG)?;

        // Pad data up to ETH_MIN_LEN - FCS_LEN
        for _ in packet.len()..(ETH_MIN_LEN - FSC_LEN + header_len) {
            let _ = packet.push(0x00);
        }

        // add ethernet FCS only over the ethernet packet.
        let crc = ETH_FSC::new(&packet[header_len..]);
        let _ = packet.extend_from_slice(crc.hton_bytes().as_slice());

        let send_len =
            u32::try_from(packet.len() - header_len + FRAME_HEADER_LEN).map_err(|_| AdinError::PACKET_TOO_BIG)?;

        // Packet read of write to the MAC packet buffer must be a multipul of 4 bytes!
        while packet.len() & 0x3 != 0 {
            let _ = packet.push(DONT_CARE_BYTE);
        }

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "TX: hdr {} [{}] {:02x} SIZE: {}",
            header_len,
            packet.len(),
            &packet,
            send_len,
        );

        self.write_reg(sr::TX_FSIZE, send_len).await?;

        // Spi packet must be half word / even length
        if send_len & 1 != 0 {
            let _ = packet.push(0x00);
        }

        self.spi.write(&packet).await.map_err(AdinError::Spi)
    }

    pub fn header_write_len(&self) -> usize {
        // u16 + [CRC] + PORT
        WR_HEADER_LEN + FRAME_HEADER_LEN + usize::from(self.crc)
    }

    pub fn header_len_read(&self) -> usize {
        // u16 + [CRC] + u8
        WR_HEADER_LEN + 1 + usize::from(self.crc)
    }

    /// Programs the mac address in the mac filters.
    /// Also set the boardcast address.
    /// The chip supports 2 priority queues but current code doesn't support this mode.
    pub async fn set_mac_addr(&mut self, mac: &[u8; 6]) -> AEResult<(), SpiE> {
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

impl<SpiE, SPI> mdio::MdioBus for ADIN1110<SPI>
where
    SPI: SpiDevice<u8, Error = SpiE>,
    SpiE: core::fmt::Debug,
{
    type Error = AdinError<SpiE>;

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
    async fn write_cl45(&mut self, phy_id: u8, regc45: (u8, u16), value: u16) -> AEResult<(), SpiE> {
        let phy_id = u32::from(phy_id & 0x1F) << 21;
        let dev_addr = u32::from(regc45.0 & 0x1F) << 16;
        let reg = u32::from(regc45.1);

        let mdio_acc_val: u32 = phy_id | dev_addr | reg;
        self.write_mdio_acc_reg(mdio_acc_val).await?;

        let mdio_acc_val: u32 = phy_id | dev_addr | (0x01 << 26) | u32::from(value);
        self.write_mdio_acc_reg(mdio_acc_val).await.map(|_| ())
    }
}

/// Background runner for the ADIN110.
///
/// You must call `.run()` in a background task for the ADIN1100 to operate.
pub struct Runner<'d, SPI, INT, RST> {
    mac: ADIN1110<SPI>,
    ch: ch::Runner<'d, MTU>,
    int: INT,
    is_link_up: bool,
    _reset: RST,
}

impl<'d, SPI: SpiDevice, INT: Wait, RST: OutputPin> Runner<'d, SPI, INT, RST> {
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self) -> ! {
        loop {
            let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();

            loop {
                #[cfg(feature = "defmt")]
                defmt::debug!("Waiting for interrupts");
                match select(self.int.wait_for_low(), tx_chan.tx_buf()).await {
                    Either::First(_) => {
                        let mut status1_clr = Status1(0);
                        let mut status1 = Status1(self.mac.read_reg(sr::STATUS1).await.unwrap());

                        while status1.p1_rx_rdy() {
                            #[cfg(feature = "defmt")]
                            defmt::debug!("alloc RX packet buffer");
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
                                            #[cfg(feature = "defmt")]
                                            defmt::error!("RX Packet to big, DROP");
                                            self.mac.write_reg(sr::FIFO_CLR, 1).await.unwrap();
                                        }
                                        AdinError::Spi(_) => {
                                            #[cfg(feature = "defmt")]
                                            defmt::error!("RX Spi error")
                                        }
                                        _ => {
                                            #[cfg(feature = "defmt")]
                                            defmt::error!("RX Error")
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
                            #[cfg(feature = "defmt")]
                            defmt::error!("SPE CHIP STATUS 0:{:08x} 1:{:08x}", status0.0, status1.0);
                        }

                        if status1.tx_rdy() {
                            status1_clr.set_tx_rdy(true);
                            #[cfg(feature = "defmt")]
                            defmt::info!("TX_DONE");
                        }

                        if status1.link_change() {
                            let link = status1.p1_link_status();
                            self.is_link_up = link;

                            #[cfg(feature = "defmt")]
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

                                defmt::info!("LINK Changed: Link Up, Volt: {} V p-p, MSE: {:0004}", volt, mse);
                            } else {
                                defmt::info!("LINK Changed: Link Down");
                            }

                            state_chan.set_link_state(if link { LinkState::Up } else { LinkState::Down });
                            status1_clr.set_link_change(true);
                        }

                        if status1.tx_ecc_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("SPI TX_ECC_ERR error, CLEAR TX FIFO");
                            self.mac.write_reg(sr::FIFO_CLR, 2).await.unwrap();
                            status1_clr.set_tx_ecc_err(true);
                        }

                        if status1.rx_ecc_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("SPI RX_ECC_ERR error");
                            status1_clr.set_rx_ecc_err(true);
                        }

                        if status1.spi_err() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("SPI SPI_ERR CRC error");
                            status1_clr.set_spi_err(true);
                        }

                        if status0.phyint() {
                            #[cfg_attr(not(feature = "defmt"), allow(unused_variables))]
                            let crsm_irq_st = self
                                .mac
                                .read_cl45(MDIO_PHY_ADDR, RegsC45::DA1E::CRSM_IRQ_STATUS.into())
                                .await
                                .unwrap();

                            #[cfg_attr(not(feature = "defmt"), allow(unused_variables))]
                            let phy_irq_st = self
                                .mac
                                .read_cl45(MDIO_PHY_ADDR, RegsC45::DA1F::PHY_SYBSYS_IRQ_STATUS.into())
                                .await
                                .unwrap();

                            #[cfg(feature = "defmt")]
                            defmt::warn!(
                                "SPE CHIP PHY CRSM_IRQ_STATUS {:04x} PHY_SUBSYS_IRQ_STATUS {:04x}",
                                crsm_irq_st,
                                phy_irq_st
                            );
                        }

                        if status0.txfcse() {
                            #[cfg(feature = "defmt")]
                            defmt::error!("SPE CHIP PHY TX Frame CRC error");
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

/// Obtain a driver for using the ADIN1110 with [`embassy-net`](crates.io/crates/embassy-net).
pub async fn new<const N_RX: usize, const N_TX: usize, SPI: SpiDevice, INT: Wait, RST: OutputPin>(
    mac_addr: [u8; 6],
    state: &'_ mut State<N_RX, N_TX>,
    spi_dev: SPI,
    int: INT,
    mut reset: RST,
    crc: bool,
) -> (Device<'_>, Runner<'_, SPI, INT, RST>) {
    use crate::regs::{IMask0, IMask1};

    #[cfg(feature = "defmt")]
    defmt::info!("INIT ADIN1110");

    // Reset sequence
    reset.set_low().unwrap();

    // Wait t1: 20-43mS
    Timer::after(Duration::from_millis(30)).await;

    reset.set_high().unwrap();

    // Wait t3: 50mS
    Timer::after(Duration::from_millis(50)).await;

    // Create device
    let mut mac = ADIN1110::new(spi_dev, crc);

    // Check PHYID
    let id = mac.read_reg(sr::PHYID).await.unwrap();
    assert_eq!(id, PHYID);

    #[cfg(feature = "defmt")]
    defmt::debug!("SPE: CHIP MAC/ID: {:08x}", id);

    #[cfg(feature = "defmt")]
    let adin_phy = Phy10BaseT1x::default();
    #[cfg(feature = "defmt")]
    let phy_id = adin_phy.get_id(&mut mac).await.unwrap();
    #[cfg(feature = "defmt")]
    defmt::debug!("SPE: CHIP: PHY ID: {:08x}", phy_id);

    let mi_control = mac.read_cl22(MDIO_PHY_ADDR, RegsC22::CONTROL as u8).await.unwrap();
    #[cfg(feature = "defmt")]
    defmt::println!("SPE CHIP PHY MI_CONTROL {:04x}", mi_control);
    if mi_control & 0x0800 != 0 {
        let val = mi_control & !0x0800;
        #[cfg(feature = "defmt")]
        defmt::println!("SPE CHIP PHY MI_CONTROL Disable PowerDown");
        mac.write_cl22(MDIO_PHY_ADDR, RegsC22::CONTROL as u8, val)
            .await
            .unwrap();
    }

    // Config2: CRC_APPEND
    let mut config2 = Config2(0x0000_0800);
    config2.set_crc_append(true);
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
    use embedded_hal_async::delay::DelayUs;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

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

    impl DelayUs for MockDelay {
        async fn delay_us(&mut self, _us: u32) {
            todo!()
        }

        async fn delay_ms(&mut self, _ms: u32) {
            todo!()
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
        let mut spi = SpiMock::new(&expectations);

        let cs = CsPinMock::default();
        let delay = MockDelay {};
        let spi_dev = ExclusiveDevice::new(spi.clone(), cs, delay);
        let mut spe = ADIN1110::new(spi_dev, false);

        // Read PHIID
        let val = spe.read_reg(sr::PHYID).await.expect("Error");
        assert_eq!(val, 0x0283_BC91);

        // Read CAPAVILITY
        let val = spe.read_reg(sr::CAPABILITY).await.expect("Error");
        assert_eq!(val, 0x0000_06C3);

        spi.done();
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
        let mut spi = SpiMock::new(&expectations);

        let cs = CsPinMock::default();
        let delay = MockDelay {};
        let spi_dev = ExclusiveDevice::new(spi.clone(), cs, delay);

        let mut spe = ADIN1110::new(spi_dev, true);

        assert_eq!(crc8(0x0283_BC91_u32.to_be_bytes().as_slice()), 215);
        assert_eq!(crc8(0x0000_06C3_u32.to_be_bytes().as_slice()), 57);

        // Read PHIID
        let val = spe.read_reg(sr::PHYID).await.expect("Error");
        assert_eq!(val, 0x0283_BC91);

        // Read CAPAVILITY
        let val = spe.read_reg(sr::CAPABILITY).await.expect("Error");
        assert_eq!(val, 0x0000_06C3);

        spi.done();
    }

    #[futures_test::test]
    async fn mac_write_registers_without_crc() {
        // Configure expectations
        let expectations = [
            SpiTransaction::write_vec(vec![0xA0, 0x09, 0x12, 0x34, 0x56, 0x78]),
            SpiTransaction::flush(),
        ];
        let mut spi = SpiMock::new(&expectations);

        let cs = CsPinMock::default();
        let delay = MockDelay {};
        let spi_dev = ExclusiveDevice::new(spi.clone(), cs, delay);

        let mut spe = ADIN1110::new(spi_dev, false);

        // Write reg: 0x1FFF
        assert!(spe.write_reg(sr::STATUS1, 0x1234_5678).await.is_ok());

        spi.done();
    }

    #[futures_test::test]
    async fn mac_write_registers_with_crc() {
        // Configure expectations
        let expectations = [
            SpiTransaction::write_vec(vec![0xA0, 0x09, 39, 0x12, 0x34, 0x56, 0x78, 28]),
            SpiTransaction::flush(),
        ];
        let mut spi = SpiMock::new(&expectations);

        let cs = CsPinMock::default();
        let delay = MockDelay {};
        let spi_dev = ExclusiveDevice::new(spi.clone(), cs, delay);

        let mut spe = ADIN1110::new(spi_dev, true);

        // Write reg: 0x1FFF
        assert!(spe.write_reg(sr::STATUS1, 0x1234_5678).await.is_ok());

        spi.done();
    }

    #[test]
    fn align_size() {
        assert_eq!(size_align_u32(1), 4);
        assert_eq!(size_align_u32(2), 4);
        assert_eq!(size_align_u32(3), 4);
        assert_eq!(size_align_u32(4), 4);
        assert_eq!(size_align_u32(5), 8);
        assert_eq!(size_align_u32(6), 8);
        assert_eq!(size_align_u32(7), 8);
        assert_eq!(size_align_u32(8), 8);
    }

    //     #[test]
    //     fn write_packet_to_fifo_less_64b_with_crc() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(136),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(66),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(201),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(ETH_MIN_LEN, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // padding
    //         for _ in packet.len()..65 {
    //             expectations.push(SpiTransaction::send(0x00));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[8, 137, 18, 4] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, true);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    //     #[test]
    //     fn write_packet_to_fifo_less_64b_no_crc() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(66),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(ETH_MIN_LEN, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // padding
    //         for _ in packet.len() as u32..ETH_MIN_LEN {
    //             expectations.push(SpiTransaction::send(0x00));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[8, 137, 18, 4] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, false);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    //     #[test]
    //     fn write_packet_to_fifo_1500b() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x05),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0xDE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(1500, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[212, 114, 18, 50] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, false);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    //     #[test]
    //     fn write_packet_to_fifo_65b() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(67),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(65, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // padding
    //         for _ in packet.len()..ETH_MIN_LEN {
    //             expectations.push(SpiTransaction::send(0x00));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[54, 117, 221, 220] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, false);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    //     #[test]
    //     fn write_packet_to_fifo_66b() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(68),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(66, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // padding
    //         for _ in packet.len()..ETH_MIN_LEN {
    //             expectations.push(SpiTransaction::send(0x00));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[97, 167, 100, 29] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }
    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, false);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    //     #[test]
    //     fn write_packet_to_fifo_67b() {
    //         // Configure expectations
    //         let mut expectations = vec![
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x30),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Frame Size
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(69),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // HEADER
    //             SpiTransaction::send(0xA0),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(0x31),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             // Port
    //             SpiTransaction::send(0x00),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //             SpiTransaction::send(PORT_ID_BYTE),
    //             SpiTransaction::read(DONT_CARE_BYTE),
    //         ];

    //         let mut packet = Packet::new();
    //         packet.resize(67, 0).unwrap();

    //         for &byte in &packet[4..] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // padding
    //         for _ in packet.len()..ETH_MIN_LEN {
    //             expectations.push(SpiTransaction::send(0x00));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }

    //         // fcs
    //         for &byte in &[228, 218, 170, 232] {
    //             expectations.push(SpiTransaction::send(byte));
    //             expectations.push(SpiTransaction::read(DONT_CARE_BYTE));
    //         }
    //         let spi = SpiMock::new(&expectations);

    //         let cs = CsPinMock {};
    //         let mut spe = Adin1110::new(spi, cs, false);

    //         assert!(spe.write_fifo(&mut packet).is_ok());
    //     }

    #[futures_test::test]
    async fn write_packet_to_fifo_arp_46bytes() {
        // Configure expectations
        let mut expectations = vec![];

        let mut packet = Packet::new();
        //arp packet;
        packet
            .extend_from_slice(&[
                34, 51, 68, 85, 102, 119, 18, 52, 86, 120, 154, 188, 8, 6, 0, 1, 8, 0, 6, 4, 0, 2, 18, 52, 86, 120,
                154, 188, 192, 168, 16, 4, 34, 51, 68, 85, 102, 119, 192, 168, 16, 1,
            ])
            .unwrap();

        let mut spi_packet = Packet::new();

        // Write TX_SIZE reg
        expectations.push(SpiTransaction::write_vec(vec![160, 48, 136, 0, 0, 0, 66, 201]));
        expectations.push(SpiTransaction::flush());

        // Write TX reg.
        // Header
        spi_packet.extend_from_slice(&[160, 49, 143, 0, 0]).unwrap();
        // Packet data
        spi_packet.extend_from_slice(&packet).unwrap();
        // Packet padding up to 60 (ETH_MIN_LEN - FCS)
        for _ in packet.len()..(ETH_MIN_LEN - FSC_LEN) {
            spi_packet.push(0x00).unwrap();
        }
        // Packet FCS
        spi_packet.extend_from_slice(&[147, 149, 213, 68]).unwrap();

        let spi_packet_len = u32::try_from(spi_packet.len()).unwrap();

        // SPI HEADER Padding of u32
        for _ in spi_packet_len..size_align_u32(spi_packet_len) {
            spi_packet.push(0x00).unwrap();
        }

        expectations.push(SpiTransaction::write_vec(spi_packet.to_vec()));
        expectations.push(SpiTransaction::flush());

        let mut spi = SpiMock::new(&expectations);

        let cs = CsPinMock::default();
        let delay = MockDelay {};
        let spi_dev = ExclusiveDevice::new(spi.clone(), cs, delay);

        let mut spe = ADIN1110::new(spi_dev, true);

        assert!(spe.write_fifo(&packet).await.is_ok());

        spi.done();
    }
}
