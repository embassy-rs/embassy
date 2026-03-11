// Hash functionality using SGI hardware

use super::sgi::{Config as SgiConfig, SGIError, Sgi, SgiInterrupt};

/// Maximum SHA-384/512 block size in bytes.
pub const MAX_BLOCK_SIZE: usize = 128;

/// Output buffer size used by the SGI SHA2 engine (max digest size = SHA-512).
pub const SGI_HASH_OUTPUT_SIZE: u32 = 64;

use crate::dma::{DmaChannel, Transfer};
use crate::{Peri, peripherals};

const SHA384_DIGEST_LEN: usize = 48;
const SHA512_DIGEST_LEN: usize = 64;

#[inline(always)]
fn required_digest_len(hash_size: HashSize) -> Result<usize, SGIError> {
    match hash_size {
        HashSize::Sha384 => Ok(SHA384_DIGEST_LEN),
        HashSize::Sha512 => Ok(SHA512_DIGEST_LEN),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashSize {
    Sha384,
    Sha512,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMode {
    Auto,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashInit {
    NoInit,
    Init,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashReload {
    NoReload,
    Reload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

struct SgiShaCtx {
    options: HashOptions,
    prev_result: [u8; SGI_HASH_OUTPUT_SIZE as usize],
    curr_block: [u8; 128], // Buffer to hold the current block of data being processed, max block size for SHA-384/512 is 128 bytes
    curr_block_ptr: usize, // Pointer to track how much of the current block is filled
    total_len: usize,
    processed_len: usize,
    first_block: bool,
}

/// Options for SGI hash command
#[derive(Debug, Clone, Copy)]
pub struct HashOptions {
    /// SHA-384 or  SHA-512
    pub hash_size: HashSize,
    /// Normal (manual FIFO management) vs. Auto mode; 0 = Normal, 1 = Automatic.
    pub op_mode: HashMode,
    // Initialize hash with default IV
    pub init: HashInit,
    /// Reload a previously computed hash            
    pub reload: HashReload,
    /// Endian-ness of I/O to SGI.
    pub byte_order: ByteOrder,
}

impl Default for HashOptions {
    fn default() -> Self {
        Self {
            hash_size: HashSize::Sha384,
            op_mode: HashMode::Auto, // Automatic mode
            init: HashInit::Init,
            reload: HashReload::NoReload,
            byte_order: ByteOrder::BigEndian,
        }
    }
}

impl SgiShaCtx {
    pub fn new() -> Self {
        Self {
            options: HashOptions::default(),
            prev_result: [0xffu8; SGI_HASH_OUTPUT_SIZE as usize],
            curr_block: [0xffu8; 128],
            curr_block_ptr: 0,
            total_len: 0,
            processed_len: 0,
            first_block: true,
        }
    }
}

// Controller.rs-style DMA hasher state.

/// SGI SHA2 DMA helper state.
///
/// SGI singleton ownership is enforced by the `Sgi` constructor consuming the Embassy `Peri` token.
/// The `start_and_finalize` function takes ownership of the SGI peripheral token and a DMA channel,
/// starts the hash operation, and returns a future that resolves when the operation is complete and
/// the final hash is written to the provided output buffer.
pub struct DmaHasher<'a, 'd> {
    peri: Peri<'d, peripherals::SGI0>,
    transfer: Option<Transfer<'a>>,
    options: HashOptions,
    remainder: [u8; MAX_BLOCK_SIZE],
    prev_result: [u8; 64],
    processed_bytes: usize,
    total_input_len: usize,
}

impl<'a, 'd> DmaHasher<'a, 'd> {
    /// Start an SGI SHA2 DMA hash operation and await completion.
    ///
    /// This configures the SGI engine from the raw peripheral token,
    /// software-starts the eDMA channel, awaits DMA and SGI completion,
    /// and writes the final digest into `hash_result`.
    pub async fn start_and_finalize(
        peri: Peri<'d, peripherals::SGI0>,
        dma_ch: &'a mut DmaChannel<'d>,
        hash_size: HashSize,
        input: &[u8],
        hash_result: &mut [u8],
    ) -> Result<(), SGIError> {
        let mut state = Self {
            peri,
            transfer: None,
            options: HashOptions {
                hash_size,
                op_mode: HashMode::Auto,
                init: HashInit::Init,
                reload: HashReload::NoReload,
                // DMA feeds raw words/bytes into SHAFIFO, so the initial DMA phase must use
                // little-endian byte order in case a byte stream is fed. The final padded
                // block is CPU-fed below and switches back to big-endian explicitly.
                byte_order: ByteOrder::LittleEndian,
            },
            remainder: [0u8; MAX_BLOCK_SIZE],
            prev_result: [0xffu8; SGI_HASH_OUTPUT_SIZE as usize],
            processed_bytes: 0,
            total_input_len: input.len(),
        };

        let digest_len = required_digest_len(state.options.hash_size)?;
        if hash_result.len() < digest_len {
            return Err(SGIError::BufferTooSmall);
        }

        let mut sgi = Sgi::new(
            state.peri.reborrow(),
            SgiConfig {
                interrupt: SgiInterrupt::AsyncEnabled,
            },
        )
        .map_err(|_| SGIError::HardwareError)?;

        // We can only process data in chunks of "blocks" i.e. 128 bytes.
        let remainder_len = input.len() % MAX_BLOCK_SIZE;
        let dma_len_bytes = (input.len() / MAX_BLOCK_SIZE) * MAX_BLOCK_SIZE;
        state.processed_bytes = dma_len_bytes;

        if remainder_len > 0 {
            state.remainder[..remainder_len].copy_from_slice(&input[input.len() - remainder_len..]);
        }

        if dma_len_bytes > 0 {
            sgi.init_sgi_sha(state.options)?;

            // Arm operation-done interrupt before starting the hash operation.
            sgi.enable_operation_done_interrupt();
            sgi.start_sgi_hash(state.options, &input[..dma_len_bytes])?;
            state.transfer = Some(sgi.fill_sha2_fifo_dma_start(dma_ch, &input[..dma_len_bytes], dma_len_bytes)?);

            // After starting, subsequent operations must not auto-init unless re-init'd.
            state.options.init = HashInit::NoInit;
        }

        if let Some(transfer) = state.transfer.take() {
            transfer.await.map_err(|_| SGIError::DmaError)?;
            sgi.wait_sha2_complete_irq().await?;
        }

        let final_remainder_len = state.total_input_len - state.processed_bytes;
        let mut padded = [0u8; MAX_BLOCK_SIZE * 2];
        let padded_len = match create_padded_message(&state.remainder[..final_remainder_len], &mut padded) {
            Some(padded_len) => padded_len,
            None => {
                #[cfg(feature = "defmt")]
                defmt::error!("Failed to create padded message");
                return Err(SGIError::BufferTooSmall);
            }
        };

        if padded_len == 0 || padded_len > MAX_BLOCK_SIZE * 2 || (padded_len % MAX_BLOCK_SIZE) != 0 {
            return Err(SGIError::InvalidSize);
        }

        // IMPORTANT: total length is the total length of the original message, not just the remainder, since the SGI engine needs this for padding and length encoding in the final block(s).
        let bit_len = (state.total_input_len as u128) * 8;
        let length_field_offset = padded_len - 16;
        padded[length_field_offset..length_field_offset + 16].copy_from_slice(&bit_len.to_be_bytes());

        if state.processed_bytes > 0 {
            sgi.update_partial_output(state.options, &mut state.prev_result)?;
            sgi.sgi_hash_reload(state.options, &state.prev_result)?;
            state.options.init = HashInit::NoInit;
        } else {
            state.options.init = HashInit::Init;
        }

        state.options.op_mode = HashMode::Auto;
        // Switch back to Big-Endian for the final CPU-driven block, since the padding and length encoding as well as output digest are expected to
        // be in Big-Endian format.
        state.options.byte_order = ByteOrder::BigEndian;

        sgi.enable_operation_done_interrupt();
        sgi.init_sgi_sha(state.options)?;
        sgi.start_sgi_hash(state.options, &padded)?;
        sgi.fill_sha2_fifo(state.options, &padded, padded_len)?;

        sgi.wait_sha2_complete_irq().await?;
        sgi.read_hash_output(state.options, hash_result)?;

        Ok(())
    }
}

/// Blocking Hash instance that provides a simple interface for hashing data with SGI in a blocking manner, input size limited to 512 bytes.
/// Holds an SGI0 peri instance.

pub struct BlockingHasher<'d> {
    peri: Peri<'d, peripherals::SGI0>,
}

impl<'d> BlockingHasher<'d> {
    pub fn new(peri: Peri<'d, peripherals::SGI0>) -> Self {
        Self { peri }
    }

    /// Hash the provided input in one blocking call and write the digest into `hash_result`.
    pub fn hash_blocking(&mut self, hash_size: HashSize, input: &[u8], hash_result: &mut [u8]) -> Result<(), SGIError> {
        let mut sgi = Sgi::new_blocking(self.peri.reborrow()).map_err(|_| SGIError::HardwareError)?;

        let options = HashOptions {
            hash_size: hash_size,
            op_mode: HashMode::Auto,
            init: HashInit::Init,
            reload: HashReload::NoReload,
            byte_order: ByteOrder::BigEndian,
        };

        let digest_len = required_digest_len(options.hash_size)?;

        if hash_result.len() < digest_len {
            return Err(SGIError::BufferTooSmall);
        }

        let required_total_len = calculate_padded_length(input.len());
        if required_total_len > MAX_BLOCK_SIZE * 5 {
            return Err(SGIError::BufferTooSmall);
        }

        let mut hash_buffer = [0u8; MAX_BLOCK_SIZE * 5];
        let len = match create_padded_message(input, &mut hash_buffer) {
            Some(len) => len,
            None => {
                #[cfg(feature = "defmt")]
                defmt::error!("Failed to create padded message");
                0
            }
        };

        #[cfg(feature = "defmt")]
        defmt::trace!("Padded message length: {=usize}", len);
        if len == 0 || len > MAX_BLOCK_SIZE * 5 || len % 128 != 0 {
            #[cfg(feature = "defmt")]
            defmt::error!("Padded message length is not multiple of 128 bytes");
            return Err(SGIError::InvalidSize);
        }

        sgi.init_sgi_sha(options)?;
        sgi.start_sgi_hash(options, &hash_buffer)?;
        sgi.fill_sha2_fifo(options, &hash_buffer, len)?;
        sgi.read_hash_output(options, hash_result)?;
        Ok(())
    }

    /// Compute the SHA-512 RKTH value and truncate it to 48 bytes.
    pub fn hsm_sha512_rkth(&mut self, input: &[u8]) -> Option<[u8; 48]> {
        let mut full_digest = [0u8; 64];

        if let Err(e) = self.hash_blocking(HashSize::Sha512, input, &mut full_digest) {
            let _ = &e;
            #[cfg(feature = "defmt")]
            defmt::error!("Failed to compute SHA-512 for RKTH: {:?}", defmt::Debug2Format(&e));
            return None;
        }

        let mut result = [0u8; 48];
        result.copy_from_slice(&full_digest[..48]);
        Some(result)
    }
}

fn calculate_padded_length(message_len: usize) -> usize {
    let bit_len = message_len * 8;
    // Calculate k where (bit_len + 1 + k) ≡ 896 (mod 1024)
    // k = (896 - (bit_len + 1)) mod 1024, need to handle cases where bit_len + 1 > 896
    let remainder = (bit_len + 1) % 1024;
    let padding_bits = (1024 + 896 - remainder) % 1024;
    // Total bits guaranteed to be a multiple of 1024 and divisible by 8.
    let total_bits = bit_len + 1 + padding_bits + 128;
    total_bits / 8 // Return bytes because array copy is in bytes.
}

// Create padded message according to FIPS-180-4 in a fixed buffer
// Returns the actual length used
fn create_padded_message(input: &[u8], buffer: &mut [u8]) -> Option<usize> {
    let padded_len = calculate_padded_length(input.len());
    if buffer.len() < padded_len {
        return None;
    }

    buffer[..input.len()].copy_from_slice(input); // Copy original message
    buffer[input.len()] = 0x80; // Add padding bit immediately after the last byte of the message per FIPS-180-4

    // Add 128-bit length in bits (big-endian) to last 16 bytes
    let bit_len = (input.len() as u128) * 8;
    let len_offset = padded_len - 16;
    buffer[len_offset..len_offset + 16].copy_from_slice(&bit_len.to_be_bytes());

    Some(padded_len)
}

fn process_multi_block_update<'d>(
    hasher: &mut StreamingHasher,
    peri: &mut Sgi<'d>,
    input: &[u8],
) -> Result<(), SGIError> {
    // Considered copying via a DMA buffer, but it still doesn't solve the issue of having to wait
    // until the context partial hash state is updated (DMA + IRQ isn't really better than just
    // blocking and doing it in the same thread).
    //
    // The idea is to keep each `hasher.update()` call as user-friendly as possible and not require
    // the caller to manage block sizes or call a "finalize copy" after every update.
    if hasher.ctx.first_block {
        hasher.ctx.first_block = false;
    } else {
        // Continue hashing without auto-init.
        hasher.ctx.options.init = HashInit::NoInit;
        peri.sgi_hash_reload(hasher.ctx.options, &hasher.ctx.prev_result)?;
        peri.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the next one
        peri.wait_until_sha2_not_busy()?;
    }

    peri.init_sgi_sha(hasher.ctx.options)?;
    peri.start_sgi_hash(hasher.ctx.options, input)?;
    peri.fill_sha2_fifo(hasher.ctx.options, input, input.len())?;
    peri.update_partial_output(hasher.ctx.options, &mut hasher.ctx.prev_result)?;
    // Once we've processed at least one block, future operations must NOT auto-init
    // (otherwise the hardware IV would overwrite the chained state).
    hasher.ctx.options.init = HashInit::NoInit;
    Ok(())
}

fn process_single_block_update<'d>(
    hasher: &mut StreamingHasher,
    peri: &mut Sgi<'d>,
    input: &[u8],
) -> Result<(), SGIError> {
    let input_len = input.len() as usize;
    let mut overflows_block = false;
    let mut write_bytes = 0;

    if hasher.ctx.curr_block_ptr + input_len < MAX_BLOCK_SIZE {
        hasher.ctx.curr_block[hasher.ctx.curr_block_ptr..hasher.ctx.curr_block_ptr + input_len].copy_from_slice(input);
        hasher.ctx.curr_block_ptr += input_len;
        hasher.ctx.curr_block_ptr = hasher.ctx.curr_block_ptr % MAX_BLOCK_SIZE; // Wrap around if we exceed block size, but we won't process until we have a full block;
        return Ok(()); // Wait until we have a full block before processing
    } else if hasher.ctx.curr_block_ptr + input_len > MAX_BLOCK_SIZE {
        let space_left = MAX_BLOCK_SIZE - hasher.ctx.curr_block_ptr;
        hasher.ctx.curr_block[hasher.ctx.curr_block_ptr..].copy_from_slice(&input[..space_left]);
        write_bytes = input_len - space_left;
        overflows_block = true;
        hasher.ctx.curr_block_ptr = 0; // Reset pointer for the next block
    } else if hasher.ctx.curr_block_ptr + input_len == MAX_BLOCK_SIZE {
        hasher.ctx.curr_block[hasher.ctx.curr_block_ptr..hasher.ctx.curr_block_ptr + input_len].copy_from_slice(input);
        hasher.ctx.curr_block_ptr = 0; // Reset pointer for the next block
    }

    if hasher.ctx.first_block {
        hasher.ctx.first_block = false;
    } else {
        hasher.ctx.options.init = HashInit::NoInit;
        peri.sgi_hash_reload(hasher.ctx.options, &hasher.ctx.prev_result)?; // Load the previous hash state into SGI for the current block
        peri.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the next one
        peri.wait_until_sha2_not_busy()?;
    }

    peri.init_sgi_sha(hasher.ctx.options)?;
    peri.start_sgi_hash(hasher.ctx.options, &hasher.ctx.curr_block)?;
    peri.fill_sha2_fifo(hasher.ctx.options, &hasher.ctx.curr_block, MAX_BLOCK_SIZE)?;
    peri.update_partial_output(hasher.ctx.options, &mut hasher.ctx.prev_result)?;

    // After the first processed block, all subsequent operations must chain without auto-init.
    hasher.ctx.options.init = HashInit::NoInit;
    hasher.ctx.processed_len += MAX_BLOCK_SIZE;

    if overflows_block && write_bytes > 0 {
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Input overflows current block by {=usize} bytes, writing remaining bytes to next block",
            write_bytes
        );
        hasher.ctx.curr_block[..write_bytes].copy_from_slice(&input[input.len() - write_bytes..]);
        hasher.ctx.curr_block_ptr = write_bytes;
    }
    Ok(())
}

pub struct StreamingHasher {
    ctx: SgiShaCtx,
}

impl StreamingHasher {
    /// Create and initialize a streaming hasher with the specified hash size and mode.
    /// Leave `hash_mode` as `None` unless there are very specific performance or memory concerns.
    pub fn new(hash_size: HashSize, hash_mode: Option<HashMode>) -> Result<Self, SGIError> {
        let mut hasher = Self { ctx: SgiShaCtx::new() };
        hasher.ctx.options.hash_size = hash_size;

        let mode = hash_mode.unwrap_or(HashMode::Normal);

        hasher.ctx.options.op_mode = mode;
        hasher.ctx.options.init = HashInit::Init;
        hasher.ctx.options.reload = HashReload::NoReload;
        hasher.ctx.first_block = true;
        Ok(hasher)
    }

    /// Update the hash state with the provided input data. This can be called multiple times to process streaming data. Input limit per call is maximum 512 bytes,
    /// but it's recommended to keep it to 128 bytes (one block) for better performance and to avoid auto mode FIFO filling which can be slower.
    pub fn update<'d>(&mut self, peri: Peri<'d, peripherals::SGI0>, input: &[u8]) -> Result<(), SGIError> {
        let mut sgi = Sgi::new_blocking(peri).map_err(|_| SGIError::HardwareError)?;
        let input_len = input.len() as usize;
        if input_len > MAX_BLOCK_SIZE * 4 || input_len == 0 {
            return Err(SGIError::InvalidSize);
            // 512 bytes seems like a reasonable upper limit for UPDATE calls that are > 128 bytes,
            // since this will require auto mode FIFO filling.
        }
        self.ctx.total_len += input_len;

        if input_len > MAX_BLOCK_SIZE {
            let mut copy_buffer = [0u8; MAX_BLOCK_SIZE * 4]; // Temporary buffer to hold chunks of the input that fit within the block size, max 512 bytes.
            let curr_op_mode = self.ctx.options.op_mode;
            self.ctx.options.op_mode = HashMode::Auto; // Switch to auto mode for large inputs that exceed block size, since normal mode can only handle one block at a time

            let copy_len = if input_len + self.ctx.curr_block_ptr > copy_buffer.len() {
                copy_buffer.len()
            } else {
                ((input_len + self.ctx.curr_block_ptr) / MAX_BLOCK_SIZE) as usize * MAX_BLOCK_SIZE
            };

            // We can only copy as much as fits in the copy buffer, and we need to make sure we only copy full blocks worth of data for processing
            // Copy available but yet unprocessed data from the current block buffer.
            copy_buffer[..self.ctx.curr_block_ptr].copy_from_slice(&self.ctx.curr_block[..self.ctx.curr_block_ptr]);
            // Copy the rest of the data that fills up to `copy_len`, which is now a multiple of block size.
            copy_buffer[self.ctx.curr_block_ptr..copy_len]
                .copy_from_slice(&input[..copy_len - self.ctx.curr_block_ptr]);

            process_multi_block_update(self, &mut sgi, &copy_buffer[..copy_len])?;

            self.ctx.processed_len += copy_len;
            let unprocessed_input_len = input_len - (copy_len - self.ctx.curr_block_ptr); // Calculate how much input is left after processing the copy buffer
            self.ctx.curr_block_ptr = unprocessed_input_len; // Set the current block pointer to the remaining unprocessed input length.

            #[cfg(feature = "defmt")]
            defmt::trace!(
                "Processed {=usize} bytes in auto mode, {=usize} bytes remain unprocessed in current block buffer",
                copy_len,
                unprocessed_input_len
            );

            self.ctx.curr_block[..self.ctx.curr_block_ptr].copy_from_slice(&input[input_len - unprocessed_input_len..]); // Copy the remaining unprocessed input into the current block buffer for future processing
            self.ctx.options.op_mode = curr_op_mode; // Restore original mode after processing large input
            return Ok(());
        }
        process_single_block_update(self, &mut sgi, input)
    }

    /// Finalize the hash and write the digest into `hash_result`. This should be called after all update() calls are done. It will process any remaining data in the current block buffer, add padding, and produce the final hash output.
    pub fn finalize<'d>(&mut self, peri: Peri<'d, peripherals::SGI0>, hash_result: &mut [u8]) -> Result<(), SGIError> {
        let mut sgi = Sgi::new_blocking(peri).map_err(|_| SGIError::HardwareError)?;
        const MAX_FINAL_BUFFER_SIZE: usize = 256;

        let mut hash_buffer = [0u8; MAX_FINAL_BUFFER_SIZE]; // Buffer to hold the final block with padding, max size is 256 bytes to accommodate padding
        let remaining_data_len = self.ctx.total_len - self.ctx.processed_len;

        if remaining_data_len > 0 {
            if remaining_data_len > MAX_BLOCK_SIZE {
                return Err(SGIError::InvalidSize); // Can't have more than 128 bytes of unprocessed data for SHA-384/512, since that's the block size
            }
            // Process the remaining data in the current block buffer
            hash_buffer[..remaining_data_len].copy_from_slice(&self.ctx.curr_block[..remaining_data_len]);
        }

        let final_block_len = calculate_padded_length(self.ctx.total_len) - self.ctx.processed_len; // Calculate how many bytes are in the final block (including padding)

        if remaining_data_len > final_block_len {
            return Err(SGIError::InvalidSize); // Remaining data can't exceed the final block length, otherwise we would need to process another block before finalizing
        }

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Final block length (including padding): {=usize}, remaining data length: {=usize}, ctx.total_len: {=usize}, ctx.processed_len: {=usize}",
            final_block_len,
            remaining_data_len,
            self.ctx.total_len,
            self.ctx.processed_len
        );

        if final_block_len > MAX_FINAL_BUFFER_SIZE {
            return Err(SGIError::InvalidSize); // Final block cannot be larger than 144 bytes, any less fits in a block and any more would exceed the max padding size for a final block.
        }

        let digest_len = required_digest_len(self.ctx.options.hash_size)?;
        if hash_result.len() < digest_len {
            return Err(SGIError::BufferTooSmall);
        }

        hash_buffer[remaining_data_len] = 0x80; // Add the '1' bit padding immediately after the message data in the final block
        let len_offset = final_block_len - 16; // The last 16 bytes of the final block are reserved for the length
        let bit_len = (self.ctx.total_len as u128) * 8;
        hash_buffer[len_offset..len_offset + 16].copy_from_slice(&bit_len.to_be_bytes());

        let mut fifo_start = 0; // We will fill the FIFO starting from the beginning of the hash_buffer which contains the final block with padding
        let mut fifo_end = if self.ctx.options.op_mode == HashMode::Normal {
            MAX_BLOCK_SIZE
        } else {
            final_block_len
        };

        // In NORMAL mode we can only process 128 bytes at a time, in AUTO mode we can process the entire final block at once, even if it's larger than 128 bytes.

        let passes = if self.ctx.options.op_mode == HashMode::Normal {
            if final_block_len > MAX_BLOCK_SIZE { 2 } else { 1 }
        } else {
            1
        };

        let remaining_final_block_len = if self.ctx.options.op_mode == HashMode::Normal {
            MAX_BLOCK_SIZE
        } else {
            final_block_len
        }; // In AUTO mode we process the entire final block at once, so we don't have any remaining data to process after the first pass.

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Final block will be processed in {=usize} pass(es) with FIFO range [{=usize}..{=usize}] with len {=usize}",
            passes,
            fifo_start,
            fifo_end,
            remaining_final_block_len
        );

        // If final block length exceeds 128 bytes, it means we have to process two blocks in normal mode,
        // but only one block in auto mode since auto mode can handle more than 128 bytes in the FIFO.
        //
        // Remember: normal mode can't handle more than 128 bytes (one SHA2 block) in the FIFO.
        // So if we have more than 128 bytes in the final block (which can happen if we have a lot of remaining data plus padding),
        // we need to process it in two steps: first fill the FIFO with the initial part of the final block (up to 128 bytes) and start the hash,
        // then fill the FIFO with the remaining part of the final block and continue.

        for _ in 0..passes {
            sgi.wait_until_sha2_not_busy()?;

            if self.ctx.processed_len >= MAX_BLOCK_SIZE {
                // At least one block has already been processed; reload the hash state for the final block.
                // Ensure we continue from the reloaded state; do not auto-init after reload.
                self.ctx.options.init = HashInit::NoInit;
                sgi.sgi_hash_reload(self.ctx.options, &self.ctx.prev_result)?;
                sgi.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the final block
                sgi.wait_until_sha2_not_busy()?;
            } else {
                // If we haven't processed any full blocks yet, it means all the data is in the final block and we
                // can start hashing without reloading since we're still in the initial state.
                self.ctx.options.init = HashInit::Init; // Initialize hash state for the final block since we haven't processed any blocks yet
            }

            sgi.init_sgi_sha(self.ctx.options)?;
            sgi.start_sgi_hash(self.ctx.options, &hash_buffer[fifo_start..fifo_end])?;
            sgi.fill_sha2_fifo(
                self.ctx.options,
                &hash_buffer[fifo_start..fifo_end],
                remaining_final_block_len,
            )?;

            fifo_start += MAX_BLOCK_SIZE;
            fifo_end += MAX_BLOCK_SIZE;
            self.ctx.processed_len += MAX_BLOCK_SIZE;
        }

        sgi.read_hash_output(self.ctx.options, hash_result)?;
        Ok(())
    }
}
