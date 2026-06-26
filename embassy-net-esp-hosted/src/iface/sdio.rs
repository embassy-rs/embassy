//! SDIO interface.
//!
//! Speaks the esp-hosted SDIO transport protocol on top of the [`sdio`] crate's
//! [`SdioCard`]. Works with both `esp-hosted-mcu` and `esp-hosted-fg`.
//!
//! # Packet mode only
//!
//! The co-processor must run in *packet mode*, where each new-packet interrupt
//! maps to a single packet that fits in one transfer buffer. `esp-hosted-fg` is
//! always packet mode. For `esp-hosted-mcu`, SDIO *streaming mode* (the default)
//! concatenates packets into one large transfer and is **not** supported here:
//! disable `Enable SDIO Streaming Mode` on the co-processor.
//!
//! The co-processor must also keep the SDIO payload checksum enabled (the
//! default), as the transport layer always uses it.
//!
//! # Host requirements
//!
//! The [`MmcBus`] HAL should implement [`MmcBus::wait_for_event`] to block until
//! the data-ready (DAT1) interrupt is asserted, otherwise the runner busy-polls.

use ::sdio::sdio::SdioCard;
use ::sdio::{MmcBus, MmcError};
use aligned::{A4, Aligned};
use embassy_time::{Delay, Timer};
use embedded_hal_async::delay::DelayNs;

use crate::Interface;

const SDIO_FUNC_1: u8 = 1;
const FUNC1_MASK: u8 = 1 << 1;

/// Host-to-co-processor interrupt bit that opens the data path.
const ESP_OPEN_DATA_PATH: u8 = 0;

// Co-processor registers, masked with `ESP_ADDRESS_MASK` (0x3FF).
const REG_INT_RAW: u32 = 0x050;
const REG_TOKEN_RDATA: u32 = 0x044;
const REG_PACKET_LEN: u32 = 0x060;
const REG_INT_CLR: u32 = 0x0D4;
const REG_HOST_TO_SLAVE_INTR: u32 = 0x08C; // scratch register 7

const NEW_PACKET_INT: u32 = 1 << 23;

/// CMD53 data window end; data is transferred to/from `ESP_DATA_END_ADDR - len`.
const ESP_DATA_END_ADDR: u32 = 0x1F800;
const ESP_BLOCK_SIZE: usize = 512;

const ESP_RX_BYTE_MAX: u32 = 0x10_0000;
const ESP_LEN_MASK: u32 = 0x0F_FFFF;
const ESP_TX_BUFFER_MAX: u32 = 0x1000;
const ESP_TX_BUFFER_MASK: u32 = 0x0FFF;
/// Co-processor RX buffer size; one packet uses `ceil(len / this)` buffers.
const ESP_RX_BUFFER_SIZE: usize = 1536;

const MAX_WRITE_BUF_RETRIES: u8 = 50;
const HEADER_LEN: usize = 12;

/// SDIO interface for esp-hosted.
///
/// Wraps an [`SdioCard`]. The card is (re)acquired by [`Interface::init`] after
/// the runner pulses reset, so pass an un-acquired card from [`SdioCard::new_uninit`].
pub struct SdioInterface<B: MmcBus, D: DelayNs> {
    card: SdioCard<B, D>,
    freq: u32,
    tx_buf_count: u32,
    rx_byte_count: u32,
    /// Last buffer-grant count read from `TOKEN_RDATA`. Cached so TX doesn't read
    /// the register on every packet; refreshed only when credits run out.
    cached_granted: u32,
}

impl<B: MmcBus, D: DelayNs> SdioInterface<B, D> {
    /// Create a new SDIO interface targeting `freq_hz` (clamped during acquisition).
    pub fn new(card: SdioCard<B, D>, freq_hz: u32) -> Self {
        Self {
            card,
            freq: freq_hz,
            tx_buf_count: 0,
            rx_byte_count: 0,
            cached_granted: 0,
        }
    }

