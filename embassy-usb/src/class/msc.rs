//! USB Mass storage class
//! - Async BlockDevice trait
//! - Configurable INQUIRY strings (vendor/product/revision)
//! - Fixed 512-byte sector size
//! - Bulk-Only Transport (BOT) + minimal SCSI (INQUIRY, TEST UNIT READY,
//! REQUEST SENSE, READ CAPACITY(10), READ(10), WRITE(10), MODE SENSE(6))

use core::cmp::min;
use core::mem::size_of;

use embedded_hal_async::delay::DelayNs;

use crate::driver::{Driver, EndpointError, EndpointIn, EndpointOut};
use crate::Builder;

/// only 512 byte block size is supported
pub const BLOCK_SIZE: usize = 512;

const CBW_SIGNATURE: u32 = 0x4342_5355; // 'USBC'
const CSW_SIGNATURE: u32 = 0x5342_5355; // 'USBS'

const CSW_STATUS_PASSED: u8 = 0;
const CSW_STATUS_FAILED: u8 = 1;
const CSW_STATUS_PHASE: u8 = 2;

/// USB Mass Storage Class codes
pub const USB_CLASS_MSC: u8 = 0x08;
/// USB Mass Storage subclass
const MSC_SUBCLASS_SCSI: u8 = 0x06;
/// USB Mass Storage bulk-only
const MSC_PROTOCOL_BULK_ONLY: u8 = 0x50;

