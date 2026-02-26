// Hash functionality using SGI hardware
use crate::sgi::{
    SGIError, Sgi, SGI_HASH_OUTPUT_SIZE, SGI_SHA2_CTRL_AUTO_INIT, SGI_SHA2_CTRL_MODE_AUTO, SGI_SHA2_CTRL_MODE_NORMAL,
    SGI_SHA2_CTRL_NO_AUTO_INIT, SGI_SHA2_CTRL_NO_HASH_RELOAD, SGI_SHA2_CTRL_SIZE_SHA384, SGI_SHA2_CTRL_SIZE_SHA512,
};
use embassy_mcxa::dma::DmaChannel;
use embassy_time::{with_timeout, Duration, Instant, Timer};
use log::{error, info};

use core::future::poll_fn;
use core::task::Poll;

const MAX_BLOCK_SIZE: usize = 128;
const SHA384_DIGEST_LEN: usize = 48;
const SHA512_DIGEST_LEN: usize = 64;

#[inline(always)]
fn required_digest_len(hash_mode: u32) -> Result<usize, SGIError> {
    match hash_mode {
        SGI_SHA2_CTRL_SIZE_SHA384 => Ok(SHA384_DIGEST_LEN),
        SGI_SHA2_CTRL_SIZE_SHA512 => Ok(SHA512_DIGEST_LEN),
        _ => Err(SGIError::InvalidSize),
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

impl HashMode {
    fn to_sgi_op_mode(self) -> u32 {
        match self {
            HashMode::Auto => SGI_SHA2_CTRL_MODE_AUTO,
            HashMode::Normal => SGI_SHA2_CTRL_MODE_NORMAL,
        }
    }
}

impl HashSize {
    fn to_sgi_sha2_size(self) -> u32 {
        match self {
            HashSize::Sha384 => SGI_SHA2_CTRL_SIZE_SHA384,
            HashSize::Sha512 => SGI_SHA2_CTRL_SIZE_SHA512,
        }
    }
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
    pub hash_mode: u32,       // SHA-384, SHA-512, etc.
    pub op_mode: u32,         // 0 = Normal, 1 = Automatic
    pub init: u32,            // Initialize hash
    pub reload: u32,          // Load hash
    pub output_enable: bool,  // Enable output
    pub byte_order_big: bool, // Big endian byte order
}

impl Default for HashOptions {
    fn default() -> Self {
        Self {
            hash_mode: SGI_SHA2_CTRL_SIZE_SHA384,
            op_mode: SGI_SHA2_CTRL_MODE_AUTO, // Automatic mode
            init: SGI_SHA2_CTRL_AUTO_INIT,
            reload: SGI_SHA2_CTRL_NO_HASH_RELOAD,
            output_enable: true,
            byte_order_big: true,
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

// ============================================================================
// Controller.rs-style DMA hasher (context-only handle + async finish)
// ============================================================================

/// A non-blocking SGI SHA2 DMA operation context.
///
/// This handle owns the SGI singleton claim for the duration of the operation,
/// and stores an [`Sgi`] instance (so the singleton lock is held until `finish`).
///
/// Dropping this handle early will stop the SGI operation (via `Sgi`'s `Drop`),
/// which is consistent with typical "drop cancels" async patterns.
pub struct DmaHasher<'a, 'd> {
    sgi_instance: Sgi,
    dma_channel: &'a mut DmaChannel<'d>,
    options: HashOptions,
    remainder: [u8; MAX_BLOCK_SIZE],
    prev_result: [u8; 64],
    dma_len: usize,
    processed_bytes: usize,
    total_input_len: usize,
}

impl<'a, 'd> DmaHasher<'a, 'd> {
    fn dma_done_or_error(&mut self) -> Result<bool, SGIError> {
        let tcd = self.dma_channel.tcd();
        if tcd.ch_es().read().err() {
            return Err(SGIError::DmaError);
        }

        Ok(tcd.ch_csr().read().done())
    }

    /// Await DMA completion and SGI readiness using the DMA channel interrupt waker.
    async fn wait_done(&mut self) -> Result<(), SGIError> {
        if self.dma_len == 0 {
            return Ok(());
        }

        const MAX_DMA_WAIT_USEC: u64 = 5000;
        let start_time = Instant::now();

        poll_fn(|cx| {
            // Register waker first (avoid race)
            self.dma_channel.waker().register(cx.waker());

            // Bound the wait so we don't hang forever if an IRQ never arrives.
            if Instant::now().duration_since(start_time).as_micros() > MAX_DMA_WAIT_USEC {
                error!("Timeout waiting for SGI DMA operation to complete");
                return Poll::Ready(Err(SGIError::Timeout));
            }

            // DMA complete => proceed. SGI busy is handled in a separate wait with its own timeout.
            match self.dma_done_or_error() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(false) => Poll::Pending,
                Ok(true) => Poll::Ready(Ok(())),
            }
        })
        .await?;

        // DMA is done; now wait for the SGI SHA2 engine to become idle with its own timeout.
        // This wait is timer-driven so it doesn't depend on any further DMA interrupts.
        const MAX_SGI_WAIT_USEC: u64 = 500;
        const SGI_POLL_INTERVAL_USEC: u64 = 100;
        let wait_sgi = async {
            while self.sgi_instance.is_sha2_busy() {
                Timer::after(Duration::from_micros(SGI_POLL_INTERVAL_USEC)).await;
            }

            // Only check SGI status once the engine is no longer busy,
            // so timeout/error state is reliably observable.
            self.sgi_instance.sgi_operation_error()?;
            self.sgi_instance.sgi_error()?;
            Ok(())
        };

        match with_timeout(Duration::from_micros(MAX_SGI_WAIT_USEC), wait_sgi).await {
            Ok(on_time) => on_time,
            Err(_) => {
                error!("Timeout waiting for SGI SHA2 engine to finish after DMA completion");
                Err(SGIError::Timeout)
            }
        }
    }

    /// Finish the operation and write the digest into `hash_result`.
    pub async fn finalize(mut self, hash_result: &mut [u8]) -> Result<(), SGIError> {
        self.wait_done().await?;

        let digest_len = required_digest_len(self.options.hash_mode)?;
        if hash_result.len() < digest_len {
            return Err(SGIError::BufferTooSmall);
        }

        let remainder_len = self.total_input_len - self.processed_bytes;
        // Buffer to hold the final padded block (max 2 blocks = 256 bytes).
        // This covers the case where padding expands into an extra block.
        let mut padded = [0u8; MAX_BLOCK_SIZE * 2];
        let padded_len = match create_padded_message(&self.remainder[..remainder_len], &mut padded) {
            Some(padded_len) => padded_len,
            None => {
                error!("Failed to create padded message");
                return Err(SGIError::BufferTooSmall);
            }
        };

        if padded_len == 0 || padded_len > MAX_BLOCK_SIZE * 2 || (padded_len % MAX_BLOCK_SIZE) != 0 {
            return Err(SGIError::InvalidSize);
        }

        // IMPORTANT: length field is total message length in bits.
        let bit_len = (self.total_input_len as u128) * 8;
        let length_field_offset = padded_len - 16;
        padded[length_field_offset..length_field_offset + 16].copy_from_slice(&bit_len.to_be_bytes());

        // If we DMA-fed at least one full block, chain state through DATIN reload.
        if self.processed_bytes > 0 {
            self.sgi_instance
                .update_partial_output(self.options, &mut self.prev_result)?;
            self.sgi_instance.sgi_hash_reload(self.options, &self.prev_result)?;

            // Final block must not auto-init; we want to continue from the reloaded state.
            self.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
        } else {
            // No prior blocks: a normal one-shot finalization.
            self.options.init = SGI_SHA2_CTRL_AUTO_INIT;
        }

        self.options.byte_order_big = true;
        self.options.op_mode = SGI_SHA2_CTRL_MODE_AUTO;
        self.sgi_instance.init_sgi_sha(self.options)?;
        self.sgi_instance.start_sgi_hash(self.options, &padded)?;
        self.sgi_instance.fill_sha2_fifo(self.options, &padded, padded_len)?;
        self.sgi_instance.read_hash_output(self.options, hash_result)?;

        Ok(())
    }
}

/// Start an SGI SHA2 DMA hash operation and return a context handle.
///
/// This configures the SGI engine and software-starts the eDMA channel, then
/// returns immediately. Completion is awaited via [`DmaHasher::finalize`].
pub fn hash_dma_start<'a, 'd>(
    dma_ch: &'a mut DmaChannel<'d>,
    hash_size: HashSize,
    input: &[u8],
) -> Result<DmaHasher<'a, 'd>, SGIError> {
    let mut sgi = Sgi::new()?;

    let mut options = HashOptions {
        hash_mode: hash_size.to_sgi_sha2_size(),
        op_mode: SGI_SHA2_CTRL_MODE_AUTO,
        init: SGI_SHA2_CTRL_AUTO_INIT,
        reload: SGI_SHA2_CTRL_NO_HASH_RELOAD,
        output_enable: true,
        byte_order_big: false, // byte stream -> 4 byte FIFO writes
    };

    // Feed as many full blocks as possible by DMA. The final remainder is padded in finalize().
    let remainder_len = input.len() % MAX_BLOCK_SIZE;
    let dma_len_bytes = (input.len() / MAX_BLOCK_SIZE) * MAX_BLOCK_SIZE;

    let mut remainder_buf = [0u8; MAX_BLOCK_SIZE];
    if remainder_len > 0 {
        remainder_buf[..remainder_len].copy_from_slice(&input[input.len() - remainder_len..]);
    }

    if dma_len_bytes > 0 {
        sgi.init_sgi_sha(options)?;
        sgi.start_sgi_hash(options, &input[..dma_len_bytes])?;
        sgi.fill_sha2_fifo_dma_start(dma_ch, &input[..dma_len_bytes], dma_len_bytes)?;

        // After starting, subsequent operations must not auto-init unless re-init'd.
        options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
    }

    Ok(DmaHasher {
        sgi_instance: sgi,
        dma_channel: dma_ch,
        options,
        remainder: remainder_buf,
        prev_result: [0xffu8; SGI_HASH_OUTPUT_SIZE as usize],
        dma_len: dma_len_bytes,
        processed_bytes: dma_len_bytes,
        total_input_len: input.len(),
    })
}

pub fn hash_blocking(hash_size: HashSize, input: &[u8], hash_result: &mut [u8]) -> Result<(), SGIError> {
    let options = HashOptions {
        hash_mode: hash_size.to_sgi_sha2_size(),
        op_mode: SGI_SHA2_CTRL_MODE_AUTO,
        init: SGI_SHA2_CTRL_AUTO_INIT,
        reload: SGI_SHA2_CTRL_NO_HASH_RELOAD,
        output_enable: true,
        byte_order_big: true,
    };
    let digest_len = required_digest_len(options.hash_mode)?;
    if hash_result.len() < digest_len {
        return Err(SGIError::BufferTooSmall);
    }
    let required_total_len = calculate_padded_length(input.len());
    if required_total_len > MAX_BLOCK_SIZE * 5 {
        return Err(SGIError::BufferTooSmall);
    }
    let mut hash_buffer = [0u8; MAX_BLOCK_SIZE * 5]; // Buffer to hold the padded message, max size is 640 bytes to accommodate padding for messages up to 512 bytes
    let len = match create_padded_message(input, &mut hash_buffer) {
        Some(len) => len,
        None => {
            error!("Failed to create padded message");
            0
        }
    };
    info!("Padded message length: {}", len);
    if len == 0 || len > MAX_BLOCK_SIZE * 5 || len % 128 != 0 {
        error!("Padded message length is not multiple of 128 bytes");
        return Err(SGIError::InvalidSize);
    }
    let mut sgi = match Sgi::new() {
        Ok(sgi) => sgi,
        Err(e) => {
            error!("Failed to create SGI instance: {:?}", e);
            return Err(e);
        }
    };
    sgi.init_sgi_sha(options)?;
    sgi.start_sgi_hash(options, &hash_buffer)?;
    sgi.fill_sha2_fifo(options, &hash_buffer, len)?;
    sgi.read_hash_output(options, hash_result)?;
    Ok(())
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

fn process_multi_block_update(hasher: &mut SGIHasher, input: &[u8]) -> Result<(), SGIError> {
    let mut sgi = Sgi::new()?;
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
        hasher.ctx.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
        sgi.sgi_hash_reload(hasher.ctx.options, &hasher.ctx.prev_result)?;
        sgi.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the next one
        sgi.wait_until_sha2_not_busy()?;
    }

    sgi.init_sgi_sha(hasher.ctx.options)?;
    sgi.start_sgi_hash(hasher.ctx.options, input)?;
    sgi.fill_sha2_fifo(hasher.ctx.options, input, input.len())?;
    sgi.update_partial_output(hasher.ctx.options, &mut hasher.ctx.prev_result)?;
    // Once we've processed at least one block, future operations must NOT auto-init
    // (otherwise the hardware IV would overwrite the chained state).
    hasher.ctx.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
    Ok(())
}

fn process_single_block_update(hasher: &mut SGIHasher, input: &[u8]) -> Result<(), SGIError> {
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

    let mut sgi = Sgi::new()?;

    if hasher.ctx.first_block {
        hasher.ctx.first_block = false;
    } else {
        hasher.ctx.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
        sgi.sgi_hash_reload(hasher.ctx.options, &hasher.ctx.prev_result)?; // Load the previous hash state into SGI for the current block
        sgi.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the next one
        sgi.wait_until_sha2_not_busy()?;
    }
    sgi.init_sgi_sha(hasher.ctx.options)?;
    sgi.start_sgi_hash(hasher.ctx.options, &hasher.ctx.curr_block)?;
    sgi.fill_sha2_fifo(hasher.ctx.options, &hasher.ctx.curr_block, MAX_BLOCK_SIZE)?;
    sgi.update_partial_output(hasher.ctx.options, &mut hasher.ctx.prev_result)?;
    //info!("Hash update successful, current hash state: {:02X?}", &hasher.ctx.prev_result[..]);
    // After the first processed block, all subsequent operations must chain without auto-init.
    hasher.ctx.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
    hasher.ctx.processed_len += MAX_BLOCK_SIZE;

    if overflows_block && write_bytes > 0 {
        info!(
            "Input overflows current block by {} bytes, writing remaining bytes to next block",
            write_bytes
        );
        hasher.ctx.curr_block[..write_bytes].copy_from_slice(&input[input.len() - write_bytes..]);
        hasher.ctx.curr_block_ptr = write_bytes;
    }
    Ok(())
}

pub struct SGIHasher {
    ctx: SgiShaCtx,
}

impl SGIHasher {
    pub fn new() -> Self {
        Self { ctx: SgiShaCtx::new() }
    }

    pub fn init(&mut self, hash_size: HashSize, hash_mode: Option<HashMode>) -> Result<(), SGIError> {
        self.ctx = SgiShaCtx::new();
        self.ctx.options.hash_mode = hash_size.to_sgi_sha2_size();
        let mode = hash_mode.unwrap_or(HashMode::Normal);
        self.ctx.options.op_mode = mode.to_sgi_op_mode();
        self.ctx.options.init = SGI_SHA2_CTRL_AUTO_INIT;
        self.ctx.options.reload = SGI_SHA2_CTRL_NO_HASH_RELOAD;
        self.ctx.first_block = true;
        Ok(())
    }

    pub fn update(&mut self, input: &[u8]) -> Result<(), SGIError> {
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
            self.ctx.options.op_mode = SGI_SHA2_CTRL_MODE_AUTO; // Switch to auto mode for large inputs that exceed block size, since normal mode can only handle one block at a time
            let copy_len = if input_len + self.ctx.curr_block_ptr > copy_buffer.len() {
                copy_buffer.len()
            } else {
                ((input_len + self.ctx.curr_block_ptr) / MAX_BLOCK_SIZE) as usize * MAX_BLOCK_SIZE
            };
            // We can only copy as much as fits in the copy buffer, and we need to make sure we only copy full blocks worth of data for processing
            // Copy available but yet unprocessed data from the current block buffer.
            copy_buffer[..self.ctx.curr_block_ptr].copy_from_slice(&self.ctx.curr_block[..self.ctx.curr_block_ptr]);
            copy_buffer[self.ctx.curr_block_ptr..copy_len]
                // Copy the rest of the data that fills up to `copy_len`, which is now a multiple of block size.
                .copy_from_slice(&input[..copy_len - self.ctx.curr_block_ptr]);
            process_multi_block_update(self, &copy_buffer[..copy_len])?;
            self.ctx.processed_len += copy_len;
            let unprocessed_input_len = input_len - (copy_len - self.ctx.curr_block_ptr); // Calculate how much input is left after processing the copy buffer
            self.ctx.curr_block_ptr = unprocessed_input_len; // Set the current block pointer to the remaining unprocessed input length.
            info!(
                "Processed {} bytes in auto mode, {} bytes remain unprocessed in current block buffer",
                copy_len, unprocessed_input_len
            );
            self.ctx.curr_block[..self.ctx.curr_block_ptr].copy_from_slice(&input[input_len - unprocessed_input_len..]); // Copy the remaining unprocessed input into the current block buffer for future processing
            self.ctx.options.op_mode = curr_op_mode; // Restore original mode after processing large input
            return Ok(());
        }
        process_single_block_update(self, input)
    }

    pub fn finalize(&mut self, hash_result: &mut [u8]) -> Result<(), SGIError> {
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
        info!(
            "Final block length (including padding): {}, remaining data length: {}, ctx.total_len: {}, ctx.processed_len: {}",
            final_block_len,
            remaining_data_len,
            self.ctx.total_len,
            self.ctx.processed_len
        );
        if final_block_len > MAX_FINAL_BUFFER_SIZE {
            return Err(SGIError::InvalidSize); // Final block cannot be larger than 144 bytes, any less fits in a block and any more would exceed the max padding size for a final block.
        }
        let digest_len = required_digest_len(self.ctx.options.hash_mode)?;
        if hash_result.len() < digest_len {
            return Err(SGIError::BufferTooSmall);
        }

        hash_buffer[remaining_data_len] = 0x80; // Add the '1' bit padding immediately after the message data in the final block
        let len_offset = final_block_len - 16; // The last 16 bytes of the final block are reserved for the length
        let bit_len = (self.ctx.total_len as u128) * 8;
        hash_buffer[len_offset..len_offset + 16].copy_from_slice(&bit_len.to_be_bytes());

        let mut sgi = Sgi::new()?;
        let mut fifo_start = 0; // We will fill the FIFO starting from the beginning of the hash_buffer which contains the final block with padding
        let mut fifo_end = if self.ctx.options.op_mode == SGI_SHA2_CTRL_MODE_NORMAL {
            MAX_BLOCK_SIZE
        } else {
            final_block_len
        }; // In NORMAL mode we can only process 128 bytes at a time, in AUTO mode we can process the entire final block at once, even if it's larger than 128 bytes.
        let passes = if self.ctx.options.op_mode == SGI_SHA2_CTRL_MODE_NORMAL {
            if final_block_len > MAX_BLOCK_SIZE {
                2
            } else {
                1
            }
        } else {
            1
        };
        let remaining_final_block_len = if self.ctx.options.op_mode == SGI_SHA2_CTRL_MODE_NORMAL {
            MAX_BLOCK_SIZE
        } else {
            final_block_len
        }; // In AUTO mode we process the entire final block at once, so we don't have any remaining data to process after the first pass.
        info!(
            "Final block will be processed in {} pass(es) with FIFO range [{}..{}] in mode {} with len {}",
            passes, fifo_start, fifo_end, self.ctx.options.op_mode, remaining_final_block_len
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
                self.ctx.options.init = SGI_SHA2_CTRL_NO_AUTO_INIT;
                sgi.sgi_hash_reload(self.ctx.options, &self.ctx.prev_result)?;
                sgi.sgi_stop_sha2_cmd(); // Ensure we stop any ongoing hash operation before starting the final block
                sgi.wait_until_sha2_not_busy()?;
            } else {
                // If we haven't processed any full blocks yet, it means all the data is in the final block and we
                // can start hashing without reloading since we're still in the initial state.
                self.ctx.options.init = SGI_SHA2_CTRL_AUTO_INIT; // Initialize hash state for the final block since we haven't processed any blocks yet
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

// The following is a wrapper for SHA-512 ROTKH calculation which always returns 48 bytes by truncating
// the full SHA-512 digest.

// Hardware SHA-512 RKTH - always returns 48 bytes for RKTH calculation
pub fn hsm_sha512_rkth(input: &[u8]) -> Option<[u8; 48]> {
    let mut full_digest = [0u8; 64]; // Buffer to hold the full SHA-512 digest
    if let Err(e) = hash_blocking(HashSize::Sha512, input, &mut full_digest) {
        error!("Failed to compute SHA-512 for RKTH: {:?}", e);
        return None;
    }
    let mut result = [0u8; 48];
    result.copy_from_slice(&full_digest[..48]);
    Some(result)
}
