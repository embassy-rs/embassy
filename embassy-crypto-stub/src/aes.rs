macro_rules! aes_drivers {
    (
        $bits:literal, $key:literal,
        ecb = ($ecb_feature:literal, $ecb_trait:ident, $ecb_register:ident),
        cbc = ($cbc_feature:literal, $cbc_trait:ident, $cbc_register:ident),
        ctr = ($ctr_feature:literal, $ctr_trait:ident, $ctr_register:ident),
        gcm = ($gcm_feature:literal, $gcm_trait:ident, $gcm_register:ident),
        ccm = ($ccm_feature:literal, $ccm_trait:ident, $ccm_register:ident),
        cmac = ($cmac_feature:literal, $cmac_trait:ident, $cmac_register:ident),
    ) => {
        #[cfg(feature = $ecb_feature)]
        mod $ecb_register {
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$ecb_trait for Driver {
                type Context = ();

                fn init(_: &[u8; $key]) -> Self::Context {
                    unreachable!()
                }

                fn encrypt_blocks(_: &Self::Context, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }

                fn decrypt_blocks(_: &Self::Context, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }
            }

            embassy_crypto::$ecb_register!(Driver);
        }

        #[cfg(feature = $cbc_feature)]
        mod $cbc_register {
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$cbc_trait for Driver {
                type EncryptContext = ();
                type DecryptContext = ();

                fn encrypt_init(_: &[u8; $key], _: &[u8; 16]) -> Self::EncryptContext {
                    unreachable!()
                }

                fn decrypt_init(_: &[u8; $key], _: &[u8; 16]) -> Self::DecryptContext {
                    unreachable!()
                }

                fn encrypt_blocks(_: &mut Self::EncryptContext, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }

                fn decrypt_blocks(_: &mut Self::DecryptContext, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }
            }

            embassy_crypto::$cbc_register!(Driver);
        }

        #[cfg(feature = $ctr_feature)]
        mod $ctr_register {
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$ctr_trait for Driver {
                type Context = ();

                fn init(_: &[u8; $key], _: &[u8; 16]) -> Self::Context {
                    unreachable!()
                }

                fn apply_keystream(_: &mut Self::Context, _: InOutBuf<'_, '_, u8>) {
                    unreachable!()
                }
            }

            embassy_crypto::$ctr_register!(Driver);
        }

        #[cfg(feature = $gcm_feature)]
        mod $gcm_register {
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$gcm_trait for Driver {
                type Context = ();

                fn init(_: &[u8; $key]) -> Self::Context {
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

            embassy_crypto::$gcm_register!(Driver);
        }

        #[cfg(feature = $ccm_feature)]
        mod $ccm_register {
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$ccm_trait for Driver {
                type Context = ();

                fn init(_: &[u8; $key]) -> Self::Context {
                    unreachable!()
                }

                fn encrypt(
                    _: &Self::Context,
                    _: &[u8],
                    _: &[u8],
                    _: InOutBuf<'_, '_, u8>,
                    _: &mut [u8],
                ) -> Result<(), Error> {
                    unreachable!()
                }

                fn decrypt(
                    _: &Self::Context,
                    _: &[u8],
                    _: &[u8],
                    _: InOutBuf<'_, '_, u8>,
                    _: &[u8],
                ) -> Result<(), Error> {
                    unreachable!()
                }
            }

            embassy_crypto::$ccm_register!(Driver);
        }

        #[cfg(feature = $cmac_feature)]
        mod $cmac_register {
            struct Driver;

            impl embassy_crypto::driver::$cmac_trait for Driver {
                type Context = [u8; 16];

                fn init(_: &[u8; $key]) -> Self::Context {
                    unreachable!()
                }

                fn update(_: &mut Self::Context, _: &[u8]) {
                    unreachable!()
                }

                fn finalize(_: Self::Context, _: &mut [u8; 16]) {
                    unreachable!()
                }

                fn reset(_: &mut Self::Context) {
                    unreachable!()
                }
            }

            embassy_crypto::$cmac_register!(Driver);
        }
    };
}

aes_drivers!(
    128,
    16,
    ecb = ("embassy-crypto-aes128-ecb", Aes128Ecb, aes128_ecb_impl),
    cbc = ("embassy-crypto-aes128-cbc", Aes128Cbc, aes128_cbc_impl),
    ctr = ("embassy-crypto-aes128-ctr", Aes128Ctr, aes128_ctr_impl),
    gcm = ("embassy-crypto-aes128-gcm", Aes128Gcm, aes128_gcm_impl),
    ccm = ("embassy-crypto-aes128-ccm", Aes128Ccm, aes128_ccm_impl),
    cmac = ("embassy-crypto-aes128-cmac", Aes128Cmac, aes128_cmac_impl),
);

aes_drivers!(
    256,
    32,
    ecb = ("embassy-crypto-aes256-ecb", Aes256Ecb, aes256_ecb_impl),
    cbc = ("embassy-crypto-aes256-cbc", Aes256Cbc, aes256_cbc_impl),
    ctr = ("embassy-crypto-aes256-ctr", Aes256Ctr, aes256_ctr_impl),
    gcm = ("embassy-crypto-aes256-gcm", Aes256Gcm, aes256_gcm_impl),
    ccm = ("embassy-crypto-aes256-ccm", Aes256Ccm, aes256_ccm_impl),
    cmac = ("embassy-crypto-aes256-cmac", Aes256Cmac, aes256_cmac_impl),
);