pub trait BlockDevice {
    /// Number of blocks (each of size `BLOCK_SIZE`).
    fn num_blocks(&self) -> u32;
    /// Read one block into `buf` (exactly `BLOCK_SIZE` bytes).
    async fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]);
    /// Write one block from `buf` (exactly `BLOCK_SIZE` bytes).
    async fn write_block(&mut self, lba: u32, buf: &[u8; BLOCK_SIZE]) -> Result<(), ()>;
    /// Optional: indicate if medium is ready.
    fn ready(&self) -> bool {
        true
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct CommandBlockWrapper {
    d_cbw_signature: u32,
    d_cbw_tag: u32,
    d_cbw_data_transfer_length: u32,
    bm_cbw_flags: u8,
    b_cbw_lun: u8,
    b_cbw_cb_length: u8,
    cbw_cb: [u8; 16],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct CommandStatusWrapper {
    d_csw_signature: u32,
    d_csw_tag: u32,
    d_csw_data_residue: u32,
    b_csw_status: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DataDir {
    In,
    Out,
}

#[derive(Clone, Copy)]
struct ScsiSense {
    key: u8,
    asc: u8,
    ascq: u8,
}
impl Default for ScsiSense {
    fn default() -> Self {
        Self {
            key: 0,
            asc: 0,
            ascq: 0,
        }
    }
}

/// Mass Storage class implementation parameterized over the driver `D` and block device `BD`.
pub struct MassStorage<'d, D: Driver<'d>, BD: BlockDevice> {
    ep_in: D::EndpointIn,
    ep_out: D::EndpointOut,
    bd: BD,
    vendor: &'static [u8; 8],
    product: &'static [u8; 16],
    revision: &'static [u8; 4],
    sense: ScsiSense,
}

impl<'d, D: Driver<'d>, BD: BlockDevice> MassStorage<'d, D, BD> {
    /// Construct the MSC class from builder
    pub fn new(
        builder: &mut Builder<'d, D>,
        bd: BD,
        vendor: &'static [u8; 8],
        product: &'static [u8; 16],
        revision: &'static [u8; 4],
    ) -> Self {
        // Create the function for MSC
        let mut func = builder.function(USB_CLASS_MSC, MSC_SUBCLASS_SCSI, MSC_PROTOCOL_BULK_ONLY);

        // Create interface and alternate setting
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(USB_CLASS_MSC, MSC_SUBCLASS_SCSI, MSC_PROTOCOL_BULK_ONLY, None);

        // Allocate bulk endpoints
        let ep_in = alt.endpoint_bulk_in(None, 64);
        let ep_out = alt.endpoint_bulk_out(None, 64);

        Self {
            ep_in,
            ep_out,
            bd,
            vendor,
            product,
            revision,
            sense: ScsiSense::default(),
        }
    }

    /// Run the BOT state machine forever.
    pub async fn run<DL: DelayNs>(&mut self, delay: &mut DL) -> ! {
        let mut cbw_buf = [0u8; size_of::<CommandBlockWrapper>()];
        loop {
            if self.read_exact_out(&mut cbw_buf).await.is_err() {
                let _ = delay.delay_ms(5).await;
                continue;
            }
            let cbw = parse_cbw(&cbw_buf);
            if cbw.d_cbw_signature != CBW_SIGNATURE {
                // No `stall()` method on endpoints in the current driver API. If your driver
                // provides a separate way to stall endpoints by id, call it here.
                continue;
            }

            let dir = if cbw.bm_cbw_flags & 0x80 != 0 {
                DataDir::In
            } else {
                DataDir::Out
            };
            let exp = cbw.d_cbw_data_transfer_length;

            let (actual, status) = self.scsi_execute(&cbw, dir, exp).await;

            let csw = CommandStatusWrapper {
                d_csw_signature: CSW_SIGNATURE,
                d_csw_tag: cbw.d_cbw_tag,
                d_csw_data_residue: exp.saturating_sub(actual),
                b_csw_status: status,
            };
            let mut csw_buf = [0u8; 13];
            write_csw(&csw, &mut csw_buf);
            let _ = self.write_in(&csw_buf).await;
        }
    }

    async fn read_exact_out(&mut self, buf: &mut [u8]) -> Result<(), EndpointError> {
        let mut read = 0usize;
        while read < buf.len() {
            let n = self.ep_out.read(&mut buf[read..]).await?;
            read += n;
        }
        Ok(())
    }

    async fn write_in(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.ep_in.write(buf).await?;
        Ok(())
    }

    async fn scsi_execute(&mut self, cbw: &CommandBlockWrapper, dir: DataDir, exp: u32) -> (u32, u8) {
        let opcode = cbw.cbw_cb[0];
        match opcode {
            0x12 => {
                // INQUIRY
                let mut buf = [0u8; 36];
                buf[0] = 0x00; // Direct-access
                buf[2] = 0x06;
                buf[3] = 0x02;
                buf[4] = 31;
                buf[8..16].copy_from_slice(self.vendor);
                buf[16..32].copy_from_slice(self.product);
                buf[32..36].copy_from_slice(self.revision);
                let xfer = min(exp, buf.len() as u32) as usize;
                let _ = self.write_in(&buf[..xfer]).await;
                (xfer as u32, CSW_STATUS_PASSED)
            }
            0x00 => {
                // TEST UNIT READY
                if self.bd.ready() {
                    (0, CSW_STATUS_PASSED)
                } else {
                    self.set_sense(0x02, 0x3A, 0x00);
                    (0, CSW_STATUS_FAILED)
                }
            }
            0x03 => {
                // REQUEST SENSE
                let mut buf = [0u8; 18];
                buf[0] = 0x70;
                buf[2] = self.sense.key;
                buf[7] = 10;
                buf[12] = self.sense.asc;
                buf[13] = self.sense.ascq;
                let xfer = min(exp, buf.len() as u32) as usize;
                let _ = self.write_in(&buf[..xfer]).await;
                (xfer as u32, CSW_STATUS_PASSED)
            }
            0x1A => {
                // MODE SENSE (6)
                let mut buf = [0u8; 4];
                buf[0] = 3;
                let xfer = min(exp, buf.len() as u32) as usize;
                let _ = self.write_in(&buf[..xfer]).await;
                (xfer as u32, CSW_STATUS_PASSED)
            }
            0x25 => {
                // READ CAPACITY (10)
                let blocks = self.bd.num_blocks();
                if blocks == 0 {
                    self.set_sense(0x02, 0x3A, 0x00);
                    return (0, CSW_STATUS_FAILED);
                }
                let last_lba = blocks - 1;
                let block_len = BLOCK_SIZE as u32;
                let mut buf = [0u8; 8];
                buf[0..4].copy_from_slice(&last_lba.to_be_bytes());
                buf[4..8].copy_from_slice(&block_len.to_be_bytes());
                let _ = self.write_in(&buf).await;
                (8, CSW_STATUS_PASSED)
            }
            0x28 => {
                // READ (10)
                if dir != DataDir::In {
                    self.set_sense(0x05, 0x20, 0x00);
                    return (0, CSW_STATUS_PHASE);
                }
                let lba = u32::from_be_bytes([cbw.cbw_cb[2], cbw.cbw_cb[3], cbw.cbw_cb[4], cbw.cbw_cb[5]]);
                let blocks = u16::from_be_bytes([cbw.cbw_cb[7], cbw.cbw_cb[8]]) as u32;
                let mut sent = 0u32;
                let mut block = [0u8; BLOCK_SIZE];
                for i in 0..blocks {
                    self.bd.read_block(lba + i, &mut block).await;
                    if self.write_in(&block).await.is_err() {
                        break;
                    }
                    sent = sent.saturating_add(BLOCK_SIZE as u32);
                }
                (min(sent, exp), CSW_STATUS_PASSED)
            }
            0x2A => {
                // WRITE (10)
                if dir != DataDir::Out {
                    self.set_sense(0x05, 0x20, 0x00);
                    return (0, CSW_STATUS_PHASE);
                }
                let lba = u32::from_be_bytes([cbw.cbw_cb[2], cbw.cbw_cb[3], cbw.cbw_cb[4], cbw.cbw_cb[5]]);
                let blocks = u16::from_be_bytes([cbw.cbw_cb[7], cbw.cbw_cb[8]]) as u32;
                let mut recv = 0u32;
                let mut block = [0u8; BLOCK_SIZE];
                for i in 0..blocks {
                    if self.read_exact_out(&mut block).await.is_err() {
                        break;
                    }
                    if self.bd.write_block(lba + i, &block).await.is_err() {
                        self.set_sense(0x03, 0x0C, 0x00);
                        return (recv, CSW_STATUS_FAILED);
                    }
                    recv = recv.saturating_add(BLOCK_SIZE as u32);
                }
                (min(recv, exp), CSW_STATUS_PASSED)
            }
            _ => {
                self.set_sense(0x05, 0x20, 0x00);
                (0, CSW_STATUS_FAILED)
            }
        }
    }

    fn set_sense(&mut self, key: u8, asc: u8, ascq: u8) {
        self.sense = ScsiSense { key, asc, ascq };
    }
}

fn parse_cbw(buf: &[u8]) -> CommandBlockWrapper {
    let mut cb = CommandBlockWrapper {
        d_cbw_signature: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        d_cbw_tag: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        d_cbw_data_transfer_length: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
        bm_cbw_flags: buf[12],
        b_cbw_lun: buf[13],
        b_cbw_cb_length: buf[14],
        cbw_cb: [0u8; 16],
    };
    cb.cbw_cb.copy_from_slice(&buf[15..31]);
    cb
}

fn write_csw(csw: &CommandStatusWrapper, out: &mut [u8]) {
    out[0..4].copy_from_slice(&csw.d_csw_signature.to_le_bytes());
    out[4..8].copy_from_slice(&csw.d_csw_tag.to_le_bytes());
    out[8..12].copy_from_slice(&csw.d_csw_data_residue.to_le_bytes());
    out[12] = csw.b_csw_status;
}
