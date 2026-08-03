//! OPEN Alliance TC6 (10BASE-T1x MAC-PHY Serial Interface) protocol implementation.
//!
//! Used by the ADIN1110/ADIN2111 when the `SPI_CFG0` strap selects OPEN Alliance
//! SPI mode. The protocol is specified in the OPEN Alliance
//! "10BASE-T1x MAC-PHY Serial Interface" specification (TC6) and implemented by
//! Analog Devices' reference driver (`adi_spi_oa.c`), which this code follows.
//!
//! Two transaction types exist, selected by the DNC bit in the 4-byte header:
//!
//! * Control transactions (DNC=0) read/write registers. The MAC-PHY echoes the
//!   header one word later, so a transaction spans `N + 2` words for `N`
//!   register values.
//! * Data transactions (DNC=1) carry Ethernet frame data in fixed-size chunks
//!   of 4 bytes header/footer + 64 bytes payload. Chunks are full duplex: while
//!   the host clocks out a header and (optional) transmit payload, the MAC-PHY
//!   simultaneously clocks in receive payload followed by a footer in the last
//!   4 bytes. Frame boundaries are signalled with SV/SWO (start) and EV/EBO
//!   (end) fields; flow control uses the TXC (transmit credits) and RCA
//!   (receive chunks available) fields of the footer.

use embedded_hal_async::spi::{Operation, SpiDevice};

use super::Adin1110Protocol;
use crate::crc32::ETH_FCS;
use crate::{AdinError, ETH_MIN_LEN, FCS_LEN, MTU};

/// TC6 chunk payload size. The ADIN2111 resets to CPS=64 bytes, which is also
/// the only size this implementation supports.
const CHUNK_PAYLOAD_SIZE: usize = 64;

/// TC6 header/footer size.
const HDR_SIZE: usize = 4;

/// Total chunk size on the wire (header/footer + payload).
const CHUNK_SIZE: usize = HDR_SIZE + CHUNK_PAYLOAD_SIZE;

/// Maximum Ethernet frame size on the wire, including the FCS.
const MAX_FRAME_SIZE: usize = MTU + FCS_LEN;

/// Minimum Ethernet frame size without the FCS.
const ETH_MIN_WITHOUT_FCS_LEN: usize = ETH_MIN_LEN - FCS_LEN;

/// Registers at this address and above are located in MMS 1 on the
/// ADIN1110/ADIN2111 (vendor specific memory map). Below it is MMS 0,
/// containing the OPEN Alliance standard registers.
const MMS1_START_ADDR: u16 = 0x30;

/// Footer value returned by the MAC-PHY when the received header failed its
/// parity check: only the HDRB bit is set. Note this value itself passes the
/// footer parity check, so it must be tested for before the parity check.
const FOOTER_HDRB_ONLY: u32 = 0x4000_0000;

/// How many times to poll for TX credits / RX chunks before giving up.
const POLL_LIMIT: u32 = 50_000;

// Control command header bits (DNC=0).
const CTRL_HDR_WNR: u32 = 1 << 29;
const CTRL_HDR_MMS_SHIFT: u32 = 24;
const CTRL_HDR_ADDR_SHIFT: u32 = 8;

// Data chunk header bits (DNC=1).
const DATA_HDR_DNC: u32 = 1 << 31;
const DATA_HDR_NORX: u32 = 1 << 29;
const DATA_HDR_VS_SHIFT: u32 = 22;
const DATA_HDR_DV: u32 = 1 << 21;
const DATA_HDR_SV: u32 = 1 << 20;
const DATA_HDR_EV: u32 = 1 << 14;
const DATA_HDR_EBO_SHIFT: u32 = 8;

/// Set the parity bit (bit 0) such that the resulting 32-bit word has an odd
/// number of ones, as required for TC6 headers.
fn with_parity(val: u32) -> u32 {
    val | ((val | 1).count_ones() & 1)
}

/// Returns true if the 32-bit word has odd parity (i.e. a valid TC6 footer).
fn parity_ok(val: u32) -> bool {
    val.count_ones() & 1 == 1
}

/// TC6 receive data chunk footer.
#[derive(Debug, Clone, Copy)]
struct Footer(u32);

