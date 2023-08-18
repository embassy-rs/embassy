#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// must go first.
mod fmt;

#[macro_use]
mod macros;
mod bank0;
mod bank1;
mod bank2;
mod bank3;
mod common;
mod header;
mod phy;
mod traits;

use core::cmp;
use core::convert::TryInto;

use embassy_net_driver::{Capabilities, HardwareAddress, LinkState, Medium};
use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{Operation, SpiDevice};
use traits::U16Ext;

// Total buffer size (see section 3.2)
const BUF_SZ: u16 = 8 * 1024;

// Maximum frame length
const MAX_FRAME_LENGTH: u16 = 1518; // value recommended in the data sheet

// Size of the Frame check sequence (32-bit CRC)
const CRC_SZ: u16 = 4;

// define the boundaries of the TX and RX buffers
// to workaround errata #5 we do the opposite of what section 6.1 of the data sheet
// says: we place the RX buffer at address 0 and the TX buffer after it
const RXST: u16 = 0x0000;
const RXND: u16 = 0x19ff;
const TXST: u16 = 0x1a00;
const _TXND: u16 = 0x1fff;

const MTU: usize = 1514; // 1500 IP + 14 ethernet header

/// ENC28J60 embassy-net driver
pub struct Enc28j60<S, O> {
    mac_addr: [u8; 6],

    spi: S,
    rst: Option<O>,

    bank: Bank,

    // address of the next packet in buffer memory
    next_packet: u16,
}

