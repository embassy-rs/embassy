use embassy_hal_internal::Peri;
use pac::hashcrypt::vals::Mode::Sha2256;
use pac::syscon::vals::HashAesRst::Released;

use crate::pac;
use crate::peripherals::HASHCRYPT;

// Helper function to enable and reset the hash peripheral\
fn enable_reset() {
    pac::SYSCON.ahbclkctrl2().modify(|w| {
        w.set_hash_aes(true);
    });

    pac::SYSCON.presetctrl2().modify(|w| {
        w.set_hash_aes_rst(Released);
    });
}
pub struct Sha256<'d> {
    _peri: Peri<'d, HASHCRYPT>,
    buffer: [u8; 64],
    buffer_len: usize,
    total_len: u64,
}

impl<'d> Sha256<'d> {
    pub fn new(peri: Peri<'d, HASHCRYPT>) -> Self {
        enable_reset();

        let sha256 = pac::HASHCRYPT;

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

    pub fn update(&mut self, data: &[u8]) {
        let data_len = data.len() as u32;
        let sha256 = pac::HASHCRYPT.indata();
        let mut offset = 0;
        self.total_len += data.len() as u64;

        while offset < data_len {
            // how much room do i have in the current word i think ?
            let space = 64 - self.buffer_len;

            // how many bites can i take from the data ?
            let take = space.min((data_len - offset) as usize);

            // move "take" bytes from the data to the buffer, ocupying whatever space is left in that particular word
            self.buffer[self.buffer_len..(self.buffer_len + take)]
                .copy_from_slice(&data[(offset as usize)..((offset) as usize) + take]);

            self.buffer_len += take;
            offset += take as u32;

            if self.buffer_len == 64 {
                // buffer is full, so we drain the message streamed so far in to the sha2 fifo
                for chunk in self.buffer.chunks_exact(4) {
                    let word = u32::from_le_bytes(chunk.try_into().unwrap());
                    sha256.write(|w| {
                        w.set_data(word);
                    });
                }
                self.buffer_len = 0;
            }
        }
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let sha256 = pac::HASHCRYPT.indata();

        // Separate the message from the padding with a single 1
        self.buffer[self.buffer_len] = 0x80;

        // add the padding untill the last 2 words
        for i in (self.buffer_len + 1)..56 {
            self.buffer[i] = 0;
        }

        // Append the size of the entire message
        let message_length = self.total_len * 8;
        self.buffer[56..64].copy_from_slice(&message_length.to_be_bytes());

        // now we can hash the final padded block

        for chunk in self.buffer.chunks_exact(4) {
            let word = u32::from_le_bytes(chunk.try_into().unwrap());
            sha256.write(|w| {
                w.set_data(word);
            });
        }

        // Now we can prepair the 8 word digest
        let mut digest: [u8; 32] = [0u8; 32];
        loop {
            let status = pac::HASHCRYPT.status().read().digest();
            if status {
                for i in 0..8 {
                    let word = pac::HASHCRYPT.digest0(i).read().digest();
                    digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
                }
                break;
            }
        }

        digest
    }
}