impl Footer {
    /// Extended status: unmasked status bits are set in STATUS0/STATUS1.
    fn exst(self) -> bool {
        self.0 & (1 << 31) != 0
    }
    /// Header bad: the MAC-PHY rejected the previously received header.
    fn hdrb(self) -> bool {
        self.0 & (1 << 30) != 0
    }
    /// Configuration synchronized: CONFIG0.SYNC has been set.
    fn sync(self) -> bool {
        self.0 & (1 << 29) != 0
    }
    /// Receive chunks available in the MAC-PHY receive buffer.
    #[allow(clippy::cast_possible_truncation)]
    fn rca(self) -> u8 {
        ((self.0 >> 24) & 0x1F) as u8
    }
    /// Data valid: the chunk payload contains receive frame data.
    fn dv(self) -> bool {
        self.0 & (1 << 21) != 0
    }
    /// Start valid: a frame starts in this chunk, at word offset SWO.
    fn sv(self) -> bool {
        self.0 & (1 << 20) != 0
    }
    /// Start word offset of the frame within the chunk payload.
    fn swo(self) -> usize {
        ((self.0 >> 16) & 0xF) as usize
    }
    /// Frame drop: the frame ending in this chunk must be discarded.
    fn fd(self) -> bool {
        self.0 & (1 << 15) != 0
    }
    /// End valid: a frame ends in this chunk, at byte offset EBO.
    fn ev(self) -> bool {
        self.0 & (1 << 14) != 0
    }
    /// End byte offset: offset of the last byte of the frame in the payload.
    fn ebo(self) -> usize {
        ((self.0 >> 8) & 0x3F) as usize
    }
    /// Transmit credits: number of chunks the host may transmit.
    #[allow(clippy::cast_possible_truncation)]
    fn txc(self) -> u8 {
        ((self.0 >> 1) & 0x1F) as u8
    }
}

/// Destination port(s) for transmitted frames on the ADIN2111.
///
/// The ADIN1110 has a single port; use [`TxPort::Port1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TxPort {
    /// Transmit on port 1 only.
    Port1,
    /// Transmit on port 2 only.
    Port2,
    /// Transmit every frame on both ports, like an unmanaged switch flooding.
    Flood,
}

/// TC6 (OPEN Alliance) protocol implementation.
pub struct Tc6<SPI> {
    spi: SPI,
    /// Append the FCS by the host on transmit (CONFIG0.TXFCSVE set) instead of
    /// letting the MAC append it (`CONFIG2.CRC_APPEND`).
    append_fcs_on_tx: bool,
    /// Port(s) to transmit frames on.
    tx_port: TxPort,
    /// Transmit credits, from the last received footer.
    txc: u8,
    /// Receive chunks available, from the last received footer.
    rca: u8,
    /// Extended status flag, from the last received footer.
    exst: bool,
    /// Reassembly buffer for the frame currently being received.
    rx_buf: [u8; MAX_FRAME_SIZE],
    /// Number of valid bytes in `rx_buf`.
    rx_len: usize,
    /// A frame start (SV) was seen and its end (EV) is still pending.
    in_frame: bool,
    /// `CONFIG0.PROTE` is set: every control data word is followed by its
    /// bitwise complement, in both directions.
    protected: bool,
}

impl<SPI> Tc6<SPI> {
    /// Create a new TC6 protocol handler.
    pub fn new(spi: SPI, append_fcs_on_tx: bool, tx_port: TxPort) -> Self {
        Self {
            spi,
            append_fcs_on_tx,
            tx_port,
            txc: 0,
            rca: 0,
            exst: false,
            rx_buf: [0; MAX_FRAME_SIZE],
            rx_len: 0,
            in_frame: false,
            protected: false,
        }
    }

    /// Select whether control transactions use the protected format, in which
    /// each data word is followed by its bitwise complement.
    ///
    /// This must match `CONFIG0.PROTE`, which some boards strap on at reset. A
    /// mismatch is silent in one direction: an unprotected read of a protected
    /// MAC-PHY still returns the right value in the first word, but an
    /// unprotected *write* is discarded without any error indication.
    pub fn set_protected(&mut self, protected: bool) {
        self.protected = protected;
    }

    /// Receive chunks are available in the MAC-PHY, as of the last footer.
    pub fn rx_available(&self) -> bool {
        self.rca > 0
    }

    /// Unmasked status bits are pending in STATUS0/STATUS1, as of the last footer.
    pub fn ext_status(&self) -> bool {
        self.exst
    }

    /// Clear the cached extended status flag (after the status registers have
    /// been read and acknowledged).
    pub fn clear_ext_status(&mut self) {
        self.exst = false;
    }

    fn control_header(write_not_read: bool, addr: u16) -> u32 {
        let mut val = 0u32;
        if write_not_read {
            val |= CTRL_HDR_WNR;
        }
        if addr >= MMS1_START_ADDR {
            val |= 1 << CTRL_HDR_MMS_SHIFT;
        }
        val |= u32::from(addr) << CTRL_HDR_ADDR_SHIFT;
        // LEN (bits 7:1) = number of registers - 1 = 0
        with_parity(val)
    }

