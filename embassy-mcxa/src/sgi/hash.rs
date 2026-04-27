// Hash functionality using SGI hardware

use core::sync::atomic::{Ordering, compiler_fence};

use embassy_hal_internal::Peri;

use super::{Async, Blocking, Sgi, SgiError};
use crate::peripherals;

/// Maximum SHA-384/512 block size in bytes.
pub const MAX_BLOCK_SIZE: usize = 128;

/// Output buffer size used by the SGI SHA2 engine (max digest size = SHA-512).
pub const SGI_HASH_OUTPUT_SIZE: u32 = 64;

use crate::dma::{DmaChannel, Transfer};

const SHA384_DIGEST_LEN: usize = 48;
const SHA512_DIGEST_LEN: usize = 64;

/// Two supported hash sizes: SHA-384 and SHA-512 (both CNSA 2.0 compliant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashSize {
    Sha384,
    Sha512,
}

impl From<HashSize> for usize {
    fn from(size: HashSize) -> usize {
        match size {
            HashSize::Sha384 => SHA384_DIGEST_LEN,
            HashSize::Sha512 => SHA512_DIGEST_LEN,
        }
    }
}

/// Mode of operation for SGI hash commands. Auto mode allows the FIFO to be managed by SGI itself, pacing the transfer of data from FIFO
/// to internal registers automatically. Normal mode requires manual management of the FIFO by the driver, limited to one block per invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMode {
    Auto,
    Normal,
}

/// Whether to initialize the hash state with the default IV (first block) or to continue from a previous hash state (streaming mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashInit {
    NoInit,
    Init,
}

/// Whether to reload a previously computed hash state into the SGI engine before processing the current block. This is used for chaining multiple blocks together in streaming mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashReload {
    NoReload,
    Reload,
}

/// Little vs. Big endian byte order for input and output data to/from SGI. Used as Little Endian for DMA transfers and big Endian for instruction driven transfers.
/// Outputs are always in big endian format for hashing operations.
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

    fn zeroize(&mut self) {
        // SAFETY: all-zeros is a valid bit pattern for SgiShaCtx — enum first variants have
        // discriminant 0, bool false = 0, numeric fields = 0, byte arrays = 0.
        unsafe { core::ptr::write_volatile(self as *mut Self, core::mem::zeroed()) };
        compiler_fence(Ordering::SeqCst);
    }
}

impl Drop for SgiShaCtx {
    fn drop(&mut self) {
        self.zeroize();
    }
}

// Controller.rs-style DMA hasher state.

