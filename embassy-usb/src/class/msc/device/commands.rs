//! SCSI command handlers that don't stream block data.
//!
//! Each handler fills the response buffer and returns a [`Reply`], or the sense data to report.

use core::cmp::min;

use super::super::bot::Cbw;
use super::super::scsi::*;
use super::{AsyncBlockDevice, Config, StopReason};

/// Size of the buffer command handlers write their Data-In response to.
pub(super) const RESPONSE_LEN: usize = 40;

pub(super) type Response = [u8; RESPONSE_LEN];

/// What a command handler asks the dispatcher to do once the command succeeded.
#[derive(Clone, Copy)]
pub(super) enum Reply {
    /// Complete the command with no data phase.
    Done,
    /// Send this many bytes of the response buffer to the host.
    Data(usize),
    /// Complete the command, then stop [`MscClass::run`].
    Stop(StopReason),
}

impl Reply {
    /// Sends `len` bytes of the response, truncated to the CDB's allocation length.
    pub(super) fn data(len: usize, allocation_length: usize) -> Self {
        Self::Data(min(len, allocation_length))
    }

    /// Copies `data` to the response buffer and sends it, truncated to the CDB's allocation length.
    pub(super) fn copy(buf: &mut Response, data: &[u8], allocation_length: usize) -> Self {
        buf[..data.len()].copy_from_slice(data);
        Self::data(data.len(), allocation_length)
    }
}

pub(super) fn request_sense(
    sense: &SenseData,
    allocation_length: usize,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    // The dispatcher then resets the sense data, per SCSI SPC-2.
    Ok(Reply::copy(buf, &sense.encode(), allocation_length))
}

pub(super) fn inquiry(config: &Config, inquiry: Inquiry, buf: &mut Response) -> Result<Reply, SenseData> {
    // CmdDt is obsolete and unsupported.
    if inquiry.cmd_dt {
        return Err(SenseData::INVALID_FIELD_IN_CDB);
    }

    let len = if inquiry.evpd {
        match inquiry.page_code {
            VPD_PAGE_SUPPORTED_PAGES => {
                let data = [
                    PERIPHERAL_DIRECT_ACCESS,
                    VPD_PAGE_SUPPORTED_PAGES,
                    0x00,
                    0x03, // 3 bytes follow
                    VPD_PAGE_SUPPORTED_PAGES,
                    VPD_PAGE_UNIT_SERIAL_NUMBER,
                    VPD_PAGE_DEVICE_IDENTIFICATION,
                ];
                return Ok(Reply::copy(buf, &data, inquiry.allocation_length));
            }
            VPD_PAGE_UNIT_SERIAL_NUMBER => {
                let serial = config.serial_number_bytes();
                buf[1] = VPD_PAGE_UNIT_SERIAL_NUMBER;
                buf[3] = serial.len() as u8;
                buf[4..4 + serial.len()].copy_from_slice(serial);
                4 + serial.len()
            }
            VPD_PAGE_DEVICE_IDENTIFICATION => {
                let id = config.serial_number_bytes();
                buf[1] = VPD_PAGE_DEVICE_IDENTIFICATION;
                if id.is_empty() {
                    4
                } else {
                    buf[3] = (4 + id.len()) as u8; // page length
                    buf[4] = VPD_CODE_SET_ASCII;
                    buf[5] = VPD_DESIGNATOR_VENDOR_SPECIFIC;
                    buf[7] = id.len() as u8;
                    buf[8..8 + id.len()].copy_from_slice(id);
                    8 + id.len()
                }
            }
            _ => return Err(SenseData::INVALID_FIELD_IN_CDB),
        }
    } else {
        // Standard inquiry should request page code 0.
        if inquiry.page_code != 0 {
            return Err(SenseData::INVALID_FIELD_IN_CDB);
        }

        buf[0] = PERIPHERAL_DIRECT_ACCESS;
        buf[1] = INQUIRY_RMB;
        buf[2] = INQUIRY_VERSION_SPC2;
        buf[3] = INQUIRY_RESPONSE_DATA_FORMAT;
        buf[4] = (INQUIRY_DATA_LEN - 5) as u8; // additional length
        buf[8..16].copy_from_slice(&config.vendor_id);
        buf[16..32].copy_from_slice(&config.product_id);
        buf[32..36].copy_from_slice(&config.product_revision_level);
        INQUIRY_DATA_LEN
    };

    Ok(Reply::data(len, inquiry.allocation_length))
}

pub(super) fn mode_sense_6<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    buf[0] = 3; // Mode data length
    buf[2] = device_specific_parameter(block_device);
    Ok(Reply::data(4, allocation_length))
}

