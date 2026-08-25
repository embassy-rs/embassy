use embassy_hal_internal::Peri;
use pac::hashcrypt::vals::Mode::Sha2256;
use pac::syscon::vals::HashAesRst::Released;

use crate::pac;
use crate::peripherals::HASHCRYPT;

// Helper function to enable the clock and reset the HASHCRYPT peripheral
fn enable_reset() {
    pac::SYSCON.ahbclkctrl2().modify(|w| {
        w.set_hash_aes(true);
    });

    pac::SYSCON.presetctrl2().modify(|w| {
        w.set_hash_aes_rst(Released);
    });
}

// Helper function to drain buffered data in to the FIFO for Sha256
fn drain_sha2_buffer(buffer: &[u8; 64]) {
    let sha256 = pac::HASHCRYPT.indata();
    for chunk in buffer.chunks_exact(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        sha256.write(|w| {
            w.set_data(word);
        });
    }
}
// The necessary fields for the implementation of SHA-256
pub struct Sha256<'d> {
    _peri: Peri<'d, HASHCRYPT>, // The HASHCRYPT peripheral it self, which will be owned by the struct
    buffer: [u8; 64],           // A 64 byte buffer in which we can dump incoming data streams
    buffer_len: usize,          // The number of valid bytes currently buffered, resets to 0 after buffer is drained
    total_len: u64,             // The size of the complete message to be hashed
}

impl<'d> Sha256<'d> {
    // The function which takes ownership of the HASHCRYPT peripheral
    // and returns an owned value through Self
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
        enable_reset();

        let sha256 = pac::HASHCRYPT;

        // Initialise the HASHCRYPT peripheral in SHA-256 mode
        sha256.ctrl().modify(|w| {
            w.set_mode(Sha2256);
            w.set_new_hash(true);
        });

        Self {
            _peri: peri,
            buffer: [0u8; 64],
            buffer_len: 0usize,
            total_len: 0u64,
        }
    }
    // Accepts an arbitrary-length slice of bytes at the time, to dump in to the buffer until a full 64 byte
    // block is built, then drained in to the FIFO
    pub fn update(&mut self, data: &[u8]) {
        let data_len = data.len() as u32; // Length of the incoming data
        let mut offset = 0; // tracks how many bytes of `data` have been consumed so far

        self.total_len += data.len() as u64;
        while offset < data_len {
            // how much room is there in the buffer ?
            let space = 64 - self.buffer_len;

            // how much can i take from the incoming data ?
            let take = space.min((data_len - offset) as usize);

            // move "take" bytes from the data to the buffer, occupying whatever space is left in the buffer
            self.buffer[self.buffer_len..(self.buffer_len + take)]
                .copy_from_slice(&data[(offset as usize)..((offset) as usize) + take]);

            self.buffer_len += take;
            offset += take as u32;

            // Once the buffer is full, we drain it in to the FIFO via .indata().set_data()
            if self.buffer_len == 64 {
                // buffer is full, so we drain the message streamed so far in to the sha2 FIFO
                drain_sha2_buffer(&self.buffer);
                // Once the 16 word FIFO is full (see [drain_sha2_buffer]), hashing begins automatically, and we are free to start
                // overwriting the buffer so we can fill it once more with the incoming data

                // Reset the buffer
                self.buffer_len = 0;
                self.buffer = [0u8; 64];
            }

            // Even though a digest might be ready to read at this point,
            // polling HASHCRYPT.status().read().digest() here would yield a useless, incomplete hash,
            // caused by an incomplete message. Instead, we will poll it in [finalize()] when we can
            // be sure that no more data will be streamed.
        }
    }

    // Hashes whatever is left in the buffer after [update()] and resets the HASHCRYPT peripheral
    // so that the hashing of a new message is possible
    pub fn finalize(&mut self) -> [u8; 32] {
        // Now that we know that there is no more incoming data from this message, we can
        // start padding the message
        // Padding according to FIPS 180-4 §5.1.1, pg 13

        // Separate the message from the padding with a single 1
        self.buffer[self.buffer_len] = 0x80;

        // Is there room for the size of the message in the current block ?
        if self.buffer_len < 56 {
            // Add the padding until the last 2 words
            for i in (self.buffer_len + 1)..56 {
                self.buffer[i] = 0;
            }

            // Append the size of the entire message
            let message_length = self.total_len * 8;
            self.buffer[56..64].copy_from_slice(&message_length.to_be_bytes());
        } else {
            // Pad until the block is completely filled
            for i in (self.buffer_len + 1)..64 {
                self.buffer[i] = 0;
            }

            // Drain the buffer in to the FIFO
            drain_sha2_buffer(&self.buffer);

            // Reset
            self.buffer = [0u8; 64];
            self.buffer_len = 0;

            // Append the size of the entire message
            let message_length = self.total_len * 8;
            self.buffer[56..64].copy_from_slice(&message_length.to_be_bytes());
        }

        // Now we can hash the final padded block
        // Hashing begins automatically once the 16 words (512 bits) of the FIFO are full
        drain_sha2_buffer(&self.buffer);

        // Now we can prepare the 8 word digest
        let mut digest: [u8; 32] = [0u8; 32];
        loop {
            let status = pac::HASHCRYPT.status().read().digest();
            // if status is true, that we have a digest ready to be read. We poll digest here and not in
            // [update(&mut self, data: &[u8])] since we want to make sure that the entire message has
            // been streamed.
            if status {
                for i in 0..8 {
                    let word = pac::HASHCRYPT.digest0(i).read().digest();
                    digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
                }
                break;
            }
            // If status is false, then that means there is no digest ready to be read, and since there is
            // no more incoming data, then that means we just need to keep waiting and so there is no need
            // to explicitly handle that case
        }

        // Reset the HASHCRYPT peripheral, so it's ready for a new hash. When finalize() is called again,
        // all the registers including the length of the message that was previously hashed are reset, so that
        // the next digest is correct and free of residual values from previous hash operations.
        pac::HASHCRYPT.ctrl().modify(|w| {
            w.set_new_hash(true);
        });

        // Clean the buffer and total message length to get rid of any leftovers from previous messages.
        self.buffer = [0u8; 64];
        self.buffer_len = 0;
        self.total_len = 0;

        digest
    }
}
