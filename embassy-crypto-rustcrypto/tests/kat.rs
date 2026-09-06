//! Known-answer and round-trip tests for every driver, through the `embassy-crypto` API.

#![cfg(feature = "all")]

// A crate that is never named is not linked, so the drivers must be pulled in explicitly.
use embassy_crypto::{
    Aes128, Aes128CbcDecrypt, Aes128CbcEncrypt, Aes128Ccm, Aes128Cmac, Aes128Ctr, Aes128Gcm, Aes256, Aes256CbcDecrypt,
    Aes256CbcEncrypt, Aes256Ccm, Aes256Cmac, Aes256Ctr, Aes256Gcm, Error, HmacSha1, HmacSha224, HmacSha256, HmacSha384,
    HmacSha512, HmacSha512_224, HmacSha512_256, Md5, Rng, Sha1, Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256,
};
use embassy_crypto_rustcrypto as _;
use hex_literal::hex;

/// Deterministic xorshift PRNG: the tests need arbitrary values, not secure ones.
struct TestRng(u64);

impl TestRng {
    fn new() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}

impl Rng for TestRng {
    fn fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for b in buf {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = self.0 as u8;
        }
        Ok(())
    }
}

fn pattern(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i * 7 + 3) as u8).collect()
}

// ===========================================================================
// Digests
// ===========================================================================