    #[allow(clippy::fn_params_excessive_bools, clippy::cast_possible_truncation)]
    fn data_header(dv: bool, sv: bool, ev: bool, ebo: usize, port: u8, norx: bool) -> u32 {
        let mut val = DATA_HDR_DNC;
        if norx {
            val |= DATA_HDR_NORX;
        }
        if dv {
            val |= DATA_HDR_DV;
            val |= u32::from(port) << DATA_HDR_VS_SHIFT;
        }
        if sv {
            val |= DATA_HDR_SV;
        }
        if ev {
            val |= DATA_HDR_EV;
            val |= (ebo as u32) << DATA_HDR_EBO_SHIFT;
        }
        with_parity(val)
    }
}

impl<SPI: SpiDevice> Tc6<SPI> {
    /// Exchange one full-duplex data chunk with the MAC-PHY.
    ///
    /// Sends `header` + `tx_payload`, receives the RX payload into
    /// `rx_payload` and returns the parsed footer. Updates the cached
    /// TXC/RCA/EXST values.
    async fn transfer_chunk(
        &mut self,
        header: u32,
        tx_payload: &[u8; CHUNK_PAYLOAD_SIZE],
        rx_payload: &mut [u8; CHUNK_PAYLOAD_SIZE],
    ) -> Result<Footer, AdinError<SPI::Error>> {
        let mut tx_buf = [0u8; CHUNK_SIZE];
        tx_buf[0..HDR_SIZE].copy_from_slice(&header.to_be_bytes());
        tx_buf[HDR_SIZE..].copy_from_slice(tx_payload);

        let mut rx_buf = [0u8; CHUNK_SIZE];

        self.spi
            .transaction(&mut [Operation::Transfer(&mut rx_buf, &tx_buf)])
            .await
            .map_err(AdinError::Spi)?;

        let raw = u32::from_be_bytes(rx_buf[CHUNK_PAYLOAD_SIZE..].try_into().unwrap());

        if raw == FOOTER_HDRB_ONLY {
            return Err(AdinError::SPI_TC6_HEADER_MISMATCH);
        }
        if !parity_ok(raw) {
            return Err(AdinError::SPI_CRC);
        }

        let footer = Footer(raw);
        if footer.hdrb() {
            return Err(AdinError::SPI_TC6_HEADER_MISMATCH);
        }
        if !footer.sync() {
            // The MAC-PHY lost its configuration (e.g. it was reset).
            return Err(AdinError::TC6_SYNC);
        }

        self.txc = footer.txc();
        self.rca = footer.rca();
        self.exst = footer.exst();

        rx_payload.copy_from_slice(&rx_buf[0..CHUNK_PAYLOAD_SIZE]);

        Ok(footer)
    }

    /// Exchange an empty data chunk (no TX data, RX inhibited) to refresh the
    /// cached TXC/RCA/EXST values from the footer.
    ///
    /// This is also the required host response to an `INT_N` assertion in OPEN
    /// Alliance mode.
    pub async fn poll_status(&mut self) -> Result<(), AdinError<SPI::Error>> {
        let header = Self::data_header(false, false, false, 0, 0, true);
        let tx_payload = [0u8; CHUNK_PAYLOAD_SIZE];
        let mut rx_payload = [0u8; CHUNK_PAYLOAD_SIZE];
        self.transfer_chunk(header, &tx_payload, &mut rx_payload).await?;
        Ok(())
    }

    /// Process the payload of one receive chunk according to its footer.
    ///
    /// Returns `Ok(Some(frame_len))` when a complete frame was copied into `out`.
    fn process_rx_chunk(
        &mut self,
        payload: &[u8; CHUNK_PAYLOAD_SIZE],
        footer: Footer,
        out: &mut [u8],
    ) -> Result<Option<usize>, AdinError<SPI::Error>> {
        if !footer.dv() {
            return Ok(None);
        }

        if footer.fd() {
            // Frame drop: discard the frame in progress.
            trace!("TC6 RX: frame drop");
            self.in_frame = false;
            self.rx_len = 0;
            return Ok(None);
        }

        let sbo = footer.swo() * 4;
        let ebo = footer.ebo();

        match (footer.sv(), footer.ev()) {
            (true, true) if ebo + 1 > sbo => {
                // A complete frame is contained in this single chunk.
                if self.in_frame {
                    trace!("TC6 RX: SV while in frame, dropping partial frame");
                }
                self.in_frame = true;
                self.rx_len = 0;
                self.append_rx(&payload[sbo..=ebo])?;
                self.finish_frame(out).map(Some)
            }
            (true, true) => {
                // End of the previous frame plus start of a new frame.
                let finished = if self.in_frame {
                    self.append_rx(&payload[0..=ebo])?;
                    Some(self.finish_frame(out))
                } else {
                    trace!("TC6 RX: EV without frame start, ignored");
                    None
                };
                self.in_frame = true;
                self.rx_len = 0;
                self.append_rx(&payload[sbo..])?;
                match finished {
                    Some(res) => res.map(Some),
                    None => Ok(None),
                }
            }
            (true, false) => {
                if self.in_frame {
                    trace!("TC6 RX: SV while in frame, dropping partial frame");
                }
                self.in_frame = true;
                self.rx_len = 0;
                self.append_rx(&payload[sbo..])?;
                Ok(None)
            }
            (false, true) => {
                if self.in_frame {
                    self.append_rx(&payload[0..=ebo])?;
                    self.finish_frame(out).map(Some)
                } else {
                    trace!("TC6 RX: EV without frame start, ignored");
                    Ok(None)
                }
            }
            (false, false) => {
                if self.in_frame {
                    self.append_rx(payload)?;
                }
                Ok(None)
            }
        }
    }