pub(super) fn mode_sense_10<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    buf[1] = 6; // Mode data length (6 bytes follow)
    buf[2] = 0; // medium type
    buf[3] = device_specific_parameter(block_device);
    Ok(Reply::data(8, allocation_length))
}

pub(super) fn read_format_capacities<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    let block_size = block_device.block_size();

    buf[3] = 8;
    buf[4..8].copy_from_slice(&block_device.block_count().to_be_bytes());
    buf[8] = RFC_FORMATTED_MEDIA;
    buf[9] = (block_size >> 16) as u8;
    buf[10] = (block_size >> 8) as u8;
    buf[11] = block_size as u8;
    Ok(Reply::data(12, allocation_length))
}

pub(super) fn read_capacity_10<B: AsyncBlockDevice>(block_device: &B, buf: &mut Response) -> Result<Reply, SenseData> {
    // Sent whole, as READ CAPACITY(10) has no allocation length.
    Ok(Reply::copy(buf, &capacity(block_device).encode_10(), usize::MAX))
}

pub(super) fn read_capacity_16<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    Ok(Reply::copy(buf, &capacity(block_device).encode_16(), allocation_length))
}

/// MODE SENSE device-specific parameter for `block_device`.
pub(super) fn device_specific_parameter<B: AsyncBlockDevice>(block_device: &B) -> u8 {
    if block_device.is_write_protected() {
        MODE_DSP_WP
    } else {
        0
    }
}

pub(super) fn capacity<B: AsyncBlockDevice>(block_device: &B) -> Capacity {
    Capacity {
        last_lba: block_device.block_count().saturating_sub(1) as u64,
        block_size: block_device.block_size(),
    }
}

pub(super) async fn start_stop_unit<B: AsyncBlockDevice>(
    eject: bool,
    block_device: &mut B,
) -> Result<Reply, SenseData> {
    if !eject {
        return Ok(Reply::Done);
    }

    if block_device.flush().await.is_err() {
        warn!("msc: flush before eject failed");
        return Err(SenseData::WRITE_ERROR);
    }

    Ok(Reply::Stop(StopReason::Ejected))
}

pub(super) async fn synchronize_cache_10<B: AsyncBlockDevice>(block_device: &mut B) -> Result<Reply, SenseData> {
    if block_device.flush().await.is_err() {
        warn!("msc: flush failed");
        return Err(SenseData::WRITE_ERROR);
    }

    Ok(Reply::Done)
}

pub(super) fn report_luns(select_report: u8, allocation_length: usize, buf: &mut Response) -> Result<Reply, SenseData> {
    let Some(data) = lun_list(select_report).filter(|_| allocation_length >= RL_MIN_ALLOCATION_LENGTH) else {
        return Err(SenseData::INVALID_FIELD_IN_CDB);
    };

    Ok(Reply::copy(buf, data, allocation_length))
}

/// Checks a READ or WRITE command against `block_device` and `block_buf`.
///
/// Returns the starting LBA, or the sense data to report if the command can't run.
pub(super) fn check_read_write<B: AsyncBlockDevice>(
    cbw: &Cbw,
    rw: ReadWrite,
    block_device: &B,
    block_buf: &[u8],
) -> Result<u32, SenseData> {
    let block_size = block_device.block_size();
    if block_size == 0
        || block_buf.len() < block_size as usize
        || rw.blocks.checked_mul(block_size) != Some(cbw.data_transfer_length)
    {
        return Err(SenseData::INVALID_FIELD_IN_CDB);
    }

    if rw
        .lba
        .checked_add(rw.blocks as u64)
        .is_none_or(|end| end > block_device.block_count() as u64)
    {
        return Err(SenseData::LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE);
    }

    // In range, so it fits in u32 like the block count.
    Ok(rw.lba as u32)
}

/// REPORT LUNS parameter data for a SELECT REPORT value, or `None` if the value is unsupported.
///
/// LUN 0 is the only logical unit, and there are no well-known logical units.
fn lun_list(select_report: u8) -> Option<&'static [u8]> {
    match select_report {
        // LUN LIST LENGTH, 4 reserved bytes, then LUN 0.
        RL_SELECT_ALL | RL_SELECT_ALL_AND_WELL_KNOWN => Some(&[0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        RL_SELECT_WELL_KNOWN => Some(&[0; 8]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lun_list_reports_lun_0() {
        let lun_0 = [0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        core::assert_eq!(lun_list(RL_SELECT_ALL), Some(&lun_0[..]));
        core::assert_eq!(lun_list(RL_SELECT_ALL_AND_WELL_KNOWN), Some(&lun_0[..]));
        core::assert_eq!(lun_list(RL_SELECT_WELL_KNOWN), Some(&[0u8; 8][..]));
        core::assert_eq!(lun_list(0x03), None);
    }
}
