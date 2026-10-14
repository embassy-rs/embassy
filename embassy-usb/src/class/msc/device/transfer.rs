//! READ and WRITE, which stream block data between the host and storage.

use core::cmp::min;

use embassy_futures::join::join;

use super::super::bot::Cbw;
use super::super::scsi::*;
use super::commands::check_read_write;
use super::{AsyncBlockDevice, CommandResult, MscClass};
use crate::driver::{Driver, EndpointError, EndpointIn, EndpointOut};

impl<'d, D: Driver<'d>> MscClass<'d, D> {
    pub(super) async fn read<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        rw: ReadWrite,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        let lba = match check_read_write(cbw, rw, block_device, block_buf) {
            Ok(lba) => lba,
            Err(sense) => {
                self.sense = sense;
                return Ok(CommandResult::failed(cbw.data_transfer_length));
            }
        };

        if rw.blocks == 0 {
            self.sense = SenseData::NO_SENSE;
            return Ok(CommandResult::passed(0));
        }

        let blocks = rw.blocks;
        let block_size = block_device.block_size() as usize;
        let (mut front, mut back) = split_chunks(block_buf, block_size);
        let chunk_blocks = (front.len() / block_size) as u32;

        let mut residue = cbw.data_transfer_length;
        let mut chunk_lba = lba;
        let mut len = min(blocks, chunk_blocks) as usize * block_size;
        let mut filled = block_device.read_blocks(chunk_lba, &mut front[..len]).await;

        loop {
            if filled.is_err() {
                warn!("msc: read_blocks failed at lba {}", chunk_lba);
                self.sense = SenseData::UNRECOVERED_READ_ERROR;
                if residue < cbw.data_transfer_length {
                    let _ = self.write_ep.write(&[]).await;
                }
                return Ok(CommandResult::failed(residue));
            }

            let next_lba = chunk_lba + (len / block_size) as u32;
            let next = min(lba + blocks - next_lba, chunk_blocks) as usize * block_size;

            if back.is_empty() {
                self.write_ep.write_transfer(&front[..len], false).await?;
                filled = read_chunk(block_device, next_lba, &mut front[..next]).await;
            } else {
                // Send this chunk while storage reads the next one.
                let (sent, read) = join(
                    self.write_ep.write_transfer(&front[..len], false),
                    read_chunk(block_device, next_lba, &mut back[..next]),
                )
                .await;
                sent?;
                filled = read;
                core::mem::swap(&mut front, &mut back);
            }

            residue -= len as u32;

            if next == 0 {
                break;
            }

            chunk_lba = next_lba;
            len = next;
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(residue))
    }

    pub(super) async fn write<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        rw: ReadWrite,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        let checked = check_read_write(cbw, rw, block_device, block_buf).and_then(|lba| {
            if block_device.is_write_protected() {
                Err(SenseData::WRITE_PROTECTED)
            } else {
                Ok(lba)
            }
        });
        let lba = match checked {
            Ok(lba) => lba,
            Err(sense) => {
                self.sense = sense;
                self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
                return Ok(CommandResult::failed(cbw.data_transfer_length));
            }
        };

        if rw.blocks == 0 {
            self.sense = SenseData::NO_SENSE;
            return Ok(CommandResult::passed(0));
        }

        let blocks = rw.blocks;
        let block_size = block_device.block_size() as usize;
        let (mut front, mut back) = split_chunks(block_buf, block_size);
        let chunk_blocks = (front.len() / block_size) as u32;

        let mut received = 0;
        let mut chunk_lba = lba;
        let mut len = min(blocks, chunk_blocks) as usize * block_size;

        read_exact_out(&mut self.read_ep, &mut front[..len]).await?;

        received += len as u32;

        let failed = loop {
            let next_lba = chunk_lba + (len / block_size) as u32;
            let next = min(lba + blocks - next_lba, chunk_blocks) as usize * block_size;

            let stored = if back.is_empty() {
                block_device.write_blocks(chunk_lba, &front[..len]).await
            } else {
                // Receive the next chunk while storage writes this one.
                let (got, stored) = join(
                    read_exact_out(&mut self.read_ep, &mut back[..next]),
                    block_device.write_blocks(chunk_lba, &front[..len]),
                )
                .await;

                got?;
                received += next as u32;

                stored
            };

            if stored.is_err() {
                break Some(chunk_lba);
            }
            if next == 0 {
                break None;
            }

            if back.is_empty() {
                read_exact_out(&mut self.read_ep, &mut front[..next]).await?;
                received += next as u32;
            } else {
                core::mem::swap(&mut front, &mut back);
            }

            chunk_lba = next_lba;
            len = next;
        };

        if let Some(failed_lba) = failed {
            warn!("msc: write_blocks failed at lba {}", failed_lba);
            self.sense = SenseData::WRITE_ERROR;

            let remaining_to_drain = cbw.data_transfer_length - received;
            if remaining_to_drain > 0 {
                self.discard_out_data(remaining_to_drain, block_buf).await?;
            }

            let written = (failed_lba - lba) * block_size as u32;
            return Ok(CommandResult::failed(cbw.data_transfer_length - written));
        }

        if block_device.flush().await.is_err() {
            warn!("msc: flush after write failed");
            self.sense = SenseData::WRITE_ERROR;
            return Ok(CommandResult::failed(0));
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(0))
    }
}

/// Splits `buf` into two chunks of whole blocks so USB transfers can overlap with storage access.
///
/// Both chunks are the same size. If `buf` holds a single block, the second chunk is empty.
fn split_chunks(buf: &mut [u8], block_size: usize) -> (&mut [u8], &mut [u8]) {
    let blocks = buf.len() / block_size;
    let back_len = blocks / 2 * block_size;
    let (front, back) = buf.split_at_mut(blocks.div_ceil(2) * block_size);

    (&mut front[..back_len.max(block_size)], &mut back[..back_len])
}

async fn read_chunk<B: AsyncBlockDevice>(block_device: &mut B, lba: u32, buf: &mut [u8]) -> Result<(), B::Error> {
    if buf.is_empty() {
        return Ok(());
    }

    block_device.read_blocks(lba, buf).await
}

async fn read_exact_out(ep: &mut impl EndpointOut, mut buf: &mut [u8]) -> Result<(), EndpointError> {
    while !buf.is_empty() {
        let n = ep.read_transfer(buf).await?;
        buf = &mut buf[n..];
    }

    Ok(())
}