    fn append_rx(&mut self, data: &[u8]) -> Result<(), AdinError<SPI::Error>> {
        if self.rx_len + data.len() > self.rx_buf.len() {
            self.in_frame = false;
            self.rx_len = 0;
            return Err(AdinError::PACKET_TOO_BIG);
        }
        self.rx_buf[self.rx_len..self.rx_len + data.len()].copy_from_slice(data);
        self.rx_len += data.len();
        Ok(())
    }

    /// Validate the assembled frame, strip the FCS and copy it into `out`.
    fn finish_frame(&mut self, out: &mut [u8]) -> Result<usize, AdinError<SPI::Error>> {
        let total = self.rx_len;
        self.in_frame = false;
        self.rx_len = 0;

        // Frames arrive with the FCS appended.
        if total < ETH_MIN_LEN {
            return Err(AdinError::PACKET_TOO_SMALL);
        }
        let len = total - FCS_LEN;
        if len > out.len() {
            return Err(AdinError::PACKET_TOO_BIG);
        }

        let fcs_calc = ETH_FCS::new(&self.rx_buf[0..len]);
        if fcs_calc.hton_bytes() != self.rx_buf[len..total] {
            return Err(AdinError::FCS);
        }

        out[0..len].copy_from_slice(&self.rx_buf[0..len]);
        Ok(len)
    }

    /// Transmit one frame on one port, in chunks, respecting TX credits.
    ///
    /// The frame on the wire is `frame`, zero-padded to `pad_len` bytes,
    /// followed by `fcs` if the host appends the FCS.
    async fn send_frame_on_port(
        &mut self,
        frame: &[u8],
        pad_len: usize,
        fcs: Option<[u8; FCS_LEN]>,
        port: u8,
    ) -> Result<(), AdinError<SPI::Error>> {
        let fcs_bytes: &[u8] = fcs.as_ref().map_or(&[], |f| f.as_slice());
        let total = pad_len + fcs_bytes.len();

        let mut offset = 0;
        let mut first = true;
        let mut rx_payload = [0u8; CHUNK_PAYLOAD_SIZE];

        while offset < total {
            // Wait for a transmit credit.
            let mut polls = 0u32;
            while self.txc == 0 {
                self.poll_status().await?;
                if self.txc == 0 {
                    polls += 1;
                    if polls > POLL_LIMIT {
                        return Err(AdinError::TC6_TIMEOUT);
                    }
                    embassy_futures::yield_now().await;
                }
            }

            let n = CHUNK_PAYLOAD_SIZE.min(total - offset);
            let mut payload = [0u8; CHUNK_PAYLOAD_SIZE];
            for (i, byte) in payload[0..n].iter_mut().enumerate() {
                let pos = offset + i;
                *byte = if pos < frame.len() {
                    frame[pos]
                } else if pos < pad_len {
                    0
                } else {
                    fcs_bytes[pos - pad_len]
                };
            }

            let last = offset + n == total;
            // RX is inhibited (NORX) while transmitting so received frame data
            // stays in the MAC-PHY buffer instead of being discarded with the
            // ignored RX payload of these chunks.
            let header = Self::data_header(true, first, last, n - 1, port, true);
            self.transfer_chunk(header, &payload, &mut rx_payload).await?;

            first = false;
            offset += n;
        }

        Ok(())
    }
}

impl<SPI: SpiDevice> Adin1110Protocol for Tc6<SPI> {
    type SpiError = SPI::Error;

