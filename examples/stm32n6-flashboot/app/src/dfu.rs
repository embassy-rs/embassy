//! XMODEM-CRC firmware receiver for UART DFU.
//!
//! Implements XMODEM-CRC (16-bit CRC) receiver to accept firmware over UART
//! and write it to the DFU partition in external NOR flash.
//!
//! Usage: start the XMODEM sender on the host FIRST (e.g. `sx firmware.bin`),
//! then reset the board with PE0 held to enter DFU mode.

use defmt::info;
use embassy_stm32::mode::Blocking;
use embassy_stm32::usart::{UartRx, UartTx};

use crate::flash_ops;

// XMODEM protocol constants
const SOH: u8 = 0x01; // 128-byte data packet
const STX: u8 = 0x02; // 1024-byte data packet
const EOT: u8 = 0x04; // End of transmission
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CRC_START: u8 = b'C'; // Request CRC mode

const CHUNK_SIZE: usize = 4096; // NOR flash sector size

/// Compute CRC-16/XMODEM (polynomial 0x1021).
fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

fn read_byte(rx: &mut UartRx<'_, Blocking>) -> Result<u8, ()> {
    let mut buf = [0u8; 1];
    rx.blocking_read(&mut buf).map_err(|_| ())?;
    Ok(buf[0])
}

fn send_byte(tx: &mut UartTx<'_, Blocking>, b: u8) {
    let _ = tx.blocking_write(&[b]);
}

/// Receive firmware via XMODEM-CRC and write to DFU partition.
///
/// The XMODEM sender must already be running on the host before this is called.
/// Returns the total number of firmware bytes written on success.
pub fn receive_firmware(mut rx: UartRx<'_, Blocking>, mut tx: UartTx<'_, Blocking>) -> Result<usize, ()> {
    unsafe extern "C" {
        static __dfu_max_fw_size: u8;
    }
    let max_fw_size = core::ptr::addr_of!(__dfu_max_fw_size) as u32;

    info!("XMODEM: waiting for sender (max fw size: {}K)...", max_fw_size / 1024);

    let mut chunk_buf = [0xFFu8; CHUNK_SIZE];
    let mut chunk_pos: usize = 0;
    let mut flash_offset: u32 = 0;
    let mut expected_seq: u8 = 1;
    let mut total_bytes: usize = 0;
    let mut pkt_buf = [0u8; 1024];

    // Send 'C' to initiate CRC mode. The sender (already running) will respond
    // with the first packet immediately.
    send_byte(&mut tx, CRC_START);

    loop {
        let header = match read_byte(&mut rx) {
            Ok(b) => b,
            Err(_) => return Err(()),
        };

        match header {
            SOH | STX => {
                let data_len = if header == SOH { 128 } else { 1024 };
                match receive_packet(&mut rx, data_len, expected_seq, &mut pkt_buf) {
                    Ok(()) => {
                        // Write data into chunk buffer, flush when full
                        let mut src = 0;
                        while src < data_len {
                            let space = CHUNK_SIZE - chunk_pos;
                            let n = (data_len - src).min(space);
                            chunk_buf[chunk_pos..chunk_pos + n].copy_from_slice(&pkt_buf[src..src + n]);
                            chunk_pos += n;
                            src += n;

                            if chunk_pos >= CHUNK_SIZE {
                                if flash_offset + CHUNK_SIZE as u32 > max_fw_size {
                                    info!("XMODEM: firmware too large (>{} bytes), aborting", max_fw_size);
                                    return Err(());
                                }
                                flash_ops::write_dfu_chunk(flash_offset, &chunk_buf);
                                flash_offset += CHUNK_SIZE as u32;
                                chunk_pos = 0;
                                chunk_buf = [0xFFu8; CHUNK_SIZE];
                                info!("XMODEM: {}K written", flash_offset / 1024);
                            }
                        }

                        expected_seq = expected_seq.wrapping_add(1);
                        total_bytes += data_len;
                        send_byte(&mut tx, ACK);
                    }
                    Err(PktError::BadSeqDuplicate) => {
                        // Duplicate of previous packet — ACK but don't process
                        send_byte(&mut tx, ACK);
                    }
                    Err(_) => {
                        send_byte(&mut tx, NAK);
                    }
                }
            }
            EOT => {
                send_byte(&mut tx, ACK);

                // Flush remaining data (padded with 0xFF already)
                if chunk_pos > 0 {
                    if flash_offset + chunk_pos as u32 > max_fw_size {
                        info!("XMODEM: firmware too large (>{} bytes), aborting", max_fw_size);
                        return Err(());
                    }
                    flash_ops::write_dfu_chunk(flash_offset, &chunk_buf[..chunk_pos]);
                }

                info!("XMODEM: received {} bytes", total_bytes);
                flash_ops::mark_updated();
                info!("XMODEM: marked updated, safe to reboot");

                return Ok(total_bytes);
            }
            _ => {
                // Unexpected byte — probably noise, NAK to resync
                send_byte(&mut tx, NAK);
            }
        }
    }
}

enum PktError {
    ReadError,
    BadCrc,
    BadSeq,
    BadSeqDuplicate,
}

/// Receive and validate an XMODEM packet (seq, !seq, data, CRC).
/// On success, data is in pkt_buf[..data_len].
fn receive_packet(
    rx: &mut UartRx<'_, Blocking>,
    data_len: usize,
    expected_seq: u8,
    pkt_buf: &mut [u8; 1024],
) -> Result<(), PktError> {
    let seq = read_byte(rx).map_err(|_| PktError::ReadError)?;
    let seq_comp = read_byte(rx).map_err(|_| PktError::ReadError)?;

    for i in 0..data_len {
        pkt_buf[i] = read_byte(rx).map_err(|_| PktError::ReadError)?;
    }

    let crc_hi = read_byte(rx).map_err(|_| PktError::ReadError)?;
    let crc_lo = read_byte(rx).map_err(|_| PktError::ReadError)?;
    let received_crc = ((crc_hi as u16) << 8) | (crc_lo as u16);

    // Verify sequence
    if seq != expected_seq || seq_comp != !expected_seq {
        if seq == expected_seq.wrapping_sub(1) {
            return Err(PktError::BadSeqDuplicate);
        }
        return Err(PktError::BadSeq);
    }

    // Verify CRC
    if crc16(&pkt_buf[..data_len]) != received_crc {
        return Err(PktError::BadCrc);
    }

    Ok(())
}
