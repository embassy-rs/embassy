/// Converts an `embassy_crypto` in/out buffer into the RustCrypto one re-exported by `$path`.
macro_rules! inout_fn {
    ($path:path) => {
        fn inout<'i, 'o>(buf: InOutBuf<'i, 'o, u8>) -> $path {
            let len = buf.len();
            let (in_ptr, out_ptr) = buf.into_raw();
            // SAFETY: `InOutBuf` upholds the same validity invariants.
            unsafe { <$path>::from_raw(in_ptr, out_ptr, len) }
        }
    };
}

macro_rules! impl_stream {
    ($feature:literal, $module:ident, $trait:ident, $rounds:ty, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $module {
            use chacha20::ChaChaCore;
            use chacha20::variants::Ietf;
            use cipher::{KeyIvInit, StreamCipherCore};
            use embassy_crypto::driver::InOutBuf;

            inout_fn!(cipher::inout::InOutBuf<'i, 'o, u8>);

            struct Driver;

            const BLOCK: usize = 64;

            /// Key, nonce, next block counter and unused keystream.
            #[derive(Clone)]
            pub struct Context {
                key: [u8; 32],
                nonce: [u8; 12],
                counter: u32,
                /// Unused keystream bytes are stored at the end of this buffer.
                ks: [u8; BLOCK],
                ks_len: u8,
            }

            impl Context {
                /// Writes `blocks` keystream blocks from `counter` on. The counter wraps modulo
                /// 2^32, which the `chacha20` stream wrapper refuses to do, so the core is driven
                /// directly, in runs that never cross the wrap.
                fn keystream(&mut self, mut buf: cipher::inout::InOutBuf<'_, '_, u8>) {
                    while !buf.is_empty() {
                        let blocks = buf.len() / BLOCK;
                        let until_wrap = (u32::MAX - self.counter) as usize + 1;
                        let n = blocks.min(until_wrap);
                        let (head, tail) = buf.split_at(n * BLOCK);
                        buf = tail;
                        let mut core = ChaChaCore::<$rounds, Ietf>::new((&self.key).into(), (&self.nonce).into());
                        core.set_block_pos(self.counter);
                        let (blocks, _) = head.into_chunks();
                        core.apply_keystream_blocks_inout(blocks);
                        self.counter = self.counter.wrapping_add(n as u32);
                    }
                }
            }

            impl embassy_crypto::driver::$trait for Driver {
                type Context = Context;

                fn init(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Context {
                    Context {
                        key: *key,
                        nonce: *nonce,
                        counter,
                        ks: [0; BLOCK],
                        ks_len: 0,
                    }
                }

                fn apply_keystream(ctx: &mut Context, buf: InOutBuf<'_, '_, u8>) {
                    let mut buf = inout(buf);

                    // Leftover keystream from the previous call.
                    let n = (ctx.ks_len as usize).min(buf.len());
                    let (mut head, tail) = buf.split_at(n);
                    head.xor_in2out(&ctx.ks[BLOCK - ctx.ks_len as usize..][..n]);
                    ctx.ks_len -= n as u8;
                    buf = tail;

                    // Whole blocks.
                    let full = buf.len() / BLOCK * BLOCK;
                    let (head, tail) = buf.split_at(full);
                    ctx.keystream(head);
                    buf = tail;

                    // A partial block: generate one and keep the rest.
                    if !buf.is_empty() {
                        let mut ks = [0u8; BLOCK];
                        let mut zero = [0u8; BLOCK];
                        ctx.keystream(cipher::inout::InOutBuf::new(&zero, &mut ks).unwrap());
                        let rem = buf.len();
                        buf.xor_in2out(&ks[..rem]);
                        zero[rem..].copy_from_slice(&ks[rem..]);
                        ctx.ks = zero;
                        ctx.ks_len = (BLOCK - rem) as u8;
                    }
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_aead {
    ($feature:literal, $module:ident, $trait:ident, $cipher:ty, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $module {
            use aead::{AeadInOut, KeyInit};
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            inout_fn!(aead::inout::InOutBuf<'i, 'o, u8>);

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = [u8; 32];

                fn init(key: &[u8; 32]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    ctx: &Self::Context,
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8; 16],
                ) -> Result<(), Error> {
                    let t = <$cipher>::new(ctx.into())
                        .encrypt_inout_detached(nonce.into(), aad, inout(buffer))
                        .map_err(|_| Error::InvalidInput)?;
                    tag.copy_from_slice(&t);
                    Ok(())
                }

                fn decrypt(
                    ctx: &Self::Context,
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8; 16],
                ) -> Result<(), Error> {
                    <$cipher>::new(ctx.into())
                        .decrypt_inout_detached(nonce.into(), aad, inout(buffer), tag.into())
                        .map_err(|_| Error::InvalidSignature)
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

impl_stream!("embassy-crypto-chacha8", chacha8, ChaCha8, chacha20::R8, chacha8_impl);
impl_stream!(
    "embassy-crypto-chacha12",
    chacha12,
    ChaCha12,
    chacha20::R12,
    chacha12_impl
);
impl_stream!(
    "embassy-crypto-chacha20",
    chacha20,
    ChaCha20,
    chacha20::R20,
    chacha20_impl
);

impl_aead!(
    "embassy-crypto-chacha8-poly1305",
    chacha8_poly1305,
    ChaCha8Poly1305,
    chacha20poly1305::ChaCha8Poly1305,
    chacha8_poly1305_impl
);
impl_aead!(
    "embassy-crypto-chacha12-poly1305",
    chacha12_poly1305,
    ChaCha12Poly1305,
    chacha20poly1305::ChaCha12Poly1305,
    chacha12_poly1305_impl
);
impl_aead!(
    "embassy-crypto-chacha20-poly1305",
    chacha20_poly1305,
    ChaCha20Poly1305,
    chacha20poly1305::ChaCha20Poly1305,
    chacha20_poly1305_impl
);