impl<S, O> Enc28j60<S, O>
where
    S: SpiDevice,
    O: OutputPin,
{
    /// Create a new ENC28J60 driver instance.
    ///
    /// The RST pin is optional. If None, reset will be done with a SPI
    /// soft reset command, instead of via the RST pin.
    pub fn new(spi: S, rst: Option<O>, mac_addr: [u8; 6]) -> Self {
        let mut res = Self {
            mac_addr,
            spi,
            rst,

            bank: Bank::Bank0,
            next_packet: RXST,
        };
        res.init();
        res
    }

    fn init(&mut self) {
        if let Some(rst) = &mut self.rst {
            rst.set_low().unwrap();
            embassy_time::block_for(Duration::from_millis(5));
            rst.set_high().unwrap();
            embassy_time::block_for(Duration::from_millis(5));
        } else {
            embassy_time::block_for(Duration::from_millis(5));
            self.soft_reset();
            embassy_time::block_for(Duration::from_millis(5));
        }

        debug!(
            "enc28j60: erevid {=u8:x}",
            self.read_control_register(bank3::Register::EREVID)
        );
        debug!("enc28j60: waiting for clk");
        while common::ESTAT(self.read_control_register(common::Register::ESTAT)).clkrdy() == 0 {}
        debug!("enc28j60: clk ok");

        if self.read_control_register(bank3::Register::EREVID) == 0 {
            panic!("ErevidIsZero");
        }

        // disable CLKOUT output
        self.write_control_register(bank3::Register::ECOCON, 0);

        self.init_rx();

        // TX start
        // "It is recommended that an even address be used for ETXST"
        debug_assert_eq!(TXST % 2, 0);
        self.write_control_register(bank0::Register::ETXSTL, TXST.low());
        self.write_control_register(bank0::Register::ETXSTH, TXST.high());

        // TX end is set in `transmit`

        // MAC initialization (see section 6.5)
        // 1. Set the MARXEN bit in MACON1 to enable the MAC to receive frames.
        self.write_control_register(
            bank2::Register::MACON1,
            bank2::MACON1::default().marxen(1).passall(0).rxpaus(1).txpaus(1).bits(),
        );

        // 2. Configure the PADCFG, TXCRCEN and FULDPX bits of MACON3.
        self.write_control_register(
            bank2::Register::MACON3,
            bank2::MACON3::default().frmlnen(1).txcrcen(1).padcfg(0b001).bits(),
        );

        // 4. Program the MAMXFL registers with the maximum frame length to be permitted to be
        // received or transmitted
        self.write_control_register(bank2::Register::MAMXFLL, MAX_FRAME_LENGTH.low());
        self.write_control_register(bank2::Register::MAMXFLH, MAX_FRAME_LENGTH.high());

        // 5. Configure the Back-to-Back Inter-Packet Gap register, MABBIPG.
        // Use recommended value of 0x12
        self.write_control_register(bank2::Register::MABBIPG, 0x12);

        // 6. Configure the Non-Back-to-Back Inter-Packet Gap register low byte, MAIPGL.
        // Use recommended value of 0x12
        self.write_control_register(bank2::Register::MAIPGL, 0x12);
        self.write_control_register(bank2::Register::MAIPGH, 0x0c);

        // 9. Program the local MAC address into the MAADR1:MAADR6 registers
        self.write_control_register(bank3::Register::MAADR1, self.mac_addr[0]);
        self.write_control_register(bank3::Register::MAADR2, self.mac_addr[1]);
        self.write_control_register(bank3::Register::MAADR3, self.mac_addr[2]);
        self.write_control_register(bank3::Register::MAADR4, self.mac_addr[3]);
        self.write_control_register(bank3::Register::MAADR5, self.mac_addr[4]);
        self.write_control_register(bank3::Register::MAADR6, self.mac_addr[5]);

        // Set the PHCON2.HDLDIS bit to prevent automatic loopback of the data which is transmitted
        self.write_phy_register(phy::Register::PHCON2, phy::PHCON2::default().hdldis(1).bits());

        // Globally enable interrupts
        //self.bit_field_set(common::Register::EIE, common::EIE::mask().intie());

        // Set the per packet control byte; we'll always use the value 0
        self.write_buffer_memory(Some(TXST), &mut [0]);

        // Enable reception
        self.bit_field_set(common::Register::ECON1, common::ECON1::mask().rxen());
    }

    fn init_rx(&mut self) {
        // RX start
        // "It is recommended that the ERXST Pointer be programmed with an even address"
        self.write_control_register(bank0::Register::ERXSTL, RXST.low());
        self.write_control_register(bank0::Register::ERXSTH, RXST.high());

        // RX read pointer
        // NOTE Errata #14 so we are using an *odd* address here instead of ERXST
        self.write_control_register(bank0::Register::ERXRDPTL, RXND.low());
        self.write_control_register(bank0::Register::ERXRDPTH, RXND.high());

        // RX end
        self.write_control_register(bank0::Register::ERXNDL, RXND.low());
        self.write_control_register(bank0::Register::ERXNDH, RXND.high());

        // decrease the packet count to 0
        while self.read_control_register(bank1::Register::EPKTCNT) != 0 {
            self.bit_field_set(common::Register::ECON2, common::ECON2::mask().pktdec());
        }

        self.next_packet = RXST;
    }

    fn reset_rx(&mut self) {
        self.bit_field_set(common::Register::ECON1, common::ECON1::mask().rxrst());
        self.bit_field_clear(common::Register::ECON1, common::ECON1::mask().rxrst());
        self.init_rx();
        self.bit_field_set(common::Register::ECON1, common::ECON1::mask().rxen());
    }

    /// Flushes the transmit buffer, ensuring all pending transmissions have completed
    /// NOTE: The returned packet *must* be `read` or `ignore`-d, otherwise this method will always
    /// return `None` on subsequent invocations
    pub fn receive<'a>(&mut self, buf: &'a mut [u8]) -> Option<&'a mut [u8]> {
        if self.pending_packets() == 0 {
            // Errata #6: we can't rely on PKTIF so we check PKTCNT
            return None;
        }

        let curr_packet = self.next_packet;

        // read out the first 6 bytes
        let mut temp_buf = [0; 6];
        self.read_buffer_memory(Some(curr_packet), &mut temp_buf);

        // next packet pointer
        let next_packet = u16::from_parts(temp_buf[0], temp_buf[1]);
        // status vector
        let status = header::RxStatus(u32::from_le_bytes(temp_buf[2..].try_into().unwrap()));
        let len_with_crc = status.byte_count() as u16;

        if len_with_crc < CRC_SZ || len_with_crc > 1600 || next_packet > RXND {
            warn!("RX buffer corrupted, resetting RX logic to recover...");
            self.reset_rx();
            return None;
        }

        let len = len_with_crc - CRC_SZ;
        self.read_buffer_memory(None, &mut buf[..len as usize]);

        // update ERXRDPT
        // due to Errata #14 we must write an odd address to ERXRDPT
        // we know that ERXST = 0, that ERXND is odd and that next_packet is even
        let rxrdpt = if self.next_packet < 1 || self.next_packet > RXND + 1 {
            RXND
        } else {
            self.next_packet - 1
        };
        // "To move ERXRDPT, the host controller must write to ERXRDPTL first."
        self.write_control_register(bank0::Register::ERXRDPTL, rxrdpt.low());
        self.write_control_register(bank0::Register::ERXRDPTH, rxrdpt.high());

        // decrease the packet count
        self.bit_field_set(common::Register::ECON2, common::ECON2::mask().pktdec());

        self.next_packet = next_packet;

        Some(&mut buf[..len as usize])
    }

    fn wait_tx_ready(&mut self) {
        for _ in 0u32..10000 {
            if common::ECON1(self.read_control_register(common::Register::ECON1)).txrts() == 0 {
                return;
            }
        }

        // work around errata #12 by resetting the transmit logic before every new
        // transmission
        self.bit_field_set(common::Register::ECON1, common::ECON1::mask().txrst());
        self.bit_field_clear(common::Register::ECON1, common::ECON1::mask().txrst());
        //self.bit_field_clear(common::Register::EIR, {
        //    let mask = common::EIR::mask();
        //    mask.txerif() | mask.txif()
        //});
    }

    /// Starts the transmission of `bytes`
    ///
    /// It's up to the caller to ensure that `bytes` is a valid Ethernet frame. The interface will
    /// take care of appending a (4 byte) CRC to the frame and of padding the frame to the minimum
    /// size allowed by the Ethernet specification (64 bytes, or 46 bytes of payload).
    ///
    /// NOTE This method will flush any previous transmission that's in progress
    ///
    /// # Panics
    ///
    /// If `bytes` length is greater than 1514, the maximum frame length allowed by the interface,
    /// or greater than the transmit buffer
    pub fn transmit(&mut self, bytes: &[u8]) {
        assert!(bytes.len() <= self.mtu() as usize);

        self.wait_tx_ready();

        // NOTE the plus one is to not overwrite the per packet control byte
        let wrpt = TXST + 1;

        // 1. ETXST was set during initialization

        // 2. write the frame to the IC memory
        self.write_buffer_memory(Some(wrpt), bytes);

        let txnd = wrpt + bytes.len() as u16 - 1;

        // 3. Set the end address of the transmit buffer
        self.write_control_register(bank0::Register::ETXNDL, txnd.low());
        self.write_control_register(bank0::Register::ETXNDH, txnd.high());

        // 4. reset interrupt flag
        //self.bit_field_clear(common::Register::EIR, common::EIR::mask().txif());

        // 5. start transmission
        self.bit_field_set(common::Register::ECON1, common::ECON1::mask().txrts());

        // Wait until transmission finishes
        //while common::ECON1(self.read_control_register(common::Register::ECON1)).txrts() == 1 {}

        /*
        // read the transmit status vector
        let mut tx_stat = [0; 7];
        self.read_buffer_memory(None, &mut tx_stat);

        let stat = common::ESTAT(self.read_control_register(common::Register::ESTAT));

        if stat.txabrt() == 1 {
            // work around errata #12 by reading the transmit status vector
            if stat.latecol() == 1 || (tx_stat[2] & (1 << 5)) != 0 {
                panic!("LateCollision")
            } else {
                panic!("TransmitAbort")
            }
        }*/
    }

    /// Get whether the link is up
    pub fn is_link_up(&mut self) -> bool {
        let bits = self.read_phy_register(phy::Register::PHSTAT2);
        phy::PHSTAT2(bits).lstat() == 1
    }

    /// Returns the interface Maximum Transmission Unit (MTU)
    ///
    /// The value returned by this function will never exceed 1514 bytes. The actual value depends
    /// on the memory assigned to the transmission buffer when initializing the device
    pub fn mtu(&self) -> u16 {
        cmp::min(BUF_SZ - RXND - 1, MAX_FRAME_LENGTH - CRC_SZ)
    }

    /* Miscellaneous */
    /// Returns the number of packets that have been received but have not been processed yet
    pub fn pending_packets(&mut self) -> u8 {
        self.read_control_register(bank1::Register::EPKTCNT)
    }

    /// Adjusts the receive filter to *accept* these packet types
    pub fn accept(&mut self, packets: &[Packet]) {
        let mask = bank1::ERXFCON::mask();
        let mut val = 0;
        for packet in packets {
            match packet {
                Packet::Broadcast => val |= mask.bcen(),
                Packet::Multicast => val |= mask.mcen(),
                Packet::Unicast => val |= mask.ucen(),
            }
        }

        self.bit_field_set(bank1::Register::ERXFCON, val)
    }

    /// Adjusts the receive filter to *ignore* these packet types
    pub fn ignore(&mut self, packets: &[Packet]) {
        let mask = bank1::ERXFCON::mask();
        let mut val = 0;
        for packet in packets {
            match packet {
                Packet::Broadcast => val |= mask.bcen(),
                Packet::Multicast => val |= mask.mcen(),
                Packet::Unicast => val |= mask.ucen(),
            }
        }

        self.bit_field_clear(bank1::Register::ERXFCON, val)
    }

    /* Private */
    /* Read */
    fn read_control_register<R>(&mut self, register: R) -> u8
    where
        R: Into<Register>,
    {
        self._read_control_register(register.into())
    }

    fn _read_control_register(&mut self, register: Register) -> u8 {
        self.change_bank(register);

        if register.is_eth_register() {
            let mut buffer = [Instruction::RCR.opcode() | register.addr(), 0];
            self.spi.transfer_in_place(&mut buffer).unwrap();
            buffer[1]
        } else {
            // MAC, MII regs need a dummy byte.
            let mut buffer = [Instruction::RCR.opcode() | register.addr(), 0, 0];
            self.spi.transfer_in_place(&mut buffer).unwrap();
            buffer[2]
        }
    }

    fn read_phy_register(&mut self, register: phy::Register) -> u16 {
        // set PHY register address
        self.write_control_register(bank2::Register::MIREGADR, register.addr());

        // start read operation
        self.write_control_register(bank2::Register::MICMD, bank2::MICMD::default().miird(1).bits());

        // wait until the read operation finishes
        while self.read_control_register(bank3::Register::MISTAT) & 0b1 != 0 {}

        self.write_control_register(bank2::Register::MICMD, bank2::MICMD::default().miird(0).bits());

        let l = self.read_control_register(bank2::Register::MIRDL);
        let h = self.read_control_register(bank2::Register::MIRDH);
        (l as u16) | (h as u16) << 8
    }

    /* Write */
    fn _write_control_register(&mut self, register: Register, value: u8) {
        self.change_bank(register);

        let buffer = [Instruction::WCR.opcode() | register.addr(), value];
        self.spi.write(&buffer).unwrap();
    }

    fn write_control_register<R>(&mut self, register: R, value: u8)
    where
        R: Into<Register>,
    {
        self._write_control_register(register.into(), value)
    }

    fn write_phy_register(&mut self, register: phy::Register, value: u16) {
        // set PHY register address
        self.write_control_register(bank2::Register::MIREGADR, register.addr());

        self.write_control_register(bank2::Register::MIWRL, (value & 0xff) as u8);
        // this starts the write operation
        self.write_control_register(bank2::Register::MIWRH, (value >> 8) as u8);

        // wait until the write operation finishes
        while self.read_control_register(bank3::Register::MISTAT) & 0b1 != 0 {}
    }

    /* RMW */
    fn modify_control_register<R, F>(&mut self, register: R, f: F)
    where
        F: FnOnce(u8) -> u8,
        R: Into<Register>,
    {
        self._modify_control_register(register.into(), f)
    }

    fn _modify_control_register<F>(&mut self, register: Register, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        let r = self._read_control_register(register);
        self._write_control_register(register, f(r))
    }

    /* Auxiliary */
    fn change_bank(&mut self, register: Register) {
        let bank = register.bank();

        if let Some(bank) = bank {
            if self.bank == bank {
                // already on the register bank
                return;
            }

            // change bank
            self.bank = bank;
            match bank {
                Bank::Bank0 => self.bit_field_clear(common::Register::ECON1, 0b11),
                Bank::Bank1 => self.modify_control_register(common::Register::ECON1, |r| (r & !0b11) | 0b01),
                Bank::Bank2 => self.modify_control_register(common::Register::ECON1, |r| (r & !0b11) | 0b10),
                Bank::Bank3 => self.bit_field_set(common::Register::ECON1, 0b11),
            }
        } else {
            // common register
        }
    }

    /* Primitive operations */
    fn bit_field_clear<R>(&mut self, register: R, mask: u8)
    where
        R: Into<Register>,
    {
        self._bit_field_clear(register.into(), mask)
    }

    fn _bit_field_clear(&mut self, register: Register, mask: u8) {
        debug_assert!(register.is_eth_register());

        self.change_bank(register);

        self.spi
            .write(&[Instruction::BFC.opcode() | register.addr(), mask])
            .unwrap();
    }

    fn bit_field_set<R>(&mut self, register: R, mask: u8)
    where
        R: Into<Register>,
    {
        self._bit_field_set(register.into(), mask)
    }

    fn _bit_field_set(&mut self, register: Register, mask: u8) {
        debug_assert!(register.is_eth_register());

        self.change_bank(register);

        self.spi
            .write(&[Instruction::BFS.opcode() | register.addr(), mask])
            .unwrap();
    }

    fn read_buffer_memory(&mut self, addr: Option<u16>, buf: &mut [u8]) {
        if let Some(addr) = addr {
            self.write_control_register(bank0::Register::ERDPTL, addr.low());
            self.write_control_register(bank0::Register::ERDPTH, addr.high());
        }

        self.spi
            .transaction(&mut [Operation::Write(&[Instruction::RBM.opcode()]), Operation::Read(buf)])
            .unwrap();
    }

    fn soft_reset(&mut self) {
        self.spi.write(&[Instruction::SRC.opcode()]).unwrap();
    }

    fn write_buffer_memory(&mut self, addr: Option<u16>, buffer: &[u8]) {
        if let Some(addr) = addr {
            self.write_control_register(bank0::Register::EWRPTL, addr.low());
            self.write_control_register(bank0::Register::EWRPTH, addr.high());
        }

        self.spi
            .transaction(&mut [Operation::Write(&[Instruction::WBM.opcode()]), Operation::Write(buffer)])
            .unwrap();
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Bank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
}