/// SGI SHA2 DMA helper state.
///
/// The caller is responsible for creating the `Sgi<'d, Async>` instance (which enforces
/// the interrupt binding at the type level) and passing it here.
pub struct DmaHasher<'a, 'd> {
    sgi: Sgi<'d, Async>,
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
        sgi: Sgi<'d, Async>,
        dma_ch: &'a mut DmaChannel<'d>,
        hash_size: HashSize,
        input: &[u8],
        hash_result: &mut [u8],
    ) -> Result<(), SgiError> {
        let mut state = Self {
            sgi,
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

        let digest_len: usize = state.options.hash_size.into();
        if hash_result.len() < digest_len {
            return Err(SgiError::BufferTooSmall);
        }

        // Split input into full 128 bytes blocks (DMA path) and a trailing remainder.
        let chunks = input.chunks_exact(MAX_BLOCK_SIZE);
        let remainder = chunks.remainder();
        let dma_len_bytes = chunks.len() * MAX_BLOCK_SIZE;
        state.processed_bytes = dma_len_bytes;

        if !remainder.is_empty() {
            state.remainder[..remainder.len()].copy_from_slice(remainder);
        }

        if dma_len_bytes > 0 {
            let dma_input = &input[..dma_len_bytes];
            state.sgi.init_sgi_sha(state.options)?;

            // Arm operation-done interrupt before starting the hash operation.
            state.sgi.enable_operation_done_interrupt();
            state.sgi.start_sgi_hash(state.options, dma_input)?;
            state.transfer = Some(state.sgi.fill_sha2_fifo_dma_start(dma_ch, dma_input, dma_len_bytes)?);

            // After starting, subsequent operations must not auto-init unless re-init'd.
            state.options.init = HashInit::NoInit;
        }

        if let Some(transfer) = state.transfer.take() {
            transfer.await.map_err(|_| SgiError::DmaError)?;
            state.sgi.wait_sha2_complete_irq().await?;
        }

        let final_remainder_len = state.total_input_len - state.processed_bytes;
        let mut padded = [0u8; MAX_BLOCK_SIZE * 2];
        let final_remainder = state
            .remainder
            .get(..final_remainder_len)
            .ok_or(SgiError::InvalidSize)?;
        let padded_len = match create_padded_message(final_remainder, &mut padded) {
            Some(padded_len) => padded_len,
            None => {
                #[cfg(feature = "defmt")]
                defmt::error!("Failed to create padded message");
                return Err(SgiError::BufferTooSmall);
            }
        };

        if padded_len == 0 || padded_len > MAX_BLOCK_SIZE * 2 || (padded_len % MAX_BLOCK_SIZE) != 0 {
            return Err(SgiError::InvalidSize);
        }

        // IMPORTANT: total length is the total length of the original message, not just the remainder, since the SGI engine needs this for padding and length encoding in the final block(s).
        let bit_len = (state.total_input_len as u128) * 8;
        let length_field_offset = padded_len - 16;
        let length_field = padded
            .get_mut(length_field_offset..length_field_offset + 16)
            .ok_or(SgiError::InvalidSize)?;
        length_field.copy_from_slice(&bit_len.to_be_bytes());

        if state.processed_bytes > 0 {
            state.sgi.update_partial_output(state.options, &mut state.prev_result)?;
            state.sgi.sgi_hash_reload(state.options, &state.prev_result)?;
            state.options.init = HashInit::NoInit;
        } else {
            state.options.init = HashInit::Init;
        }

        state.options.op_mode = HashMode::Auto;
        // Switch back to Big-Endian for the final CPU-driven block, since the padding and length encoding as well as output digest are expected to
        // be in Big-Endian format.
        state.options.byte_order = ByteOrder::BigEndian;

        state.sgi.enable_operation_done_interrupt();
        state.sgi.init_sgi_sha(state.options)?;
        state.sgi.start_sgi_hash(state.options, &padded)?;
        state.sgi.fill_sha2_fifo(state.options, &padded, padded_len)?;

        state.sgi.wait_sha2_complete_irq().await?;
        state.sgi.read_hash_output(state.options, hash_result)?;

        state.zeroize();

        Ok(())
    }

    fn zeroize(&mut self) {
        volatile_zeroize(&mut self.remainder);
        volatile_zeroize(&mut self.prev_result);
        // SAFETY: pointers are valid references to fields of `self`.
        unsafe {
            core::ptr::write_volatile(&mut self.processed_bytes as *mut usize, 0);
            core::ptr::write_volatile(&mut self.total_input_len as *mut usize, 0);
        }
        compiler_fence(Ordering::SeqCst);
    }
}

// Zero a byte buffer in a way the compiler can't optimize away.
#[inline(never)]
fn volatile_zeroize(buf: &mut [u8]) {
    for byte in buf.iter_mut() {
        // SAFETY: `byte` is a valid reference to a single `u8` part of `buf`.
        unsafe { core::ptr::write_volatile(byte as *mut u8, 0) };
    }
}

impl Drop for DmaHasher<'_, '_> {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Blocking Hash instance that provides a simple interface for hashing data with SGI in a blocking manner, input size limited to 512 bytes.
pub struct BlockingHasher<'d> {
    sgi: Sgi<'d, Blocking>,
}