    async fn read_reg(&mut self, addr: u16) -> Result<u32, AdinError<Self::SpiError>> {
        let header = Self::control_header(false, addr);
        let header_bytes = header.to_be_bytes();

        // Transaction layout on the wire (3 words, 4 when protected):
        //   MOSI: header, dummy, dummy[, dummy]
        //   MISO: dummy, echoed header, register value[, !register value]
        let mut rx_buf = [0u8; 12];
        let rx = if self.protected { &mut rx_buf[..] } else { &mut rx_buf[..8] };
        let mut ops = [Operation::Write(&header_bytes), Operation::Read(rx)];
        self.spi.transaction(&mut ops).await.map_err(AdinError::Spi)?;

        let echoed = u32::from_be_bytes(rx_buf[0..4].try_into().unwrap());
        if echoed != header {
            return Err(AdinError::SPI_TC6_HEADER_MISMATCH);
        }

        let value = u32::from_be_bytes(rx_buf[4..8].try_into().unwrap());

        if self.protected {
            let complement = u32::from_be_bytes(rx_buf[8..12].try_into().unwrap());
            if complement != !value {
                return Err(AdinError::TC6_PROTECTION);
            }
        }

        trace!("TC6 REG Read {:04x} = {:08x}", addr, value);

        Ok(value)
    }

    async fn write_reg(&mut self, addr: u16, value: u32) -> Result<(), AdinError<Self::SpiError>> {
        let header = Self::control_header(true, addr);
        let header_bytes = header.to_be_bytes();
        let value_bytes = value.to_be_bytes();

        trace!("TC6 REG Write {:04x} = {:08x}", addr, value);

        // Transaction layout on the wire, one word longer when protected:
        //   MOSI: header, value[, !value], dummy
        //   MISO: dummy, echoed header, echoed value[, echoed !value]
        // The total length must match exactly what the MAC-PHY expects for the
        // configured format, a longer transaction is a framing error.
        let complement_bytes = (!value).to_be_bytes();
        let mut echo = [0u8; 4];
        let mut ignored = [0u8; 4];
        let mut protected_ops;
        let mut plain_ops;
        let ops: &mut [Operation<'_, u8>] = if self.protected {
            protected_ops = [
                Operation::Write(&header_bytes),
                Operation::Transfer(&mut echo, &value_bytes),
                Operation::Write(&complement_bytes),
                Operation::Read(&mut ignored),
            ];
            &mut protected_ops
        } else {
            plain_ops = [
                Operation::Write(&header_bytes),
                Operation::Transfer(&mut echo, &value_bytes),
                Operation::Read(&mut ignored),
            ];
            &mut plain_ops
        };
        self.spi.transaction(ops).await.map_err(AdinError::Spi)?;

        if u32::from_be_bytes(echo) != header {
            return Err(AdinError::SPI_TC6_HEADER_MISMATCH);
        }

        Ok(())
    }

    async fn read_fifo(&mut self, frame: &mut [u8]) -> Result<usize, AdinError<Self::SpiError>> {
        let mut polls = 0u32;
        loop {
            // Only clock out receive chunks the MAC-PHY has ready.
            while self.rca == 0 {
                self.poll_status().await?;
                if self.rca == 0 {
                    polls += 1;
                    if polls > POLL_LIMIT {
                        return Err(AdinError::TC6_TIMEOUT);
                    }
                    embassy_futures::yield_now().await;
                }
            }

            let header = Self::data_header(false, false, false, 0, 0, false);
            let tx_payload = [0u8; CHUNK_PAYLOAD_SIZE];
            let mut rx_payload = [0u8; CHUNK_PAYLOAD_SIZE];
            let footer = self.transfer_chunk(header, &tx_payload, &mut rx_payload).await?;

            if let Some(len) = self.process_rx_chunk(&rx_payload, footer, frame)? {
                return Ok(len);
            }
        }
    }

