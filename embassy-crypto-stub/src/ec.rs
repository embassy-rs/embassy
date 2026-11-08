macro_rules! curve_drivers {
    (
        $curve:ident, $n:literal,
        scalar = $dscalar:ident, point = $dpoint:ident, signature = $dsig:ident,
        arith = ($arith_feature:literal, $arith_trait:ident, $arith_register:ident),
        ecdh = ($ecdh_feature:literal, $ecdh_trait:ident, $ecdh_register:ident),
        ecdsa = ($ecdsa_feature:literal, $ecdsa_trait:ident, $ecdsa_register:ident),
    ) => {
        mod $curve {
            #[cfg(feature = $arith_feature)]
            mod arith {
                use embassy_crypto::driver::{$dpoint, $dscalar};

                struct Driver;

                impl embassy_crypto::driver::$arith_trait for Driver {
                    type Point = [u8; 3 * $n];

                    fn scalar_add(_: &$dscalar, _: &$dscalar) -> $dscalar {
                        unreachable!()
                    }

                    fn scalar_sub(_: &$dscalar, _: &$dscalar) -> $dscalar {
                        unreachable!()
                    }

                    fn scalar_mul(_: &$dscalar, _: &$dscalar) -> $dscalar {
                        unreachable!()
                    }

                    fn scalar_invert(_: &$dscalar) -> $dscalar {
                        unreachable!()
                    }

                    fn point_identity() -> Self::Point {
                        unreachable!()
                    }

                    fn point_from_affine(_: &$dpoint) -> Option<Self::Point> {
                        unreachable!()
                    }

                    fn point_from_affine_unchecked(_: &$dpoint) -> Self::Point {
                        unreachable!()
                    }

                    fn point_to_affine(_: &Self::Point) -> Option<$dpoint> {
                        unreachable!()
                    }

                    fn point_decompress(_: &[u8; $n], _: bool) -> Option<$dpoint> {
                        unreachable!()
                    }

                    fn point_is_identity(_: &Self::Point) -> bool {
                        unreachable!()
                    }

                    fn point_neg(_: &Self::Point) -> Self::Point {
                        unreachable!()
                    }

                    fn point_add(_: &Self::Point, _: &Self::Point) -> Self::Point {
                        unreachable!()
                    }

                    fn point_mul(_: &$dscalar, _: &Self::Point) -> Self::Point {
                        unreachable!()
                    }

                    fn point_mul_base(_: &$dscalar) -> Self::Point {
                        unreachable!()
                    }

                    fn point_lincomb(_: &$dscalar, _: &Self::Point, _: &$dscalar, _: &Self::Point) -> Self::Point {
                        unreachable!()
                    }

                    fn point_lincomb_vartime(
                        _: &$dscalar,
                        _: &Self::Point,
                        _: &$dscalar,
                        _: &Self::Point,
                    ) -> Self::Point {
                        unreachable!()
                    }
                }

                embassy_crypto::$arith_register!(Driver);
            }

            #[cfg(feature = $ecdh_feature)]
            mod ecdh {
                use embassy_crypto::Error;
                use embassy_crypto::driver::{$dpoint, $dscalar};

                struct Driver;

                impl embassy_crypto::driver::$ecdh_trait for Driver {
                    fn public_key(_: &$dscalar) -> Result<$dpoint, Error> {
                        unreachable!()
                    }

                    fn shared_secret(_: &$dscalar, _: &$dpoint) -> Result<[u8; $n], Error> {
                        unreachable!()
                    }
                }

                embassy_crypto::$ecdh_register!(Driver);
            }

            #[cfg(feature = $ecdsa_feature)]
            mod ecdsa_driver {
                use embassy_crypto::Error;
                use embassy_crypto::driver::{$dpoint, $dscalar, $dsig};

                struct Driver;

                impl embassy_crypto::driver::$ecdsa_trait for Driver {
                    fn public_key(_: &$dscalar) -> Result<$dpoint, Error> {
                        unreachable!()
                    }

                    fn sign(_: &$dscalar, _: &[u8; $n]) -> Result<$dsig, Error> {
                        unreachable!()
                    }

                    fn verify(_: &$dpoint, _: &[u8; $n], _: &$dsig) -> Result<(), Error> {
                        unreachable!()
                    }
                }

                embassy_crypto::$ecdsa_register!(Driver);
            }
        }
    };
}

#[cfg(any(
    feature = "embassy-crypto-p256-arith",
    feature = "embassy-crypto-p256-ecdh",
    feature = "embassy-crypto-p256-ecdsa"
))]
curve_drivers!(
    p256,
    32,
    scalar = P256Scalar,
    point = P256Point,
    signature = P256Signature,
    arith = ("embassy-crypto-p256-arith", P256Arith, p256_arith_impl),
    ecdh = ("embassy-crypto-p256-ecdh", P256Ecdh, p256_ecdh_impl),
    ecdsa = ("embassy-crypto-p256-ecdsa", P256Ecdsa, p256_ecdsa_impl),
);

#[cfg(any(
    feature = "embassy-crypto-p384-arith",
    feature = "embassy-crypto-p384-ecdh",
    feature = "embassy-crypto-p384-ecdsa"
))]
curve_drivers!(
    p384,
    48,
    scalar = P384Scalar,
    point = P384Point,
    signature = P384Signature,
    arith = ("embassy-crypto-p384-arith", P384Arith, p384_arith_impl),
    ecdh = ("embassy-crypto-p384-ecdh", P384Ecdh, p384_ecdh_impl),
    ecdsa = ("embassy-crypto-p384-ecdsa", P384Ecdsa, p384_ecdsa_impl),
);
