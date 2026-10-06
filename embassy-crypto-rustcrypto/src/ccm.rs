use embassy_crypto::Error;
use embassy_crypto::driver::InOutBuf;

/// The block encryption function.
type Aes<'a> = &'a dyn Fn(&mut [u8; 16]);

fn encrypt_block(aes: Aes<'_>, block: &mut [u8; 16]) {
    aes(block);
}

/// Validate the parameters and build the `B0` block and the initial counter block.
fn setup(nonce: &[u8], aad: &[u8], payload_len: usize, tag_len: usize) -> Result<([u8; 16], [u8; 16]), Error> {
    if !(7..=13).contains(&nonce.len()) {
        return Err(Error::InvalidInput);
    }
    if !(4..=16).contains(&tag_len) || tag_len % 2 != 0 {
        return Err(Error::InvalidInput);
    }
    // q: size of the length field; the payload length must fit in it.
    let q = 15 - nonce.len();
    if q < 8 && (payload_len as u64) >= (1u64 << (8 * q)) {
        return Err(Error::InvalidInput);
    }

    let mut b0 = [0u8; 16];
    b0[0] = (if aad.is_empty() { 0 } else { 0x40 }) | (((tag_len - 2) / 2) as u8) << 3 | (q - 1) as u8;
    b0[1..1 + nonce.len()].copy_from_slice(nonce);
    let len = (payload_len as u64).to_be_bytes();
    b0[16 - q..].copy_from_slice(&len[8 - q..]);

    let mut ctr = [0u8; 16];
    ctr[0] = (q - 1) as u8;
    ctr[1..1 + nonce.len()].copy_from_slice(nonce);

    Ok((b0, ctr))
}

/// CBC-MAC accumulator.
struct Mac<'a> {
    aes: Aes<'a>,
    state: [u8; 16],
    buf: [u8; 16],
    pos: usize,
}

impl<'a> Mac<'a> {
    fn new(aes: Aes<'a>, b0: [u8; 16]) -> Self {
        let mut state = b0;
        encrypt_block(aes, &mut state);
        Self {
            aes,
            state,
            buf: [0; 16],
            pos: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        while !data.is_empty() {
            let n = (16 - self.pos).min(data.len());
            for (b, d) in self.buf[self.pos..self.pos + n].iter_mut().zip(data) {
                *b = *d;
            }
            self.pos += n;
            data = &data[n..];
            if self.pos == 16 {
                self.flush();
            }
        }
    }

    /// Absorb the buffered partial block, zero-padded.
    fn pad(&mut self) {
        if self.pos != 0 {
            self.buf[self.pos..].fill(0);
            self.pos = 16;
            self.flush();
        }
    }

    fn flush(&mut self) {
        for (s, b) in self.state.iter_mut().zip(&self.buf) {
            *s ^= b;
        }
        encrypt_block(self.aes, &mut self.state);
        self.pos = 0;
    }

    fn finish(mut self) -> [u8; 16] {
        self.pad();
        self.state
    }
}

/// Absorb the associated data with its length prefix.
fn mac_aad(mac: &mut Mac<'_>, aad: &[u8]) {
    if aad.is_empty() {
        return;
    }
    let len = aad.len() as u64;
    if len < (1 << 16) - (1 << 8) {
        mac.update(&(len as u16).to_be_bytes());
    } else if len < (1 << 32) {
        mac.update(&[0xff, 0xfe]);
        mac.update(&(len as u32).to_be_bytes());
    } else {
        mac.update(&[0xff, 0xff]);
        mac.update(&len.to_be_bytes());
    }
    mac.update(aad);
    mac.pad();
}

/// Increment the counter in the last `q` bytes of the counter block.
fn increment(ctr: &mut [u8; 16], q: usize) {
    for b in ctr[16 - q..].iter_mut().rev() {
        *b = b.wrapping_add(1);
        if *b != 0 {
            break;
        }
    }
}

/// Apply the CTR keystream (counters 1..) to `buf`.
fn ctr_apply(aes: Aes<'_>, ctr: &mut [u8; 16], q: usize, buf: InOutBuf<'_, '_, u8>) {
    let out = buf.into_out_with_copied_in();
    for chunk in out.chunks_mut(16) {
        increment(ctr, q);
        let mut ks = *ctr;
        encrypt_block(aes, &mut ks);
        for (o, k) in chunk.iter_mut().zip(&ks) {
            *o ^= k;
        }
    }
}

/// Encrypt `buffer` in place, writing the tag to `tag`.
pub fn encrypt(
    aes: Aes<'_>,
    nonce: &[u8],
    aad: &[u8],
    buffer: InOutBuf<'_, '_, u8>,
    tag: &mut [u8],
) -> Result<(), Error> {
    let (b0, mut ctr) = setup(nonce, aad, buffer.len(), tag.len())?;
    let q = 15 - nonce.len();

    let mut mac = Mac::new(aes, b0);
    mac_aad(&mut mac, aad);
    mac.update(buffer.get_in());
    let t = mac.finish();

    // S0 = E(K, Ctr0) masks the tag.
    let mut s0 = ctr;
    encrypt_block(aes, &mut s0);
    for ((out, t), s) in tag.iter_mut().zip(&t).zip(&s0) {
        *out = t ^ s;
    }

    ctr_apply(aes, &mut ctr, q, buffer);
    Ok(())
}

/// Verify `tag` and decrypt `buffer` in place.
pub fn decrypt(aes: Aes<'_>, nonce: &[u8], aad: &[u8], buffer: InOutBuf<'_, '_, u8>, tag: &[u8]) -> Result<(), Error> {
    let (b0, mut ctr) = setup(nonce, aad, buffer.len(), tag.len())?;
    let q = 15 - nonce.len();

    let mut s0 = ctr;
    encrypt_block(aes, &mut s0);

    let out = buffer.into_out_with_copied_in();
    ctr_apply(aes, &mut ctr, q, out.into());

    let mut mac = Mac::new(aes, b0);
    mac_aad(&mut mac, aad);
    mac.update(out);
    let t = mac.finish();

    let mut diff = 0u8;
    for ((tag, t), s) in tag.iter().zip(&t).zip(&s0) {
        diff |= tag ^ t ^ s;
    }
    if diff != 0 {
        out.fill(0);
        return Err(Error::InvalidSignature);
    }
    Ok(())
}