#[test]
fn digests() {
    assert_eq!(Md5::digest(b"abc"), hex!("900150983cd24fb0d6963f7d28e17f72"));
    assert_eq!(Sha1::digest(b"abc"), hex!("a9993e364706816aba3e25717850c26c9cd0d89d"));
    assert_eq!(
        Sha224::digest(b"abc"),
        hex!("23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7")
    );
    assert_eq!(
        Sha256::digest(b"abc"),
        hex!("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
    assert_eq!(
        Sha384::digest(b"abc"),
        hex!("cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7")
    );
    assert_eq!(
        Sha512::digest(b"abc"),
        hex!(
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a"
            "2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        )
    );
    assert_eq!(
        Sha512_224::digest(b"abc"),
        hex!("4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa")
    );
    assert_eq!(
        Sha512_256::digest(b"abc"),
        hex!("53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23")
    );
}

#[test]
fn digest_incremental_and_clone() {
    let data = pattern(1000);
    let expected = Sha256::digest(&data);

    let mut h = Sha256::new();
    for chunk in data.chunks(37) {
        h.update(chunk);
    }
    // A clone forks the state.
    let fork = h.clone();
    h.update(b"extra");
    assert_eq!(fork.finalize(), expected);
    assert_ne!(h.finalize(), expected);
}

// ===========================================================================
// HMAC
// ===========================================================================

#[test]
fn hmac() {
    // RFC 4231 test case 2 / RFC 2202 test case 2.
    let key = b"Jefe";
    let data = b"what do ya want for nothing?";
    assert_eq!(
        HmacSha1::mac(key, data),
        hex!("effcdf6ae5eb2fa2d27416d5f184df9c259a7c79")
    );
    assert_eq!(
        HmacSha224::mac(key, data),
        hex!("a30e01098bc6dbbf45690f3a7e9e6d0f8bbea2a39e6148008fd05e44")
    );
    assert_eq!(
        HmacSha256::mac(key, data),
        hex!("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843")
    );
    assert_eq!(
        HmacSha384::mac(key, data),
        hex!("af45d2e376484031617f78d2b58a6b1b9c7ef464f5a01b47e42ec3736322445e8e2240ca5e69e2c78b3239ecfab21649")
    );
    assert_eq!(
        HmacSha512::mac(key, data),
        hex!(
            "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea250554"
            "9758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"
        )
    );

    // The 512/224 and 512/256 variants have no RFC vectors; check them against `hmac` directly.
    let mut m = HmacSha512_224::new(key);
    m.update(data);
    assert_eq!(m.finalize(), hmac_ref::<sha2::Sha512_224>(key, data)[..]);
    let mut m = HmacSha512_256::new(key);
    m.update(data);
    assert_eq!(m.finalize(), hmac_ref::<sha2::Sha512_256>(key, data)[..]);

    // Long keys are hashed.
    let long_key = pattern(200);
    assert_eq!(
        HmacSha256::mac(&long_key, data),
        hmac_ref::<sha2::Sha256>(&long_key, data)[..]
    );

    // Verification.
    let tag = HmacSha256::mac(key, data);
    let mut m = HmacSha256::new(key);
    m.update(data);
    m.verify(&tag).unwrap();
    let mut m = HmacSha256::new(key);
    m.update(data);
    m.verify(&tag[..16]).unwrap();
    let mut m = HmacSha256::new(key);
    m.update(data);
    assert_eq!(m.verify(&[]), Err(Error::InvalidSignature));
    let mut bad = tag;
    bad[3] ^= 1;
    let mut m = HmacSha256::new(key);
    m.update(data);
    assert_eq!(m.verify(&bad), Err(Error::InvalidSignature));
}

fn hmac_ref<D: hmac::EagerHash>(key: &[u8], data: &[u8]) -> Vec<u8> {
    use digest::{KeyInit, Mac};
    let mut m = hmac::Hmac::<D>::new_from_slice(key).unwrap();
    m.update(data);
    m.finalize().into_bytes().to_vec()
}

// ===========================================================================
// AES
// ===========================================================================

const KEY128: [u8; 16] = hex!("2b7e151628aed2a6abf7158809cf4f3c");
const NIST_PT: [u8; 64] = hex!(
    "6bc1bee22e409f96e93d7e117393172a"
    "ae2d8a571e03ac9c9eb76fac45af8e51"
    "30c81c46a35ce411e5fbc1191a0a52ef"
    "f69f2445df4f9b17ad2b417be66c3710"
);

#[test]
fn aes_ecb() {
    // FIPS 197 appendix C.
    let mut block = hex!("00112233445566778899aabbccddeeff");
    let aes = Aes128::new(&hex!("000102030405060708090a0b0c0d0e0f"));
    aes.encrypt_block(&mut block);
    assert_eq!(block, hex!("69c4e0d86a7b0430d8cdb78070b4c55a"));
    aes.decrypt_block(&mut block);
    assert_eq!(block, hex!("00112233445566778899aabbccddeeff"));

    let aes = Aes256::new(&hex!(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    ));
    aes.encrypt_block(&mut block);
    assert_eq!(block, hex!("8ea2b7ca516745bfeafc49904b496089"));
    aes.decrypt_block(&mut block);
    assert_eq!(block, hex!("00112233445566778899aabbccddeeff"));

    // Multi-block, in place and separate buffers.
    let aes = Aes128::new(&KEY128);
    let mut buf = NIST_PT;
    aes.encrypt_blocks(&mut buf).unwrap();
    let mut out = [0u8; 64];
    aes.encrypt_blocks_to(&NIST_PT, &mut out).unwrap();
    assert_eq!(buf, out);
    assert_eq!(&buf[..16], hex!("3ad77bb40d7a3660a89ecaf32466ef97"));
    aes.decrypt_blocks(&mut buf).unwrap();
    assert_eq!(buf, NIST_PT);
    aes.decrypt_blocks_to(&out.clone(), &mut out).unwrap();
    assert_eq!(out, NIST_PT);

    assert_eq!(aes.encrypt_blocks(&mut [0u8; 15]), Err(Error::InvalidInput));
    assert_eq!(
        aes.encrypt_blocks_to(&[0u8; 16], &mut [0u8; 32]),
        Err(Error::InvalidInput)
    );
}

#[test]
fn aes_cbc() {
    // NIST SP 800-38A F.2.1.
    let iv = hex!("000102030405060708090a0b0c0d0e0f");
    let expected = hex!(
        "7649abac8119b246cee98e9b12e9197d"
        "5086cb9b507219ee95db113a917678b2"
        "73bed6b8e3c1743b7116e69e22229516"
        "3ff1caa1681fac09120eca307586e1a7"
    );

    let mut buf = NIST_PT;
    Aes128CbcEncrypt::new(&KEY128, &iv).encrypt(&mut buf).unwrap();
    assert_eq!(buf, expected);

    // Chaining across calls.
    let mut enc = Aes128CbcEncrypt::new(&KEY128, &iv);
    let mut buf = NIST_PT;
    enc.encrypt(&mut buf[..16]).unwrap();
    enc.encrypt(&mut buf[16..48]).unwrap();
    enc.encrypt(&mut buf[48..]).unwrap();
    assert_eq!(buf, expected);

    let mut dec = Aes128CbcDecrypt::new(&KEY128, &iv);
    dec.decrypt(&mut buf[..32]).unwrap();
    dec.decrypt(&mut buf[32..]).unwrap();
    assert_eq!(buf, NIST_PT);

    let mut out = [0u8; 64];
    Aes128CbcEncrypt::new(&KEY128, &iv)
        .encrypt_to(&NIST_PT, &mut out)
        .unwrap();
    assert_eq!(out, expected);
    let mut pt = [0u8; 64];
    Aes128CbcDecrypt::new(&KEY128, &iv).decrypt_to(&out, &mut pt).unwrap();
    assert_eq!(pt, NIST_PT);

    // AES-256 round trip.
    let key = pattern(32).try_into().unwrap();
    let mut buf = NIST_PT;
    Aes256CbcEncrypt::new(&key, &iv).encrypt(&mut buf).unwrap();
    assert_ne!(buf, NIST_PT);
    Aes256CbcDecrypt::new(&key, &iv).decrypt(&mut buf).unwrap();
    assert_eq!(buf, NIST_PT);
}

#[test]
fn aes_ctr() {
    // NIST SP 800-38A F.5.1.
    let iv = hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
    let expected = hex!(
        "874d6191b620e3261bef6864990db6ce"
        "9806f66b7970fdff8617187bb9fffdff"
        "5ae4df3edbd5d35e5b4f09020db03eab"
        "1e031dda2fbe03d1792170a0f3009cee"
    );

    let mut buf = NIST_PT;
    Aes128Ctr::new(&KEY128, &iv).apply_keystream(&mut buf);
    assert_eq!(buf, expected);

    // Unaligned chunks keep the keystream position.
    let mut ctr = Aes128Ctr::new(&KEY128, &iv);
    let mut buf = NIST_PT;
    let mut pos = 0;
    for len in [1, 7, 16, 20, 3, 17] {
        ctr.apply_keystream(&mut buf[pos..pos + len]);
        pos += len;
    }
    assert_eq!(buf, expected);

    let mut out = [0u8; 64];
    Aes128Ctr::new(&KEY128, &iv)
        .apply_keystream_to(&NIST_PT, &mut out)
        .unwrap();
    assert_eq!(out, expected);

    let key = pattern(32).try_into().unwrap();
    let mut buf = NIST_PT;
    Aes256Ctr::new(&key, &iv).apply_keystream(&mut buf);
    Aes256Ctr::new(&key, &iv).apply_keystream(&mut buf);
    assert_eq!(buf, NIST_PT);
}

#[test]
fn aes_gcm() {
    // GCM spec test cases 1 and 2.
    let key = [0u8; 16];
    let nonce = [0u8; 12];
    let gcm = Aes128Gcm::new(&key);
    let tag = gcm.encrypt(&nonce, &[], &mut []).unwrap();
    assert_eq!(tag, hex!("58e2fccefa7e3061367f1d57a4e7455a"));

    let mut buf = [0u8; 16];
    let tag = gcm.encrypt(&nonce, &[], &mut buf).unwrap();
    assert_eq!(buf, hex!("0388dace60b6a392f328c2b971b2fe78"));
    assert_eq!(tag, hex!("ab6e47d42cec13bdf53a67b21257bddf"));
    gcm.decrypt(&nonce, &[], &mut buf, &tag).unwrap();
    assert_eq!(buf, [0u8; 16]);

    // Round trip with AAD, separate buffers, tampering.
    let key = pattern(32).try_into().unwrap();
    let nonce = pattern(12).try_into().unwrap();
    let pt = pattern(100);
    let gcm = Aes256Gcm::new(&key);
    let mut ct = vec![0u8; 100];
    let tag = gcm.encrypt_to(&nonce, b"aad", &pt, &mut ct).unwrap();
    let mut out = vec![0u8; 100];
    gcm.decrypt_to(&nonce, b"aad", &ct, &mut out, &tag).unwrap();
    assert_eq!(out, pt);
    assert_eq!(
        gcm.decrypt_to(&nonce, b"aae", &ct, &mut out, &tag),
        Err(Error::InvalidSignature)
    );
    ct[5] ^= 1;
    assert_eq!(
        gcm.decrypt_to(&nonce, b"aad", &ct, &mut out, &tag),
        Err(Error::InvalidSignature)
    );
}

#[test]
fn aes_ccm() {
    // RFC 3610 packet vector #1.
    let key = hex!("C0C1C2C3C4C5C6C7C8C9CACBCCCDCECF");
    let nonce = hex!("00000003020100A0A1A2A3A4A5");
    let aad = hex!("0001020304050607");
    let pt = hex!("08090A0B0C0D0E0F101112131415161718191A1B1C1D1E");
    let ccm = Aes128Ccm::new(&key);
    let mut buf = pt;
    let mut tag = [0u8; 8];
    ccm.encrypt(&nonce, &aad, &mut buf, &mut tag).unwrap();
    assert_eq!(buf, hex!("588C979A61C663D2F066D0C2C0F989806D5F6B61DAC384"));
    assert_eq!(tag, hex!("17E8D12CFDF926E0"));
    ccm.decrypt(&nonce, &aad, &mut buf, &tag).unwrap();
    assert_eq!(buf, pt);

    // NIST SP 800-38C example 1.
    let key = hex!("404142434445464748494a4b4c4d4e4f");
    let nonce = hex!("10111213141516");
    let aad = hex!("0001020304050607");
    let ccm = Aes128Ccm::new(&key);
    let mut buf = hex!("20212223");
    let mut tag = [0u8; 4];
    ccm.encrypt(&nonce, &aad, &mut buf, &mut tag).unwrap();
    assert_eq!(buf, hex!("7162015b"));
    assert_eq!(tag, hex!("4dac255d"));

    // Round trip, tampering, parameter validation.
    let key = pattern(32).try_into().unwrap();
    let nonce = pattern(13);
    let pt = pattern(70);
    let ccm = Aes256Ccm::new(&key);
    let mut ct = vec![0u8; 70];
    let mut tag = [0u8; 16];
    ccm.encrypt_to(&nonce, b"aad", &pt, &mut ct, &mut tag).unwrap();
    let mut out = vec![0u8; 70];
    ccm.decrypt_to(&nonce, b"aad", &ct, &mut out, &tag).unwrap();
    assert_eq!(out, pt);
    tag[0] ^= 1;
    assert_eq!(
        ccm.decrypt_to(&nonce, b"aad", &ct, &mut out, &tag),
        Err(Error::InvalidSignature)
    );
    assert_eq!(
        ccm.encrypt(&nonce[..6], b"", &mut [], &mut tag),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        ccm.encrypt(&nonce, b"", &mut [], &mut tag[..3]),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        ccm.encrypt(&nonce, b"", &mut [], &mut tag[..5]),
        Err(Error::InvalidInput)
    );
}

#[test]
fn aes_cmac() {
    // RFC 4493.
    assert_eq!(Aes128Cmac::mac(&KEY128, &[]), hex!("bb1d6929e95937287fa37d129b756746"));
    assert_eq!(
        Aes128Cmac::mac(&KEY128, &NIST_PT[..16]),
        hex!("070a16b46b4d4144f79bdd9dd04a287c")
    );
    assert_eq!(
        Aes128Cmac::mac(&KEY128, &NIST_PT[..40]),
        hex!("dfa66747de9ae63030ca32611497c827")
    );
    assert_eq!(
        Aes128Cmac::mac(&KEY128, &NIST_PT),
        hex!("51f0bebf7e3b9d92fc49741779363cfe")
    );

    let mut m = Aes128Cmac::new(&KEY128);
    m.update(&NIST_PT[..7]);
    m.update(&NIST_PT[7..40]);
    m.verify(&hex!("dfa66747de9ae63030ca32611497c827")).unwrap();

    let mut m = Aes128Cmac::new(&KEY128);
    m.update(b"garbage");
    m.reset();
    m.update(&NIST_PT[..16]);
    assert_eq!(m.finalize(), hex!("070a16b46b4d4144f79bdd9dd04a287c"));

    let key = pattern(32).try_into().unwrap();
    assert_eq!(Aes256Cmac::mac(&key, b"abc"), cmac_ref256(&key, b"abc"));
}

fn cmac_ref256(key: &[u8; 32], data: &[u8]) -> [u8; 16] {
    use digest::{KeyInit, Mac};
    let mut m = cmac::Cmac::<aes::Aes256>::new(key.into());
    m.update(data);
    m.finalize().into_bytes().into()
}

// ===========================================================================
// P-256
// ===========================================================================

#[test]
fn p256_arith() {
    use embassy_crypto::p256::{Point, Scalar};

    // RFC 6979 A.2.5: the public key of the private key `x`.
    let x = Scalar::from_bytes(&hex!(
        "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721"
    ))
    .unwrap();
    let u = Point::mul_base(&x).unwrap();
    assert_eq!(
        u.x(),
        &hex!("60FED4BA255A9D31C961EB74C6356D68C049B8923B61FA6CE669622E60F29FB6")
    );
    assert_eq!(
        u.y(),
        &hex!("7903FE1008B8BC99A41AE9E95628BC64F2F1B20C2D7E9F5177A3C294D4462299")
    );
    assert_eq!(Point::GENERATOR.mul(&x), Some(u));
    assert_eq!(Point::from_sec1(&u.to_sec1()), Ok(u));
    assert_eq!(Point::from_xy(u.x(), u.y()), Ok(u));
    assert_eq!(Point::from_xy(u.x(), &[1u8; 32]), Err(Error::InvalidKey));

    // The generator constant is on the curve.
    assert_eq!(
        Point::from_xy(Point::GENERATOR.x(), Point::GENERATOR.y()),
        Ok(Point::GENERATOR)
    );

    let one = Scalar::from_bytes(&{
        let mut b = [0u8; 32];
        b[31] = 1;
        b
    })
    .unwrap();
    let two = one.add(&one);
    assert_eq!(Point::mul_base(&one), Some(Point::GENERATOR));
    assert_eq!(Point::mul_base(&two), Point::GENERATOR.add(&Point::GENERATOR));
    assert_eq!(Point::mul_base(&Scalar::ZERO), None);
    assert_eq!(Scalar::from_bytes(&embassy_crypto::p256::ORDER), Err(Error::InvalidKey));

    // Field identities.
    let mut rng = TestRng::new();
    let a = embassy_crypto::p256::SecretKey::generate(&mut rng).unwrap().to_scalar();
    let b = embassy_crypto::p256::SecretKey::generate(&mut rng).unwrap().to_scalar();
    assert_eq!(a.add(&b).sub(&b), a);
    assert_eq!(a.mul(&a.invert().unwrap()), one);
    assert_eq!(Scalar::ZERO.invert(), None);
    assert_eq!(a.sub(&a), Scalar::ZERO);

    // lincomb == mul + add; cancellation gives the identity.
    let p = Point::mul_base(&b).unwrap();
    let expected = Point::mul_base(&a).unwrap().add(&p.mul(&b).unwrap());
    assert_eq!(Point::lincomb(&a, &Point::GENERATOR, &b, &p), expected);
    let neg_b = Scalar::ZERO.sub(&b);
    assert_eq!(Point::lincomb(&b, &Point::GENERATOR, &neg_b, &Point::GENERATOR), None);
    let minus_p = p.mul(&neg_b.mul(&b.invert().unwrap())).unwrap();
    assert_eq!(p.add(&minus_p), None);
}

#[test]
fn p256_ecdh() {
    use embassy_crypto::p256::{PublicKey, SecretKey};

    let mut rng = TestRng::new();
    let a = SecretKey::generate(&mut rng).unwrap();
    let b = SecretKey::generate(&mut rng).unwrap();
    let pa = a.public_key().unwrap();
    let pb = b.public_key().unwrap();
    let sab = a.diffie_hellman(&pb).unwrap();
    let sba = b.diffie_hellman(&pa).unwrap();
    assert_eq!(sab.as_bytes(), sba.as_bytes());

    // Cross-check against the `p256` crate.
    let a_ref = p256::SecretKey::from_slice(&a.to_bytes()).unwrap();
    let pb_ref = p256::PublicKey::from_sec1_bytes(&pb.to_sec1()).unwrap();
    let s_ref = p256::ecdh::diffie_hellman(a_ref.to_nonzero_scalar(), pb_ref.as_affine());
    assert_eq!(sab.as_bytes(), s_ref.raw_secret_bytes().as_slice());

    // Invalid peers are rejected.
    let bad = PublicKey::from_xy(pb.x(), &[1u8; 32]);
    assert_eq!(a.diffie_hellman(&bad).err(), Some(Error::InvalidKey));
    assert_eq!(SecretKey::from_bytes(&[0u8; 32]).err(), Some(Error::InvalidKey));
    assert_eq!(SecretKey::from_bytes(&[0xffu8; 32]).err(), Some(Error::InvalidKey));
    let roundtrip = SecretKey::from_bytes(&a.to_bytes()).unwrap();
    assert_eq!(roundtrip.public_key().unwrap(), pa);
    assert_eq!(PublicKey::from_sec1(&pa.to_sec1()), Ok(pa));
}

#[test]
fn p256_ecdsa() {
    use embassy_crypto::p256::{Signature, SigningKey, VerifyingKey};

    let mut rng = TestRng::new();
    let sk = SigningKey::generate(&mut rng).unwrap();
    let vk = sk.verifying_key().unwrap();
    let digest = Sha256::digest(b"hello world");
    let sig = sk.sign_prehash(&digest, &mut rng).unwrap();
    vk.verify_prehash(&digest, &sig).unwrap();

    // Interop with the `p256` crate, both directions.
    use p256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
    let vk_ref = p256::ecdsa::VerifyingKey::from_sec1_bytes(&vk.to_sec1()).unwrap();
    let sig_ref = p256::ecdsa::Signature::from_slice(&sig.to_bytes()).unwrap();
    vk_ref.verify_prehash(&digest, &sig_ref).unwrap();
    let sk_ref = p256::ecdsa::SigningKey::from_slice(&sk.to_bytes()).unwrap();
    let sig_ref: p256::ecdsa::Signature = sk_ref.sign_prehash(&digest).unwrap();
    let sig2 = Signature::from_bytes(&sig_ref.to_bytes().into()).unwrap();
    vk.verify_prehash(&digest, &sig2).unwrap();

    // Failures.
    let other = Sha256::digest(b"hello world!");
    assert_eq!(vk.verify_prehash(&other, &sig), Err(Error::InvalidSignature));
    let mut bytes = sig.to_bytes();
    bytes[10] ^= 1;
    let bad = Signature::from_bytes(&bytes).unwrap();
    assert_eq!(vk.verify_prehash(&digest, &bad), Err(Error::InvalidSignature));
    bytes[..32].fill(0);
    assert_eq!(Signature::from_bytes(&bytes).err(), Some(Error::InvalidSignature));
    let bad_vk = VerifyingKey::from_xy(vk.x(), &[1u8; 32]);
    assert_eq!(bad_vk.verify_prehash(&digest, &sig), Err(Error::InvalidKey));
    assert_eq!(VerifyingKey::from_sec1(&vk.to_sec1()), Ok(vk));
}

// ===========================================================================
// P-384
// ===========================================================================

#[test]
fn p384_arith() {
    use embassy_crypto::p384::{Point, Scalar};

    // RFC 6979 A.2.6: the public key of the private key `x`.
    let x = Scalar::from_bytes(&hex!(
        "6B9D3DAD2E1B8C1C05B19875B6659F4DE23C3B667BF297BA9AA47740787137D896D5724E4C70A825F872C9EA60D2EDF5"
    ))
    .unwrap();
    let u = Point::mul_base(&x).unwrap();
    assert_eq!(
        u.x(),
        &hex!("EC3A4E415B4E19A4568618029F427FA5DA9A8BC4AE92E02E06AAE5286B300C64DEF8F0EA9055866064A254515480BC13")
    );
    assert_eq!(
        u.y(),
        &hex!("8015D9B72D7D57244EA8EF9AC0C621896708A59367F9DFB9F54CA84B3F1C9DB1288B231C3AE0D4FE7344FD2533264720")
    );
    assert_eq!(
        Point::from_xy(Point::GENERATOR.x(), Point::GENERATOR.y()),
        Ok(Point::GENERATOR)
    );
    assert_eq!(Point::GENERATOR.mul(&x), Some(u));

    let mut rng = TestRng::new();
    let a = embassy_crypto::p384::SecretKey::generate(&mut rng).unwrap().to_scalar();
    let b = embassy_crypto::p384::SecretKey::generate(&mut rng).unwrap().to_scalar();
    assert_eq!(a.add(&b).sub(&b), a);
    let one = a.mul(&a.invert().unwrap());
    assert_eq!(Point::mul_base(&one), Some(Point::GENERATOR));
    let p = Point::mul_base(&b).unwrap();
    let expected = Point::mul_base(&a).unwrap().add(&p.mul(&b).unwrap());
    assert_eq!(Point::lincomb(&a, &Point::GENERATOR, &b, &p), expected);
}

#[test]
fn p384_ecdh_ecdsa() {
    use embassy_crypto::p384::{SecretKey, Signature, SigningKey};

    let mut rng = TestRng::new();
    let a = SecretKey::generate(&mut rng).unwrap();
    let b = SecretKey::generate(&mut rng).unwrap();
    let sab = a.diffie_hellman(&b.public_key().unwrap()).unwrap();
    let sba = b.diffie_hellman(&a.public_key().unwrap()).unwrap();
    assert_eq!(sab.as_bytes(), sba.as_bytes());

    let a_ref = p384::SecretKey::from_slice(&a.to_bytes()).unwrap();
    let pb_ref = p384::PublicKey::from_sec1_bytes(&b.public_key().unwrap().to_sec1()).unwrap();
    let s_ref = p384::ecdh::diffie_hellman(a_ref.to_nonzero_scalar(), pb_ref.as_affine());
    assert_eq!(sab.as_bytes(), s_ref.raw_secret_bytes().as_slice());

    let sk = SigningKey::generate(&mut rng).unwrap();
    let vk = sk.verifying_key().unwrap();
    let digest = Sha384::digest(b"hello world");
    let sig = sk.sign_prehash(&digest, &mut rng).unwrap();
    vk.verify_prehash(&digest, &sig).unwrap();

    use p384::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
    let vk_ref = p384::ecdsa::VerifyingKey::from_sec1_bytes(&vk.to_sec1()).unwrap();
    let sig_ref = p384::ecdsa::Signature::from_slice(&sig.to_bytes()).unwrap();
    vk_ref.verify_prehash(&digest, &sig_ref).unwrap();
    let sk_ref = p384::ecdsa::SigningKey::from_slice(&sk.to_bytes()).unwrap();
    let sig_ref: p384::ecdsa::Signature = sk_ref.sign_prehash(&digest).unwrap();
    let sig2 = Signature::from_bytes(&sig_ref.to_bytes().into()).unwrap();
    vk.verify_prehash(&digest, &sig2).unwrap();

    let other = Sha384::digest(b"hello world!");
    assert_eq!(vk.verify_prehash(&other, &sig), Err(Error::InvalidSignature));
}

// ===========================================================================
// X25519
// ===========================================================================

#[test]
fn x25519() {
    use embassy_crypto::x25519::{PublicKey, SecretKey};

    // RFC 7748 section 6.1.
    let alice = SecretKey::from_bytes(&hex!(
        "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a"
    ));
    let bob = SecretKey::from_bytes(&hex!(
        "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb"
    ));
    let alice_pk = alice.public_key().unwrap();
    let bob_pk = bob.public_key().unwrap();
    assert_eq!(
        alice_pk.to_bytes(),
        hex!("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a")
    );
    assert_eq!(
        bob_pk.to_bytes(),
        hex!("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f")
    );
    let shared = hex!("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742");
    assert_eq!(alice.diffie_hellman(&bob_pk).unwrap().as_bytes(), &shared);
    assert_eq!(bob.diffie_hellman(&alice_pk).unwrap().as_bytes(), &shared);

    // Low-order peer point: all-zero secret, rejected.
    assert_eq!(
        alice.diffie_hellman(&PublicKey::from_bytes(&[0u8; 32])).err(),
        Some(Error::InvalidKey)
    );

    let mut rng = TestRng::new();
    let k = SecretKey::generate(&mut rng).unwrap();
    assert_eq!(
        k.diffie_hellman(&bob_pk).unwrap().as_bytes(),
        bob.diffie_hellman(&k.public_key().unwrap()).unwrap().as_bytes()
    );
}
