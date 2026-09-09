macro_rules! curve_drivers {
    (
        $curve:ident, $curve_ty:ident, $n:literal,
        scalar = $dscalar:ident, point = $dpoint:ident, signature = $dsig:ident,
        arith = ($arith_feature:literal, $arith_trait:ident, $arith_register:ident),
        ecdh = ($ecdh_feature:literal, $ecdh_trait:ident, $ecdh_register:ident),
        ecdsa = ($ecdsa_feature:literal, $ecdsa_trait:ident, $ecdsa_register:ident),
    ) => {
        mod $curve {
            #![allow(dead_code)]

            use elliptic_curve::PrimeField;
            use elliptic_curve::point::AffineCoordinates;
            use elliptic_curve::sec1::{FromSec1Point, Sec1Point};
            use embassy_crypto::Error;
            use embassy_crypto::driver::{$dpoint, $dscalar};
            use $curve::{AffinePoint, FieldBytes, NonZeroScalar, ProjectivePoint, Scalar};

            /// `None` if not canonical.
            pub(super) fn scalar(k: &$dscalar) -> Option<Scalar> {
                Scalar::from_repr(FieldBytes::from(k.0)).into()
            }

            pub(super) fn nonzero_scalar(k: &$dscalar) -> Result<NonZeroScalar, Error> {
                let s = scalar(k).ok_or(Error::InvalidKey)?;
                Option::from(NonZeroScalar::new(s)).ok_or(Error::InvalidKey)
            }

            pub(super) fn scalar_bytes(s: &Scalar) -> $dscalar {
                $dscalar(s.to_repr().into())
            }

            /// `None` if not on the curve.
            pub(super) fn point(p: &$dpoint) -> Option<AffinePoint> {
                let encoded = Sec1Point::<$curve::$curve_ty>::from_affine_coordinates(
                    &FieldBytes::from(p.x),
                    &FieldBytes::from(p.y),
                    false,
                );
                AffinePoint::from_sec1_point(&encoded).into()
            }

            /// `None` for the identity.
            pub(super) fn point_bytes(p: &ProjectivePoint) -> Option<$dpoint> {
                let affine = p.to_affine();
                if bool::from(affine.is_identity()) {
                    return None;
                }
                Some($dpoint {
                    x: affine.x().into(),
                    y: affine.y().into(),
                })
            }

            #[cfg(feature = $arith_feature)]
            mod arith {
                use elliptic_curve::group::Group;
                use elliptic_curve::ops::LinearCombination;
                use embassy_crypto::driver::{$dpoint, $dscalar};
                use $curve::{ProjectivePoint, Scalar};

                use super::{point, point_bytes, scalar, scalar_bytes};

                struct Driver;

                fn terms(
                    a: &$dscalar,
                    p: &ProjectivePoint,
                    b: &$dscalar,
                    q: &ProjectivePoint,
                ) -> [(ProjectivePoint, Scalar); 2] {
                    [(*p, scalar(a).unwrap()), (*q, scalar(b).unwrap())]
                }

                impl embassy_crypto::driver::$arith_trait for Driver {
                    type Point = ProjectivePoint;

                    fn scalar_add(a: &$dscalar, b: &$dscalar) -> $dscalar {
                        scalar_bytes(&(scalar(a).unwrap() + scalar(b).unwrap()))
                    }

                    fn scalar_sub(a: &$dscalar, b: &$dscalar) -> $dscalar {
                        scalar_bytes(&(scalar(a).unwrap() - scalar(b).unwrap()))
                    }

                    fn scalar_mul(a: &$dscalar, b: &$dscalar) -> $dscalar {
                        scalar_bytes(&(scalar(a).unwrap() * scalar(b).unwrap()))
                    }

                    fn scalar_invert(a: &$dscalar) -> $dscalar {
                        scalar_bytes(&scalar(a).unwrap().invert().unwrap())
                    }

                    fn point_identity() -> ProjectivePoint {
                        ProjectivePoint::IDENTITY
                    }

                    fn point_from_affine(p: &$dpoint) -> Option<ProjectivePoint> {
                        point(p).map(Into::into)
                    }

                    fn point_from_affine_unchecked(p: &$dpoint) -> ProjectivePoint {
                        point(p).unwrap().into()
                    }

                    fn point_to_affine(p: &ProjectivePoint) -> Option<$dpoint> {
                        point_bytes(p)
                    }

                    fn point_is_identity(p: &ProjectivePoint) -> bool {
                        bool::from(p.is_identity())
                    }

                    fn point_neg(p: &ProjectivePoint) -> ProjectivePoint {
                        -*p
                    }

                    fn point_add(p: &ProjectivePoint, q: &ProjectivePoint) -> ProjectivePoint {
                        p + q
                    }

                    fn point_mul(k: &$dscalar, p: &ProjectivePoint) -> ProjectivePoint {
                        *p * scalar(k).unwrap()
                    }

                    fn point_mul_base(k: &$dscalar) -> ProjectivePoint {
                        ProjectivePoint::mul_by_generator(&scalar(k).unwrap())
                    }

                    fn point_lincomb(
                        a: &$dscalar,
                        p: &ProjectivePoint,
                        b: &$dscalar,
                        q: &ProjectivePoint,
                    ) -> ProjectivePoint {
                        <ProjectivePoint as LinearCombination<[(ProjectivePoint, Scalar); 2]>>::lincomb(&terms(
                            a, p, b, q,
                        ))
                    }

                    fn point_lincomb_vartime(
                        a: &$dscalar,
                        p: &ProjectivePoint,
                        b: &$dscalar,
                        q: &ProjectivePoint,
                    ) -> ProjectivePoint {
                        <ProjectivePoint as LinearCombination<[(ProjectivePoint, Scalar); 2]>>::lincomb_vartime(&terms(
                            a, p, b, q,
                        ))
                    }
                }

                embassy_crypto::$arith_register!(Driver);
            }

            #[cfg(feature = $ecdh_feature)]
            mod ecdh {
                use elliptic_curve::group::Group;
                use embassy_crypto::Error;
                use embassy_crypto::driver::{$dpoint, $dscalar};
                use $curve::ProjectivePoint;

                use super::{nonzero_scalar, point, point_bytes};

                struct Driver;

                impl embassy_crypto::driver::$ecdh_trait for Driver {
                    fn public_key(k: &$dscalar) -> Result<$dpoint, Error> {
                        let k = nonzero_scalar(k)?;
                        Ok(point_bytes(&ProjectivePoint::mul_by_generator(&k)).unwrap())
                    }

                    fn shared_secret(k: &$dscalar, peer: &$dpoint) -> Result<[u8; $n], Error> {
                        let k = nonzero_scalar(k)?;
                        let peer = point(peer).ok_or(Error::InvalidKey)?;
                        let secret = elliptic_curve::ecdh::diffie_hellman(&k, &peer);
                        let mut out = [0u8; $n];
                        out.copy_from_slice(secret.raw_secret_bytes());
                        Ok(out)
                    }
                }

                embassy_crypto::$ecdh_register!(Driver);
            }

            #[cfg(feature = $ecdsa_feature)]
            mod ecdsa_driver {
                use ecdsa::signature::hazmat::PrehashVerifier;
                use elliptic_curve::group::Group;
                use embassy_crypto::driver::{$dpoint, $dscalar, $dsig};
                use embassy_crypto::{Error, rng_fill_bytes};
                use $curve::{FieldBytes, ProjectivePoint};

                use super::{nonzero_scalar, point, point_bytes};

                struct Driver;

                impl embassy_crypto::driver::$ecdsa_trait for Driver {
                    fn public_key(k: &$dscalar) -> Result<$dpoint, Error> {
                        let k = nonzero_scalar(k)?;
                        Ok(point_bytes(&ProjectivePoint::mul_by_generator(&k)).unwrap())
                    }

                    fn sign(k: &$dscalar, digest: &[u8; $n]) -> Result<$dsig, Error> {
                        let d = nonzero_scalar(k)?;
                        let mut nonce_bytes = $dscalar([0u8; $n]);
                        let nonce = loop {
                            rng_fill_bytes(&mut nonce_bytes.0);
                            if let Ok(n) = nonzero_scalar(&nonce_bytes) {
                                break n;
                            }
                        };
                        let (sig, _) = ecdsa::hazmat::sign_prehashed::<$curve::$curve_ty>(&d, &nonce, digest)
                            .map_err(|_| Error::InvalidInput)?;
                        let bytes = sig.normalize_s().to_bytes();
                        let mut r = [0u8; $n];
                        let mut s = [0u8; $n];
                        r.copy_from_slice(&bytes[..$n]);
                        s.copy_from_slice(&bytes[$n..]);
                        Ok($dsig {
                            r: $dscalar(r),
                            s: $dscalar(s),
                        })
                    }

                    fn verify(q: &$dpoint, digest: &[u8; $n], sig: &$dsig) -> Result<(), Error> {
                        let q = point(q).ok_or(Error::InvalidKey)?;
                        let vk = $curve::ecdsa::VerifyingKey::from_affine(q).map_err(|_| Error::InvalidKey)?;
                        let signature = $curve::ecdsa::Signature::from_scalars(
                            FieldBytes::from(sig.r.0),
                            FieldBytes::from(sig.s.0),
                        )
                        .map_err(|_| Error::InvalidSignature)?;
                        vk.verify_prehash(digest, &signature)
                            .map_err(|_| Error::InvalidSignature)
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
    NistP256,
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
    NistP384,
    48,
    scalar = P384Scalar,
    point = P384Point,
    signature = P384Signature,
    arith = ("embassy-crypto-p384-arith", P384Arith, p384_arith_impl),
    ecdh = ("embassy-crypto-p384-ecdh", P384Ecdh, p384_ecdh_impl),
    ecdsa = ("embassy-crypto-p384-ecdsa", P384Ecdsa, p384_ecdsa_impl),
);