impl<'d> BlockingHasher<'d> {
    pub fn new(sgi: Sgi<'d, Blocking>) -> Self {
        Self { sgi }
    }

    /// Hash the provided input in one blocking call and write the digest into `hash_result`.
    pub fn hash_blocking(&mut self, hash_size: HashSize, input: &[u8], hash_result: &mut [u8]) -> Result<(), SgiError> {
        let sgi = &mut self.sgi;

        let options = HashOptions {
            hash_size: hash_size,
            op_mode: HashMode::Auto,
            init: HashInit::Init,
            reload: HashReload::NoReload,
            byte_order: ByteOrder::BigEndian,
        };

        let digest_len: usize = options.hash_size.into();

        if hash_result.len() < digest_len {
            return Err(SgiError::BufferTooSmall);
        }

        let required_total_len = calculate_padded_length(input.len()).ok_or(SgiError::InvalidSize)?;
        if required_total_len > MAX_BLOCK_SIZE * 5 {
            return Err(SgiError::BufferTooSmall);
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
            return Err(SgiError::InvalidSize);
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
        result.copy_from_slice(full_digest.get(..48)?);
        Some(result)
    }
}

fn calculate_padded_length(message_len: usize) -> Option<usize> {
    let bit_len = message_len.checked_mul(8)?;
    // Calculate k where (bit_len + 1 + k) ≡ 896 (mod 1024)
    // k = (896 - (bit_len + 1)) mod 1024, need to handle cases where bit_len + 1 > 896
    let remainder = bit_len.checked_add(1)? % 1024;
    let padding_bits = (1024 + 896 - remainder) % 1024;
    // Total bits guaranteed to be a multiple of 1024 and divisible by 8.
    let total_bits = bit_len.checked_add(1)?.checked_add(padding_bits)?.checked_add(128)?;
    Some(total_bits / 8) // Return bytes because array copy is in bytes.
}

// Create padded message according to FIPS-180-4 in a fixed buffer
// Returns the actual length used
fn create_padded_message(input: &[u8], buffer: &mut [u8]) -> Option<usize> {
    let padded_len = calculate_padded_length(input.len())?;
    if buffer.len() < padded_len {
        return None;
    }

    buffer.get_mut(..input.len())?.copy_from_slice(input); // Copy original message
    *buffer.get_mut(input.len())? = 0x80; // Add padding bit immediately after the last byte of the message per FIPS-180-4

    // Add 128-bit length in bits (big-endian) to last 16 bytes
    let bit_len = (input.len() as u128) * 8;
    let len_offset = padded_len - 16;
    buffer
        .get_mut(len_offset..len_offset + 16)?
        .copy_from_slice(&bit_len.to_be_bytes());

    Some(padded_len)
}

fn process_multi_block_update<'d>(
    hasher: &mut StreamingHasher,
    peri: &mut Sgi<'d>,
    input: &[u8],
) -> Result<(), SgiError> {
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
) -> Result<(), SgiError> {
    let input_len = input.len() as usize;
    let mut overflows_block = false;
    let mut write_bytes = 0;

    if hasher.ctx.curr_block_ptr + input_len < MAX_BLOCK_SIZE {
        // process_single_block_update() only handles sub-block inputs, so this end offset stays in-bounds.
        let block_end = hasher.ctx.curr_block_ptr + input_len;
        hasher
            .ctx
            .curr_block
            .get_mut(hasher.ctx.curr_block_ptr..block_end)
            .ok_or(SgiError::InvalidSize)?
            .copy_from_slice(input);
        hasher.ctx.curr_block_ptr += input_len;
        hasher.ctx.curr_block_ptr = hasher.ctx.curr_block_ptr % MAX_BLOCK_SIZE; // Wrap around if we exceed block size, but we won't process until we have a full block;
        return Ok(()); // Wait until we have a full block before processing
    } else if hasher.ctx.curr_block_ptr + input_len > MAX_BLOCK_SIZE {
        let space_left = MAX_BLOCK_SIZE - hasher.ctx.curr_block_ptr;
        let curr_block_tail = hasher
            .ctx
            .curr_block
            .get_mut(hasher.ctx.curr_block_ptr..)
            .ok_or(SgiError::InvalidSize)?;
        let input_prefix = input.get(..space_left).ok_or(SgiError::InvalidSize)?;
        curr_block_tail.copy_from_slice(input_prefix);
        write_bytes = input_len - space_left;
        overflows_block = true;
        hasher.ctx.curr_block_ptr = 0; // Reset pointer for the next block
    } else {
        // This branch exactly fills the current block, so the end offset lands on MAX_BLOCK_SIZE.
        let block_end = hasher.ctx.curr_block_ptr + input_len;
        hasher
            .ctx
            .curr_block
            .get_mut(hasher.ctx.curr_block_ptr..block_end)
            .ok_or(SgiError::InvalidSize)?
            .copy_from_slice(input);
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
        let curr_block_prefix = hasher
            .ctx
            .curr_block
            .get_mut(..write_bytes)
            .ok_or(SgiError::InvalidSize)?;
        // This overflow path only writes the leftover suffix, so `write_bytes` cannot exceed `input.len()`.
        let input_suffix = input.get(input.len() - write_bytes..).ok_or(SgiError::InvalidSize)?;
        curr_block_prefix.copy_from_slice(input_suffix);
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
    pub fn new(hash_size: HashSize, hash_mode: Option<HashMode>) -> Result<Self, SgiError> {
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
    pub fn update<'d>(&mut self, peri: Peri<'d, peripherals::SGI0>, input: &[u8]) -> Result<(), SgiError> {
        let mut sgi = Sgi::new_blocking(peri).map_err(|_| SgiError::HardwareError)?;
        let input_len = input.len() as usize;
        if input_len > MAX_BLOCK_SIZE * 4 || input_len == 0 {
            return Err(SgiError::InvalidSize);
            // 512 bytes seems like a reasonable upper limit for UPDATE calls that are > 128 bytes,
            // since this will require auto mode FIFO filling.
        }
        if self.ctx.curr_block_ptr > MAX_BLOCK_SIZE {
            return Err(SgiError::InvalidSize);
        }

        self.ctx.total_len = self.ctx.total_len.checked_add(input_len).ok_or(SgiError::InvalidSize)?;

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
            // Copy available but yet unprocessed data from the current block buffer. Copy is safe because `curr_block_ptr` is guaranteed to be less than or equal to `MAX_BLOCK_SIZE`, and thus less than the `copy_buffer` size.
            copy_buffer[..self.ctx.curr_block_ptr].copy_from_slice(&self.ctx.curr_block[..self.ctx.curr_block_ptr]);
            // Copy the rest of the data that fills up to `copy_len`, which is now a multiple of block size. Copy is safe because `copy_len` is guaranteed to be less than or equal to `input_len + self.ctx.curr_block_ptr`,
            // and we only copy the portion of `input` that fits within `copy_len - self.ctx.curr_block_ptr`.
            copy_buffer[self.ctx.curr_block_ptr..copy_len]
                .copy_from_slice(&input[..copy_len - self.ctx.curr_block_ptr]);

            process_multi_block_update(self, &mut sgi, &copy_buffer[..copy_len])?;

            self.ctx.processed_len = self
                .ctx
                .processed_len
                .checked_add(copy_len)
                .ok_or(SgiError::InvalidSize)?;
            let unprocessed_input_len = input_len - (copy_len - self.ctx.curr_block_ptr); // Calculate how much input is left after processing the copy buffer
            self.ctx.curr_block_ptr = unprocessed_input_len; // Set the current block pointer to the remaining unprocessed input length.

            #[cfg(feature = "defmt")]
            defmt::trace!(
                "Processed {=usize} bytes in auto mode, {=usize} bytes remain unprocessed in current block buffer",
                copy_len,
                unprocessed_input_len
            );

            let remaining_input_start = input_len
                .checked_sub(unprocessed_input_len)
                .ok_or(SgiError::InvalidSize)?;
            let remaining_input = input.get(remaining_input_start..).ok_or(SgiError::InvalidSize)?;
            let curr_block_prefix = self
                .ctx
                .curr_block
                .get_mut(..self.ctx.curr_block_ptr)
                .ok_or(SgiError::InvalidSize)?;
            curr_block_prefix.copy_from_slice(remaining_input); // Copy the remaining unprocessed input into the current block buffer for future processing
            self.ctx.options.op_mode = curr_op_mode; // Restore original mode after processing large input
            return Ok(());
        }
        process_single_block_update(self, &mut sgi, input)
    }

    /// Finalize the hash and write the digest into `hash_result`. This should be called after all update() calls are done. It will process any remaining data in the current block buffer, add padding, and produce the final hash output.
    pub fn finalize<'d>(&mut self, peri: Peri<'d, peripherals::SGI0>, hash_result: &mut [u8]) -> Result<(), SgiError> {
        let mut sgi = Sgi::new_blocking(peri).map_err(|_| SgiError::HardwareError)?;
        const MAX_FINAL_BUFFER_SIZE: usize = 256;

        let mut hash_buffer = [0u8; MAX_FINAL_BUFFER_SIZE]; // Buffer to hold the final block with padding, max size is 256 bytes to accommodate padding
        let remaining_data_len = self
            .ctx
            .total_len
            .checked_sub(self.ctx.processed_len)
            .ok_or(SgiError::InvalidSize)?;

        if remaining_data_len > 0 {
            if remaining_data_len > MAX_BLOCK_SIZE {
                return Err(SgiError::InvalidSize); // Can't have more than 128 bytes of unprocessed data for SHA-384/512, since that's the block size
            }
            // Process the remaining data in the current block buffer
            let remaining_curr_block = self
                .ctx
                .curr_block
                .get(..remaining_data_len)
                .ok_or(SgiError::InvalidSize)?;
            let hash_buffer_prefix = hash_buffer.get_mut(..remaining_data_len).ok_or(SgiError::InvalidSize)?;
            hash_buffer_prefix.copy_from_slice(remaining_curr_block);
        }

        let padded_total_len = calculate_padded_length(self.ctx.total_len).ok_or(SgiError::InvalidSize)?;
        let final_block_len = padded_total_len
            .checked_sub(self.ctx.processed_len)
            .ok_or(SgiError::InvalidSize)?; // Calculate how many bytes are in the final block (including padding)

        if remaining_data_len > final_block_len {
            return Err(SgiError::InvalidSize); // Remaining data can't exceed the final block length, otherwise we would need to process another block before finalizing
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
            return Err(SgiError::InvalidSize); // Final block cannot be larger than 144 bytes, any less fits in a block and any more would exceed the max padding size for a final block.
        }

        let digest_len: usize = self.ctx.options.hash_size.into();
        if hash_result.len() < digest_len {
            return Err(SgiError::BufferTooSmall);
        }

        *hash_buffer.get_mut(remaining_data_len).ok_or(SgiError::InvalidSize)? = 0x80; // Add the '1' bit padding immediately after the message data in the final block
        let len_offset = final_block_len.checked_sub(16).ok_or(SgiError::InvalidSize)?; // The last 16 bytes of the final block are reserved for the length
        // total_len widens from `usize` to `u128`, so multiplying by 8 cannot overflow.
        let bit_len = (self.ctx.total_len as u128) * 8;
        let len_field = hash_buffer
            .get_mut(len_offset..len_offset + 16)
            .ok_or(SgiError::InvalidSize)?;
        len_field.copy_from_slice(&bit_len.to_be_bytes());

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

        self.ctx.zeroize();

        Ok(())
    }
}