#[derive(Clone, Copy)]
enum Instruction {
    /// Read Control Register
    RCR = 0b000_00000,
    /// Read Buffer Memory
    RBM = 0b001_11010,
    /// Write Control Register
    WCR = 0b010_00000,
    /// Write Buffer Memory
    WBM = 0b011_11010,
    /// Bit Field Set
    BFS = 0b100_00000,
    /// Bit Field Clear
    BFC = 0b101_00000,
    /// System Reset Command
    SRC = 0b111_11111,
}

impl Instruction {
    fn opcode(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy)]
enum Register {
    Bank0(bank0::Register),
    Bank1(bank1::Register),
    Bank2(bank2::Register),
    Bank3(bank3::Register),
    Common(common::Register),
}

impl Register {
    fn addr(&self) -> u8 {
        match *self {
            Register::Bank0(r) => r.addr(),
            Register::Bank1(r) => r.addr(),
            Register::Bank2(r) => r.addr(),
            Register::Bank3(r) => r.addr(),
            Register::Common(r) => r.addr(),
        }
    }

    fn bank(&self) -> Option<Bank> {
        Some(match *self {
            Register::Bank0(_) => Bank::Bank0,
            Register::Bank1(_) => Bank::Bank1,
            Register::Bank2(_) => Bank::Bank2,
            Register::Bank3(_) => Bank::Bank3,
            Register::Common(_) => return None,
        })
    }