    async fn try_init(&mut self) -> Result<(), MmcError> {
        self.card
            .reacquire(self.freq)
            .await
            .inspect_err(|e| warn!("sdio: reacquire failed: {:?}", e))?;

        self.tx_buf_count = 0;
        self.rx_byte_count = 0;
        self.cached_granted = 0;

        let mut delay = Delay;
        self.card.enable_functions(FUNC1_MASK, &mut delay).await?;
        self.card.set_block_size(0, ESP_BLOCK_SIZE as u16).await?;
        self.card.set_block_size(1, ESP_BLOCK_SIZE as u16).await?;
        self.card.enable_interrupts(FUNC1_MASK).await?;
        self.card
            .cmd52_write(SDIO_FUNC_1, REG_HOST_TO_SLAVE_INTR, 1 << ESP_OPEN_DATA_PATH)
            .await?;

        Ok(())
    }

    async fn read_reg32(&mut self, addr: u32) -> Result<u32, MmcError> {
        let mut buf: Aligned<A4, [u8; 4]> = Aligned([0; 4]);
        self.card.cmd53_read_bytes(SDIO_FUNC_1, true, addr, &mut buf).await?;
        Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
    }

    async fn write_reg32(&mut self, addr: u32, val: u32) -> Result<(), MmcError> {
        let buf: Aligned<A4, [u8; 4]> = Aligned(val.to_le_bytes());
        self.card.cmd53_write_bytes(SDIO_FUNC_1, true, addr, &buf).await?;
        Ok(())
    }

    /// Read `INT_RAW` and `PACKET_LEN` in a single CMD53, returning
    /// `(interrupts, raw_packet_len)`.
    async fn read_status(&mut self) -> Result<(u32, u32), MmcError> {
        /// Length of the combined status read (`INT_RAW`..=`PACKET_LEN`), in bytes.
        const REG_STATUS_LEN: usize = (REG_PACKET_LEN - REG_INT_RAW + 4) as usize;
        /// Offset of `PACKET_LEN` within the combined status read.
        const PACKET_LEN_OFFSET: usize = (REG_PACKET_LEN - REG_INT_RAW) as usize;

        let mut buf = Aligned([0; REG_STATUS_LEN]);
        self.card
            .cmd53_read_bytes(SDIO_FUNC_1, true, REG_INT_RAW, &mut buf)
            .await?;
        let interrupts = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let packet_len = u32::from_le_bytes([
            buf[PACKET_LEN_OFFSET],
            buf[PACKET_LEN_OFFSET + 1],
            buf[PACKET_LEN_OFFSET + 2],
            buf[PACKET_LEN_OFFSET + 3],
        ]);
        Ok((interrupts, packet_len))
    }

