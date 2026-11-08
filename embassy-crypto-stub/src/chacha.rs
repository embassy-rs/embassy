macro_rules! impl_stream {
    ($feature:literal, $module:ident, $trait:ident, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $module {
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = ();

                fn init(_: &[u8; 32], _: &[u8; 12], _: u32) -> Self::Context {
                    unreachable!()
                }

                fn apply_keystream(_: &mut Self::Context, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_aead {
    ($feature:literal, $module:ident, $trait:ident, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $module {
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = ();

                fn init(_: &[u8; 32]) -> Self::Context {
                    unreachable!()
                }

                fn encrypt(
                    _: &Self::Context,
                    _: &[u8; 12],
                    _: &[u8],
                    _: InOutBuf<'_, '_, u8>,
                    _: &mut [u8; 16],
                ) -> Result<(), Error> {
                    unreachable!()
                }

                fn decrypt(
                    _: &Self::Context,
                    _: &[u8; 12],
                    _: &[u8],
                    _: InOutBuf<'_, '_, u8>,
                    _: &[u8; 16],
                ) -> Result<(), Error> {
                    unreachable!()
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

impl_stream!("embassy-crypto-chacha8", chacha8, ChaCha8, chacha8_impl);
impl_stream!("embassy-crypto-chacha12", chacha12, ChaCha12, chacha12_impl);
impl_stream!("embassy-crypto-chacha20", chacha20, ChaCha20, chacha20_impl);

impl_aead!(
    "embassy-crypto-chacha8-poly1305",
    chacha8_poly1305,
    ChaCha8Poly1305,
    chacha8_poly1305_impl
);
impl_aead!(
    "embassy-crypto-chacha12-poly1305",
    chacha12_poly1305,
    ChaCha12Poly1305,
    chacha12_poly1305_impl
);
impl_aead!(
    "embassy-crypto-chacha20-poly1305",
    chacha20_poly1305,
    ChaCha20Poly1305,
    chacha20_poly1305_impl
);