    async fn write_fifo(&mut self, frame: &[u8]) -> Result<(), AdinError<Self::SpiError>> {
        // Ethernet header: 6 bytes dst + 6 bytes src + 2 bytes type/len.
        if frame.len() < (6 + 6 + 2) {
            return Err(AdinError::PACKET_TOO_SMALL);
        }
        if frame.len() > MTU {
            return Err(AdinError::PACKET_TOO_BIG);
        }

        // The MAC does not pad short frames; pad to the minimum frame size,
        // FCS excluded.
        let pad_len = frame.len().max(ETH_MIN_WITHOUT_FCS_LEN);

        let fcs = if self.append_fcs_on_tx {
            let mut fcs = ETH_FCS::new(frame);
            if pad_len > frame.len() {
                fcs = fcs.update(&[0u8; ETH_MIN_WITHOUT_FCS_LEN][0..pad_len - frame.len()]);
            }
            Some(fcs.hton_bytes())
        } else {
            None
        };

        match self.tx_port {
            TxPort::Port1 => self.send_frame_on_port(frame, pad_len, fcs, 0).await,
            TxPort::Port2 => self.send_frame_on_port(frame, pad_len, fcs, 1).await,
            TxPort::Flood => {
                self.send_frame_on_port(frame, pad_len, fcs, 0).await?;
                self.send_frame_on_port(frame, pad_len, fcs, 1).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use embedded_hal_1::digital::{ErrorType, OutputPin};
    use embedded_hal_async::delay::DelayNs;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    use super::*;

    #[derive(Debug, Default)]
    struct CsPinMock;
    impl OutputPin for CsPinMock {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    impl ErrorType for CsPinMock {
        type Error = Infallible;
    }

    struct MockDelay {}
    impl DelayNs for MockDelay {
        async fn delay_ns(&mut self, _ns: u32) {}
    }

    type MockTc6 = Tc6<ExclusiveDevice<embedded_hal_mock::common::Generic<SpiTransaction<u8>>, CsPinMock, MockDelay>>;

    fn harness(
        expectations: &[SpiTransaction<u8>],
    ) -> (MockTc6, embedded_hal_mock::common::Generic<SpiTransaction<u8>>) {
        let spi = SpiMock::new(expectations);
        let spi_dev = ExclusiveDevice::new(spi.clone(), CsPinMock, MockDelay {});
        (Tc6::new(spi_dev, false, TxPort::Port1), spi)
    }

    /// The chunk footer for the given flag bits, with valid parity.
    fn footer(bits: u32) -> u32 {
        with_parity(bits)
    }

    const FTR_SYNC: u32 = 1 << 29;
    const FTR_DV: u32 = 1 << 21;
    const FTR_SV: u32 = 1 << 20;
    const FTR_EV: u32 = 1 << 14;

    #[futures_test::test]
    async fn control_read_transaction() {
        // Read PHYID (register 0x01, MMS 0):
        // MOSI: header 0x00000100, then 8 dummy bytes.
        // MISO: 4 dummy bytes, echoed header, register value.
        let header = [0x00, 0x00, 0x01, 0x00];
        let expectations = [
            SpiTransaction::write_vec(header.to_vec()),
            SpiTransaction::read_vec(vec![0x00, 0x00, 0x01, 0x00, 0x02, 0x83, 0xBC, 0xA1]),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);

        let val = tc6.read_reg(0x01).await.expect("read_reg");
        assert_eq!(val, 0x0283_BCA1);
        spi.done();
    }

    #[futures_test::test]
    async fn control_read_echo_mismatch() {
        let header = [0x00, 0x00, 0x01, 0x00];
        let expectations = [
            SpiTransaction::write_vec(header.to_vec()),
            // Chip echoes all zeros (still starting up).
            SpiTransaction::read_vec(vec![0x00; 8]),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);

        assert!(matches!(
            tc6.read_reg(0x01).await,
            Err(AdinError::SPI_TC6_HEADER_MISMATCH)
        ));
        spi.done();
    }

    #[futures_test::test]
    async fn control_write_transaction() {
        // Write CONFIG0 (register 0x04, MMS 0) = 0x0000_8006:
        // MOSI: header 0x20000401, value, 4 dummy bytes.
        // MISO: 4 dummy bytes, echoed header, echoed value.
        let header = [0x20, 0x00, 0x04, 0x01];
        let value = [0x00, 0x00, 0x80, 0x06];
        let expectations = [
            SpiTransaction::write_vec(header.to_vec()),
            SpiTransaction::transfer(value.to_vec(), header.to_vec()),
            SpiTransaction::read_vec(vec![0x00; 4]),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);

        tc6.write_reg(0x04, 0x0000_8006).await.expect("write_reg");
        spi.done();
    }

    #[futures_test::test]
    async fn control_write_mms1() {
        // Write to ADDR_FILT_UPR0 (0x50): a vendor register in MMS 1.
        // Header: WNR | MMS=1 | ADDR 0x0050 -> 0x2100_5000 | parity.
        let header = with_parity(0x2100_5000).to_be_bytes();
        let value = [0x12, 0x34, 0x56, 0x78];
        let expectations = [
            SpiTransaction::write_vec(header.to_vec()),
            SpiTransaction::transfer(value.to_vec(), header.to_vec()),
            SpiTransaction::read_vec(vec![0x00; 4]),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);

        tc6.write_reg(0x50, 0x1234_5678).await.expect("write_reg");
        spi.done();
    }

    /// Build the expected MISO bytes of one chunk: 64 payload bytes + footer.
    fn rx_chunk(payload: &[u8], ftr: u32) -> Vec<u8> {
        let mut buf = vec![0u8; CHUNK_PAYLOAD_SIZE];
        buf[0..payload.len()].copy_from_slice(payload);
        buf.extend_from_slice(&ftr.to_be_bytes());
        buf
    }

    /// Build the expected MOSI bytes of one chunk: header + 64 payload bytes.
    fn tx_chunk(header: u32, payload: &[u8]) -> Vec<u8> {
        let mut buf = header.to_be_bytes().to_vec();
        buf.extend_from_slice(payload);
        buf.resize(CHUNK_SIZE, 0);
        buf
    }

    #[futures_test::test]
    async fn receive_single_chunk_frame() {
        // A 60-byte frame + FCS fits exactly in one 64-byte chunk.
        let mut frame = [0u8; 60];
        for (i, b) in frame.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            {
                *b = i as u8;
            }
        }
        let fcs = ETH_FCS::new(&frame).hton_bytes();
        let mut wire = frame.to_vec();
        wire.extend_from_slice(&fcs);

        // Data chunk with RX enabled: DNC only.
        let rd_header = Tc6::<()>::data_header(false, false, false, 0, 0, false);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(rd_header, &[]),
                rx_chunk(&wire, footer(FTR_SYNC | FTR_DV | FTR_SV | FTR_EV | (63 << 8))),
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);
        tc6.rca = 1;

        let mut out = [0u8; MTU];
        let n = tc6.read_fifo(&mut out).await.expect("read_fifo");
        assert_eq!(n, 60);
        assert_eq!(&out[0..n], &frame[..]);
        spi.done();
    }

    #[futures_test::test]
    async fn receive_multi_chunk_frame() {
        // A 96-byte frame + FCS spans two chunks: 64 + 36 bytes.
        let mut frame = [0u8; 96];
        for (i, b) in frame.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            {
                *b = (i * 7) as u8;
            }
        }
        let fcs = ETH_FCS::new(&frame).hton_bytes();
        let mut wire = frame.to_vec();
        wire.extend_from_slice(&fcs);

        let rd_header = Tc6::<()>::data_header(false, false, false, 0, 0, false);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(rd_header, &[]),
                rx_chunk(&wire[0..64], footer(FTR_SYNC | FTR_DV | FTR_SV | (1 << 24))),
            ),
            SpiTransaction::flush(),
            SpiTransaction::transfer(
                tx_chunk(rd_header, &[]),
                rx_chunk(&wire[64..100], footer(FTR_SYNC | FTR_DV | FTR_EV | (35 << 8))),
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);
        tc6.rca = 2;

        let mut out = [0u8; MTU];
        let n = tc6.read_fifo(&mut out).await.expect("read_fifo");
        assert_eq!(n, 96);
        assert_eq!(&out[0..n], &frame[..]);
        spi.done();
    }