    fn is_eth_register(&self) -> bool {
        match *self {
            Register::Bank0(r) => r.is_eth_register(),
            Register::Bank1(r) => r.is_eth_register(),
            Register::Bank2(r) => r.is_eth_register(),
            Register::Bank3(r) => r.is_eth_register(),
            Register::Common(r) => r.is_eth_register(),
        }
    }
}

/// Packet type, used to configure receive filters
#[non_exhaustive]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Packet {
    /// Broadcast packets
    Broadcast,
    /// Multicast packets
    Multicast,
    /// Unicast packets
    Unicast,
}

static mut TX_BUF: [u8; MTU] = [0; MTU];
static mut RX_BUF: [u8; MTU] = [0; MTU];

impl<S, O> embassy_net_driver::Driver for Enc28j60<S, O>
where
    S: SpiDevice,
    O: OutputPin,
{
    type RxToken<'a> = RxToken<'a>
    where
        Self: 'a;

    type TxToken<'a> = TxToken<'a, S, O>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut core::task::Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_buf = unsafe { &mut RX_BUF };
        let tx_buf = unsafe { &mut TX_BUF };
        if let Some(pkt) = self.receive(rx_buf) {
            let n = pkt.len();
            Some((RxToken { buf: &mut pkt[..n] }, TxToken { buf: tx_buf, eth: self }))
        } else {
            cx.waker().wake_by_ref();
            None
        }
    }

    fn transmit(&mut self, _cx: &mut core::task::Context) -> Option<Self::TxToken<'_>> {
        let tx_buf = unsafe { &mut TX_BUF };
        Some(TxToken { buf: tx_buf, eth: self })
    }

    fn link_state(&mut self, cx: &mut core::task::Context) -> LinkState {
        cx.waker().wake_by_ref();
        match self.is_link_up() {
            true => LinkState::Up,
            false => LinkState::Down,
        }
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        caps.medium = Medium::Ethernet;
        caps
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.mac_addr)
    }
}

/// embassy-net RX token.
pub struct RxToken<'a> {
    buf: &'a mut [u8],
}

impl<'a> embassy_net_driver::RxToken for RxToken<'a> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(self.buf)
    }
}

/// embassy-net TX token.
pub struct TxToken<'a, S, O>
where
    S: SpiDevice,
    O: OutputPin,
{
    eth: &'a mut Enc28j60<S, O>,
    buf: &'a mut [u8],
}

impl<'a, S, O> embassy_net_driver::TxToken for TxToken<'a, S, O>
where
    S: SpiDevice,
    O: OutputPin,
{
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        assert!(len <= self.buf.len());
        let r = f(&mut self.buf[..len]);
        self.eth.transmit(&self.buf[..len]);
        r
    }
}