    /// Read `len` bytes via a single block-only CMD53. The transfer is rounded up
    /// to whole blocks (the co-processor zero-pads the tail); the address encodes
    /// the real `len`, so the co-processor's FIFO still advances by `len`.
    async fn read_data(&mut self, buffer: &mut Aligned<A4, [u8]>, len: usize) -> Result<(), MmcError> {
        let blocks_n = len.div_ceil(ESP_BLOCK_SIZE);
        // SAFETY: `buffer` is A4-aligned and `blocks_n * ESP_BLOCK_SIZE <= buffer.len()`
        // (the caller guards against the padded length overflowing the buffer).
        let blocks: &mut [Aligned<A4, [u8; ESP_BLOCK_SIZE]>] = unsafe {
            core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut Aligned<A4, [u8; ESP_BLOCK_SIZE]>, blocks_n)
        };
        self.card
            .cmd53_read_blocks::<ESP_BLOCK_SIZE>(SDIO_FUNC_1, true, ESP_DATA_END_ADDR - len as u32, blocks)
            .await?;
        Ok(())
    }

    /// Write `len` bytes via a single block-only CMD53. The tail up to the block
    /// boundary is sent from `buffer` and discarded by the co-processor.
    async fn write_data(&mut self, buffer: &Aligned<A4, [u8]>, len: usize) -> Result<(), MmcError> {
        let blocks_n = len.div_ceil(ESP_BLOCK_SIZE);
        // SAFETY: see `read_data`.
        let blocks: &[Aligned<A4, [u8; ESP_BLOCK_SIZE]>] = unsafe {
            core::slice::from_raw_parts(buffer.as_ptr() as *const Aligned<A4, [u8; ESP_BLOCK_SIZE]>, blocks_n)
        };
        self.card
            .cmd53_write_blocks::<ESP_BLOCK_SIZE>(SDIO_FUNC_1, true, ESP_DATA_END_ADDR - len as u32, blocks)
            .await?;
        Ok(())
    }

    async fn write_packet(&mut self, buffer: &Aligned<A4, [u8]>, tx_len: usize) -> Result<(), MmcError> {
        let buf_needed = tx_len.div_ceil(ESP_RX_BUFFER_SIZE) as u32;

        let mut retries = MAX_WRITE_BUF_RETRIES;
        loop {
            // Common path: spend a cached credit without touching the bus.
            if self.cached_granted.wrapping_sub(self.tx_buf_count) & ESP_TX_BUFFER_MASK >= buf_needed {
                break;
            }
            // Out of cached credits: refresh from the co-processor.
            let token = self.read_reg32(REG_TOKEN_RDATA).await?;
            self.cached_granted = (token >> 16) & ESP_TX_BUFFER_MASK;
            if self.cached_granted.wrapping_sub(self.tx_buf_count) & ESP_TX_BUFFER_MASK >= buf_needed {
                break;
            }
            retries -= 1;
            if retries == 0 {
                warn!("sdio: no write buffers on co-processor, dropping packet");
                return Ok(());
            }
            Timer::after_millis(1).await;
        }

        self.write_data(buffer, tx_len).await?;
        self.tx_buf_count = (self.tx_buf_count + buf_needed) % ESP_TX_BUFFER_MAX;

        Ok(())
    }

    async fn read_packet(&mut self, buffer: &mut Aligned<A4, [u8]>) -> Result<usize, MmcError> {
        let (interrupts, raw) = self.read_status().await?;

        if interrupts & NEW_PACKET_INT == 0 {
            return Ok(0);
        }
        self.write_reg32(REG_INT_CLR, interrupts).await?;

        let raw = raw & ESP_LEN_MASK;
        let len = (raw.wrapping_sub(self.rx_byte_count) & ESP_LEN_MASK) as usize;

        if len == 0 {
            return Ok(0);
        }
        // The block-only read rounds up to a whole block, so guard the padded length.
        if len.div_ceil(ESP_BLOCK_SIZE) * ESP_BLOCK_SIZE > buffer.len() {
            warn!("sdio: rx packet length {} exceeds buffer", len);
            return Ok(0);
        }

        self.read_data(buffer, len).await?;
        self.rx_byte_count = (self.rx_byte_count + len as u32) % ESP_RX_BYTE_MAX;

        Ok(len)
    }
}

impl<B: MmcBus, D: DelayNs> Interface for SdioInterface<B, D> {
    async fn init(&mut self, cold_boot: bool) {
        if !cold_boot {
            Timer::after_secs(2).await;
        }

        loop {
            match self.try_init().await {
                Ok(()) => {
                    info!("sdio: co-processor ready");
                    return;
                }
                Err(e) => {
                    warn!("sdio: init failed, retrying: {:?}", e);
                    Timer::after_millis(100).await;
                }
            }
        }
    }

    async fn wait_for_handshake(&mut self) {}

    async fn wait_for_ready(&mut self) {
        if let Err(e) = self.card.wait_for_event().await {
            warn!("sdio: wait_for_event failed: {:?}", e);
        }
    }

    async fn transfer(&mut self, buffer: &mut Aligned<A4, [u8]>, tx_len: usize) {
        if tx_len > 0
            && let Err(e) = self.write_packet(buffer, tx_len).await
        {
            warn!("sdio: tx failed: {:?}", e);
        }

        let received = match self.read_packet(buffer).await {
            Ok(n) => n,
            Err(e) => {
                warn!("sdio: rx failed: {:?}", e);
                0
            }
        };

        if received == 0 {
            buffer[..HEADER_LEN].fill(0);
        }
    }
}