    #[futures_test::test]
    async fn receive_bad_fcs() {
        let frame = [0xAAu8; 60];
        let mut wire = frame.to_vec();
        wire.extend_from_slice(&[0, 1, 2, 3]); // wrong FCS

        let rd_header = Tc6::<()>::data_header(false, false, false, 0, 0, false);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(rd_header, &[]),
                rx_chunk(&wire, footer(FTR_SYNC | FTR_DV | FTR_SV | FTR_EV | (63 << 8))),
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);
        tc6.rca = 1;

        let mut out = [0u8; MTU];
        assert!(matches!(tc6.read_fifo(&mut out).await, Err(AdinError::FCS)));
        spi.done();
    }

    #[futures_test::test]
    async fn transmit_short_frame_is_padded() {
        // A minimal 14-byte frame is zero-padded to 60 bytes and fits in one
        // chunk: DNC | NORX | DV | SV | EV | EBO=59.
        let frame = [0x11u8; 14];
        let mut payload = [0u8; 60];
        payload[0..14].copy_from_slice(&frame);

        let wr_header = Tc6::<()>::data_header(true, true, true, 59, 0, true);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(wr_header, &payload),
                rx_chunk(&[], footer(FTR_SYNC | (10 << 1))),
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);
        tc6.txc = 31;

        tc6.write_fifo(&frame).await.expect("write_fifo");
        assert_eq!(tc6.txc, 10);
        spi.done();
    }

    #[futures_test::test]
    async fn transmit_multi_chunk_frame() {
        // A 100-byte frame spans two chunks: 64 bytes + 36 bytes.
        let mut frame = [0u8; 100];
        for (i, b) in frame.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            {
                *b = (255 - i) as u8;
            }
        }

        let hdr1 = Tc6::<()>::data_header(true, true, false, 0, 0, true);
        let hdr2 = Tc6::<()>::data_header(true, false, true, 35, 0, true);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(hdr1, &frame[0..64]),
                rx_chunk(&[], footer(FTR_SYNC | (30 << 1))),
            ),
            SpiTransaction::flush(),
            SpiTransaction::transfer(
                tx_chunk(hdr2, &frame[64..100]),
                rx_chunk(&[], footer(FTR_SYNC | (29 << 1))),
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);
        tc6.txc = 31;

        tc6.write_fifo(&frame).await.expect("write_fifo");
        spi.done();
    }

    #[futures_test::test]
    async fn footer_sync_lost() {
        let rd_header = Tc6::<()>::data_header(false, false, false, 0, 0, true);
        let expectations = [
            SpiTransaction::transfer(
                tx_chunk(rd_header, &[]),
                rx_chunk(&[], footer(0)), // SYNC bit clear
            ),
            SpiTransaction::flush(),
        ];
        let (mut tc6, mut spi) = harness(&expectations);

        assert!(matches!(tc6.poll_status().await, Err(AdinError::TC6_SYNC)));
        spi.done();
    }

    #[test]
    fn parity() {
        // A control read of register 0 with LEN=0: all zeros except P.
        assert_eq!(with_parity(0), 0x0000_0001);
        // One bit set: P stays 0.
        assert_eq!(with_parity(1 << 29), 1 << 29);
        // Two bits set: P becomes 1.
        assert_eq!(with_parity((1 << 29) | (1 << 21)), (1 << 29) | (1 << 21) | 1);

        assert!(parity_ok(0x0000_0001));
        assert!(!parity_ok(0x0000_0003));
        // The header-bad footer passes the parity check on its own; the code
        // must check for the exact value before checking parity.
        assert!(parity_ok(FOOTER_HDRB_ONLY));
    }

    #[test]
    fn control_headers() {
        // Read of PHYID (MMS0 register 0x01):
        // DNC=0, WNR=0, MMS=0, ADDR=0x0001 -> one bit set, P stays 0.
        assert_eq!(Tc6::<()>::control_header(false, 0x01), 0x0000_0100);

        // Write to CONFIG0 (MMS0 register 0x04):
        // WNR (bit 29) | ADDR 0x0004 << 8 = 0x2000_0400, two bits -> P=1.
        assert_eq!(Tc6::<()>::control_header(true, 0x04), 0x2000_0401);

        // TX_FSIZE (0x30) is the first MMS1 register.
        let hdr = Tc6::<()>::control_header(true, 0x30);
        assert_eq!(hdr & (0xF << CTRL_HDR_MMS_SHIFT), 1 << CTRL_HDR_MMS_SHIFT);
        // ADDR_FILT_UPR0 (0x50) is also MMS1.
        let hdr = Tc6::<()>::control_header(true, 0x50);
        assert_eq!(hdr & (0xF << CTRL_HDR_MMS_SHIFT), 1 << CTRL_HDR_MMS_SHIFT);
        // STATUS0 (0x08) is MMS0.
        let hdr = Tc6::<()>::control_header(false, 0x08);
        assert_eq!(hdr & (0xF << CTRL_HDR_MMS_SHIFT), 0);
    }

    #[test]
    fn data_headers() {
        // Poll chunk: DNC | NORX, two bits set -> P=1.
        assert_eq!(
            Tc6::<()>::data_header(false, false, false, 0, 0, true),
            DATA_HDR_DNC | DATA_HDR_NORX | 1
        );

        // Single-chunk frame of 64 bytes on port 2:
        // DNC | NORX | VS=1 | DV | SV | EV | EBO=63.
        let hdr = Tc6::<()>::data_header(true, true, true, 63, 1, true);
        assert_eq!(
            hdr & !1,
            DATA_HDR_DNC
                | DATA_HDR_NORX
                | (1 << DATA_HDR_VS_SHIFT)
                | DATA_HDR_DV
                | DATA_HDR_SV
                | DATA_HDR_EV
                | (63 << DATA_HDR_EBO_SHIFT)
        );
        assert!(parity_ok(hdr));
    }

    #[test]
    fn footer_fields() {
        // EXST | SYNC | RCA=3 | DV | SV | SWO=2 | EV | EBO=17 | TXC=12
        let raw: u32 =
            (1 << 31) | (1 << 29) | (3 << 24) | (1 << 21) | (1 << 20) | (2 << 16) | (1 << 14) | (17 << 8) | (12 << 1);
        let f = Footer(raw);
        assert!(f.exst());
        assert!(!f.hdrb());
        assert!(f.sync());
        assert_eq!(f.rca(), 3);
        assert!(f.dv());
        assert!(f.sv());
        assert_eq!(f.swo(), 2);
        assert!(!f.fd());
        assert!(f.ev());
        assert_eq!(f.ebo(), 17);
        assert_eq!(f.txc(), 12);
    }
}
