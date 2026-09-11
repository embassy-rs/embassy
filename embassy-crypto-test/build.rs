//! Generates the vector tables in `$OUT_DIR/vectors.rs` and the byte blobs
//! they point into, one per suite, `$OUT_DIR/blob<n>.bin`.
//!
//! Two sources: a Wycheproof checkout (cloned at a pinned commit), and vectors computed here
//! with the RustCrypto crates for the algorithms Wycheproof has no file for:
//!
//! - plain digests
//! - AES-ECB
//! - AES-CTR
//! - ChaCha
//! - ChaCha-Poly1305
//! - CCM with a long AAD
//! - curve arithmetic
//! - known answers on more curves
//! - raw RSA
//! - X25519 key generation
//! - Ed25519 signing

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde_json::Value;

// =============================================================================
// Output
// =============================================================================

/// Every byte string of a suite lives in one blob per suite, so the linker
/// drops the vectors of the suites a binary does not call. Identical strings
/// within a suite (keys shared by a test group, ...) are stored once.
struct Out {
    out_dir: PathBuf,
    blob: Vec<u8>,
    dedup: HashMap<Vec<u8>, usize>,
    src: String,
    blobs: u32,
}

impl Out {
    fn new(out_dir: &Path) -> Self {
        Self {
            out_dir: out_dir.to_path_buf(),
            blob: Vec::new(),
            dedup: HashMap::new(),
            src: String::new(),
            blobs: 0,
        }
    }

    /// A `&'static [u8]` expression for `bytes`, into the blob of the suite being built.
    fn bytes(&mut self, bytes: &[u8]) -> String {
        if bytes.is_empty() {
            return "&[]".into();
        }
        let off = match self.dedup.get(bytes) {
            Some(&off) => off,
            None => {
                let off = self.blob.len();
                self.blob.extend_from_slice(bytes);
                self.dedup.insert(bytes.to_vec(), off);
                off
            }
        };
        format!("sub(BLOB, {off}, {})", bytes.len())
    }

    /// Write out the blob the `bytes` calls since the last flush went into,
    /// and return the name of its static.
    fn flush_blob(&mut self) -> String {
        let name = format!("blob{}.bin", self.blobs);
        self.blobs += 1;
        fs::write(self.out_dir.join(&name), &self.blob).unwrap();
        self.blob.clear();
        self.dedup.clear();
        name
    }

    /// Emit a suite from the cases formatted so far. The cases must have been
    /// formatted (their `bytes` calls made) since the previous suite.
    fn suite<T: AsRef<str>>(&mut self, name: &str, ty: &str, cases: impl IntoIterator<Item = T>) {
        let mut body = String::new();
        let mut n = 0;
        for c in cases {
            writeln!(body, "        {},", c.as_ref()).unwrap();
            n += 1;
        }
        assert!(n > 0, "empty suite {name}");
        let blob = self.flush_blob();
        writeln!(
            self.src,
            "pub static {}: Suite<{ty}> = {{
    static BLOB: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{blob}\"));
    Suite {{ name: {name:?}, cases: &[\n{body}    ] }}\n}};",
            name.to_uppercase()
        )
        .unwrap();
    }
}

fn hex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "odd hex: {s}");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn expected(result: &str) -> &'static str {
    match result {
        "valid" => "Expected::Valid",
        "invalid" => "Expected::Invalid",
        "acceptable" => "Expected::Acceptable",
        other => panic!("unknown result {other}"),
    }
}

// =============================================================================
// Wycheproof
// =============================================================================

const WYCHEPROOF_URL: &str = "https://github.com/C2SP/wycheproof";
const WYCHEPROOF_COMMIT: &str = "3fa63dd0344abb611f1fb1d77e119938603ea230";

struct Wycheproof {
    dir: PathBuf,
}

impl Wycheproof {
    /// The `testvectors_v1` directory of a checkout at [`WYCHEPROOF_COMMIT`].
    ///
    /// Cloned into `OUT_DIR` on first use. Set `WYCHEPROOF_DIR` to the root of
    /// an existing checkout to build offline.
    fn checkout(out_dir: &Path) -> Self {
        println!("cargo:rerun-if-env-changed=WYCHEPROOF_DIR");
        let root = match env::var_os("WYCHEPROOF_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => {
                let root = out_dir.join("wycheproof");
                let stamp = root.join(".embassy-commit");
                if fs::read_to_string(&stamp).ok().as_deref() != Some(WYCHEPROOF_COMMIT) {
                    let _ = fs::remove_dir_all(&root);
                    fs::create_dir_all(&root).unwrap();
                    let git = |args: &[&str]| {
                        let status = std::process::Command::new("git")
                            .current_dir(&root)
                            .args(args)
                            .status()
                            .expect("failed to run git; set WYCHEPROOF_DIR to a checkout of C2SP/wycheproof");
                        assert!(status.success(), "git {} failed", args.join(" "));
                    };
                    git(&["init", "-q"]);
                    git(&["fetch", "-q", "--depth=1", WYCHEPROOF_URL, WYCHEPROOF_COMMIT]);
                    git(&["checkout", "-q", "FETCH_HEAD"]);
                    fs::write(&stamp, WYCHEPROOF_COMMIT).unwrap();
                }
                root
            }
        };
        Self {
            dir: root.join("testvectors_v1"),
        }
    }

    fn load(&self, name: &str) -> Value {
        let path = self.dir.join(format!("{name}_test.json"));
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
    }

    /// Every (group, test) pair of the file.
    fn tests(&self, name: &str) -> Vec<(Value, Value)> {
        let file = self.load(name);
        let mut out = Vec::new();
        for group in file["testGroups"].as_array().unwrap() {
            for test in group["tests"].as_array().unwrap() {
                out.push((group.clone(), test.clone()));
            }
        }
        out
    }
}

fn field(v: &Value, name: &str) -> Vec<u8> {
    hex(v[name].as_str().unwrap_or_else(|| panic!("missing field {name}")))
}

fn tc_id(t: &Value) -> u64 {
    t["tcId"].as_u64().unwrap()
}

fn key_size(g: &Value) -> u64 {
    g["keySize"].as_u64().unwrap()
}

/// One AEAD suite from a Wycheproof file, keeping the groups `keep` selects,
/// plus the generated `extra` cases.
fn aead_suite(out: &mut Out, w: &Wycheproof, file: &str, suite: &str, keep: impl Fn(&Value) -> bool, extra: &[String]) {
    let mut cases: Vec<String> = w
        .tests(file)
        .iter()
        .filter(|(g, _)| keep(g))
        .map(|(_, t)| {
            let mut f = |n| out.bytes(&field(t, n));
            let (key, nonce, aad, msg, ct, tag) = (f("key"), f("iv"), f("aad"), f("msg"), f("ct"), f("tag"));
            format!(
                "Aead {{ tc_id: {}, key: {key}, nonce: {nonce}, aad: {aad}, msg: {msg}, ct: {ct}, tag: {tag}, result: {} }}",
                tc_id(t),
                expected(t["result"].as_str().unwrap())
            )
        })
        .collect();
    cases.extend_from_slice(extra);
    out.suite(suite, "Aead", cases);
}

/// GCM and CCM: one suite per key size.
fn aead(out: &mut Out, w: &Wycheproof, file: &str, suite: &str, extra: impl Fn(&mut Out, usize) -> Vec<String>) {
    for bits in [128, 256] {
        let extra = extra(out, bits as usize / 8);
        aead_suite(
            out,
            w,
            file,
            &format!("{suite}_{bits}"),
            |g| key_size(g) == bits,
            &extra,
        );
    }
}

/// CBC with PKCS#7 padding: one suite per key size.
fn cbc(out: &mut Out, w: &Wycheproof) {
    for bits in [128, 256] {
        let cases: Vec<String> = w
            .tests("aes_cbc_pkcs5")
            .iter()
            .filter(|(g, _)| key_size(g) == bits)
            .map(|(_, t)| {
                let mut f = |n| out.bytes(&field(t, n));
                let (key, iv, msg, ct) = (f("key"), f("iv"), f("msg"), f("ct"));
                format!(
                    "Cbc {{ tc_id: {}, key: {key}, iv: {iv}, msg: {msg}, ct: {ct}, result: {} }}",
                    tc_id(t),
                    expected(t["result"].as_str().unwrap())
                )
            })
            .collect();
        out.suite(&format!("aes_cbc_{bits}"), "Cbc", cases);
    }
}

/// HMAC and CMAC. CMAC vectors with a key of the wrong size for the suite are
/// kept: rejecting them is what is being tested.
fn mac(out: &mut Out, w: &Wycheproof, file: &str, suite: &str, keep: impl Fn(&Value) -> bool) {
    let cases: Vec<String> = w
        .tests(file)
        .iter()
        .filter(|(g, _)| keep(g))
        .map(|(_, t)| {
            let mut f = |n| out.bytes(&field(t, n));
            let (key, msg, tag) = (f("key"), f("msg"), f("tag"));
            format!(
                "Mac {{ tc_id: {}, key: {key}, msg: {msg}, tag: {tag}, result: {} }}",
                tc_id(t),
                expected(t["result"].as_str().unwrap())
            )
        })
        .collect();
    out.suite(suite, "Mac", cases);
}

/// ECDH over SEC1 points, and X25519. Both files share the private/public/shared shape.
fn dh(out: &mut Out, w: &Wycheproof, file: &str, suite: &str) {
    let cases: Vec<String> = w
        .tests(file)
        .iter()
        .map(|(_, t)| {
            let mut f = |n| out.bytes(&field(t, n));
            let (private, public, shared) = (f("private"), f("public"), f("shared"));
            format!(
                "Dh {{ tc_id: {}, private: {private}, public: {public}, shared: {shared}, result: {} }}",
                tc_id(t),
                expected(t["result"].as_str().unwrap())
            )
        })
        .collect();
    out.suite(suite, "Dh", cases);
}

/// ECDSA verification. The message is pre-hashed here so the target needs no
/// hash driver to run the suite.
fn ecdsa(out: &mut Out, w: &Wycheproof, file: &str, suite: &str) {
    use sha2::Digest;
    let cases: Vec<String> = w
        .tests(file)
        .iter()
        .map(|(g, t)| {
            let public = hex(g["publicKey"]["uncompressed"].as_str().unwrap());
            let msg = field(t, "msg");
            let digest = match g["sha"].as_str().unwrap() {
                "SHA-224" => sha2::Sha224::digest(&msg).to_vec(),
                "SHA-256" => sha2::Sha256::digest(&msg).to_vec(),
                "SHA-384" => sha2::Sha384::digest(&msg).to_vec(),
                "SHA-512" => sha2::Sha512::digest(&msg).to_vec(),
                other => panic!("unsupported hash {other}"),
            };
            let public = out.bytes(&public);
            let digest = out.bytes(&digest);
            let sig = out.bytes(&field(t, "sig"));
            format!(
                "Ecdsa {{ tc_id: {}, public: {public}, digest: {digest}, sig: {sig}, result: {} }}",
                tc_id(t),
                expected(t["result"].as_str().unwrap())
            )
        })
        .collect();
    out.suite(suite, "Ecdsa", cases);
}

/// EdDSA verification over the raw message.
fn eddsa(out: &mut Out, w: &Wycheproof, file: &str, suite: &str) {
    let cases: Vec<String> = w
        .tests(file)
        .iter()
        .map(|(g, t)| {
            let public = out.bytes(&hex(g["publicKey"]["pk"].as_str().unwrap()));
            let msg = out.bytes(&field(t, "msg"));
            let sig = out.bytes(&field(t, "sig"));
            format!(
                "Eddsa {{ tc_id: {}, public: {public}, msg: {msg}, sig: {sig}, result: {} }}",
                tc_id(t),
                expected(t["result"].as_str().unwrap())
            )
        })
        .collect();
    out.suite(suite, "Eddsa", cases);
}

// =============================================================================
// Generated vectors
// =============================================================================

/// The fixed message every generated vector is computed over (a prefix of it).
const MSG_LEN: usize = 2048;

fn message() -> Vec<u8> {
    let mut x: u32 = 0x1234_5678;
    (0..MSG_LEN)
        .map(|_| {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (x >> 16) as u8
        })
        .collect()
}

/// Deterministic bytes for keys and IVs.
fn pattern(seed: u8, len: usize) -> Vec<u8> {
    let mut x = seed;
    (0..len)
        .map(|_| {
            x = x.wrapping_mul(31).wrapping_add(7);
            x
        })
        .collect()
}

/// Message lengths around the block boundaries of every hash, plus long ones.
const DIGEST_LENS: &[usize] = &[
    0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 33, 54, 55, 56, 57, 63, 64, 65, 100, 111, 112, 113, 119, 120, 121, 127, 128,
    129, 255, 256, 257, 511, 512, 513, 1000, 1023, 1024, 1025, 2047, 2048,
];

fn digests(out: &mut Out, msg: &[u8]) {
    use sha2::Digest;
    macro_rules! suite {
        ($name:literal, $ty:ty) => {{
            let cases: Vec<String> = DIGEST_LENS
                .iter()
                .map(|&len| {
                    let d = <$ty>::digest(&msg[..len]).to_vec();
                    format!("Digest {{ len: {len}, digest: {} }}", out.bytes(&d))
                })
                .collect();
            out.suite($name, "Digest", cases);
        }};
    }
    suite!("md5", md5::Md5);
    suite!("sha1", sha1::Sha1);
    suite!("sha224", sha2::Sha224);
    suite!("sha256", sha2::Sha256);
    suite!("sha384", sha2::Sha384);
    suite!("sha512", sha2::Sha512);
    suite!("sha512_224", sha2::Sha512_224);
    suite!("sha512_256", sha2::Sha512_256);
}

fn aes_ecb_ctr(out: &mut Out, msg: &[u8]) {
    use cipher::{BlockCipherEncrypt, KeyInit, KeyIvInit, StreamCipher};

    // ECB: the first kilobyte of the message under a few keys.
    macro_rules! ecb {
        ($name:literal, $ty:ty, $key_len:literal) => {{
            let cases: Vec<String> = (0..3u8)
                .map(|i| {
                    let key = pattern(0x10 + i, $key_len);
                    let pt = &msg[..1024];
                    let mut ct = pt.to_vec();
                    let aes = <$ty>::new(key.as_slice().try_into().unwrap());
                    for block in ct.chunks_exact_mut(16) {
                        aes.encrypt_block(block.try_into().unwrap());
                    }
                    format!(
                        "Ecb {{ key: {}, pt_len: 1024, ct: {} }}",
                        out.bytes(&key),
                        out.bytes(&ct)
                    )
                })
                .collect();
            out.suite($name, "Ecb", cases);
        }};
    }
    ecb!("aes_ecb_128", aes::Aes128, 16);
    ecb!("aes_ecb_256", aes::Aes256, 32);

    // CTR: initial counters that make the counter wrap at 32, 64 and 128 bits
    // within the message, which is where implementations differ.
    let ivs: [[u8; 16]; 5] = [
        pattern(0x20, 16).try_into().unwrap(),
        [0; 16],
        {
            let mut iv = pattern(0x21, 16);
            iv[12..].copy_from_slice(&[0xff, 0xff, 0xff, 0xf0]);
            iv.try_into().unwrap()
        },
        {
            let mut iv = pattern(0x22, 16);
            iv[8..].copy_from_slice(&[0xff; 8]);
            iv[15] = 0xf0;
            iv.try_into().unwrap()
        },
        {
            let mut iv = [0xff; 16];
            iv[15] = 0xf0;
            iv
        },
    ];
    macro_rules! ctr {
        ($name:literal, $ty:ty, $key_len:literal) => {{
            let cases: Vec<String> = ivs
                .iter()
                .enumerate()
                .map(|(i, iv)| {
                    let key = pattern(0x30 + i as u8, $key_len);
                    let pt_len = 1000;
                    let mut ct = msg[..pt_len].to_vec();
                    let mut c = <ctr::Ctr128BE<$ty>>::new(key.as_slice().try_into().unwrap(), iv.into());
                    c.apply_keystream(&mut ct);
                    format!(
                        "Ctr {{ key: {}, iv: {}, pt_len: {pt_len}, ct: {} }}",
                        out.bytes(&key),
                        out.bytes(iv),
                        out.bytes(&ct)
                    )
                })
                .collect();
            out.suite($name, "Ctr", cases);
        }};
    }
    ctr!("aes_ctr_128", aes::Aes128, 16);
    ctr!("aes_ctr_256", aes::Aes256, 32);
}

/// The ChaCha block function (RFC 8439 section 2.3) with `rounds` rounds: the
/// `chacha20` crate refuses to step its counter past the 32-bit wrap, which is
/// exactly the case worth testing, so the keystream blocks are computed here
/// and only checked against the crate where it can.
fn chacha_block(rounds: usize, key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> [u8; 64] {
    let mut state = [0u32; 16];
    state[..4].copy_from_slice(&[0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    for (i, w) in key.chunks_exact(4).enumerate() {
        state[4 + i] = u32::from_le_bytes(w.try_into().unwrap());
    }
    state[12] = counter;
    for (i, w) in nonce.chunks_exact(4).enumerate() {
        state[13 + i] = u32::from_le_bytes(w.try_into().unwrap());
    }
    let mut x = state;
    let qr = |x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize| {
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(16);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(12);
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(8);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(7);
    };
    for _ in 0..rounds / 2 {
        qr(&mut x, 0, 4, 8, 12);
        qr(&mut x, 1, 5, 9, 13);
        qr(&mut x, 2, 6, 10, 14);
        qr(&mut x, 3, 7, 11, 15);
        qr(&mut x, 0, 5, 10, 15);
        qr(&mut x, 1, 6, 11, 12);
        qr(&mut x, 2, 7, 8, 13);
        qr(&mut x, 3, 4, 9, 14);
    }
    let mut out = [0u8; 64];
    for i in 0..16 {
        out[4 * i..4 * i + 4].copy_from_slice(&x[i].wrapping_add(state[i]).to_le_bytes());
    }
    out
}

/// ChaCha keystreams with `rounds` rounds, starting at block counters that
/// make the 32-bit counter wrap within the message.
fn chacha(out: &mut Out, msg: &[u8], rounds: usize) {
    use cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
    let cases: Vec<String> = [
        (0u32, 1usize),
        (0, 64),
        (1, 65),
        (7, 200),
        (0, 1000),
        (0, MSG_LEN),
        (0xffff_fffd, 128),
        (0xffff_fffe, 200),
        (0xffff_ffff, 64),
    ]
    .iter()
    .enumerate()
    .map(|(i, &(counter, pt_len))| {
        let key: [u8; 32] = pattern(0x90 + i as u8, 32).try_into().unwrap();
        let nonce: [u8; 12] = pattern(0xa0 + i as u8, 12).try_into().unwrap();
        let mut ct = msg[..pt_len].to_vec();
        for (b, block) in ct.chunks_mut(64).enumerate() {
            let ks = chacha_block(rounds, &key, &nonce, counter.wrapping_add(b as u32));
            for (c, k) in block.iter_mut().zip(ks) {
                *c ^= k;
            }
        }
        if u64::from(counter) * 64 + pt_len as u64 <= u64::from(u32::MAX) * 64 {
            let mut reference = msg[..pt_len].to_vec();
            let mut c: Box<dyn StreamCipher> = match rounds {
                8 => {
                    let mut c = chacha20::ChaCha8::new((&key).into(), (&nonce).into());
                    c.seek(u64::from(counter) * 64);
                    Box::new(c)
                }
                12 => {
                    let mut c = chacha20::ChaCha12::new((&key).into(), (&nonce).into());
                    c.seek(u64::from(counter) * 64);
                    Box::new(c)
                }
                20 => {
                    let mut c = chacha20::ChaCha20::new((&key).into(), (&nonce).into());
                    c.seek(u64::from(counter) * 64);
                    Box::new(c)
                }
                _ => unreachable!(),
            };
            c.apply_keystream(&mut reference);
            assert_eq!(
                ct, reference,
                "chacha{rounds} block function disagrees with the chacha20 crate"
            );
        }
        format!(
            "ChaCha {{ key: {}, nonce: {}, counter: {counter:#x}, pt_len: {pt_len}, ct: {} }}",
            out.bytes(&key),
            out.bytes(&nonce),
            out.bytes(&ct)
        )
    })
    .collect();
    out.suite(&format!("chacha{rounds}"), "ChaCha", cases);
}

/// ChaCha-Poly1305 with a reduced-round ChaCha, with the `chacha20poly1305`
/// crate: Wycheproof only covers the 20-round version. AAD and message lengths
/// around the 16-byte Poly1305 block and the 64-byte ChaCha block, one case
/// with a tampered tag, one with tampered ciphertext.
fn chacha_poly1305(out: &mut Out, msg: &[u8], rounds: usize) {
    use aead::{Aead, KeyInit, Payload};
    let cases: Vec<String> = [
        (0usize, 0usize),
        (0, 1),
        (1, 0),
        (16, 16),
        (15, 17),
        (17, 63),
        (32, 64),
        (100, 65),
        (7, 200),
        (0, 1000),
        (300, 1024),
    ]
    .iter()
    .enumerate()
    .flat_map(|(i, &(aad_len, msg_len))| {
        let key: [u8; 32] = pattern(0xc0 + i as u8, 32).try_into().unwrap();
        let nonce: [u8; 12] = pattern(0xd0 + i as u8, 12).try_into().unwrap();
        let aad = pattern(0xe0 + i as u8, aad_len);
        let pt = &msg[..msg_len];
        let payload = Payload { msg: pt, aad: &aad };
        let ct_tag = match rounds {
            8 => chacha20poly1305::ChaCha8Poly1305::new((&key).into()).encrypt((&nonce).into(), payload),
            12 => chacha20poly1305::ChaCha12Poly1305::new((&key).into()).encrypt((&nonce).into(), payload),
            _ => unreachable!(),
        }
        .unwrap();
        let (ct, tag) = ct_tag.split_at(msg_len);
        let mut bad_tag = tag.to_vec();
        bad_tag[0] ^= 1;
        let mut bad_ct = ct.to_vec();
        if let Some(b) = bad_ct.first_mut() {
            *b ^= 1;
        }
        let mut case = |tc_id: usize, ct: &[u8], tag: &[u8], result: &str| {
            format!(
                "Aead {{ tc_id: {tc_id}, key: {}, nonce: {}, aad: {}, msg: {}, ct: {}, tag: {}, result: {result} }}",
                out.bytes(&key),
                out.bytes(&nonce),
                out.bytes(&aad),
                out.bytes(pt),
                out.bytes(ct),
                out.bytes(tag),
            )
        };
        let mut v = vec![
            case(3 * i + 1, ct, tag, "Expected::Valid"),
            case(3 * i + 2, ct, &bad_tag, "Expected::Invalid"),
        ];
        if msg_len > 0 {
            v.push(case(3 * i + 3, &bad_ct, tag, "Expected::Invalid"));
        }
        v
    })
    .collect();
    out.suite(&format!("chacha{rounds}_poly1305"), "Aead", cases);
}

/// Length of [`long_aad`]: past `0xFEFF`, where CCM switches to a 6-byte
/// encoding of the AAD length.
const LONG_AAD_LEN: usize = 0xff00 + 5;

fn long_aad() -> Vec<u8> {
    pattern(0xb0, LONG_AAD_LEN)
}

/// CCM (NIST SP 800-38C) with the `aes` crate, for the cases Wycheproof lacks.
fn ccm(key: &[u8], nonce: &[u8], aad: &[u8], msg: &[u8], tag_len: usize) -> (Vec<u8>, Vec<u8>) {
    use cipher::{BlockCipherEncrypt, KeyInit};
    let enc: Box<dyn Fn(&mut [u8; 16])> = match key.len() {
        16 => {
            let aes = aes::Aes128::new(<&[u8; 16]>::try_from(key).unwrap().into());
            Box::new(move |b: &mut [u8; 16]| aes.encrypt_block(b.into()))
        }
        32 => {
            let aes = aes::Aes256::new(<&[u8; 32]>::try_from(key).unwrap().into());
            Box::new(move |b: &mut [u8; 16]| aes.encrypt_block(b.into()))
        }
        _ => unreachable!(),
    };
    let q = 15 - nonce.len();
    assert!(
        aad.len() < 0xff00 + 0x1_0000,
        "the long AAD encoding is not implemented"
    );

    // B0 and the AAD length header, then the CBC-MAC.
    let mut b0 = [0u8; 16];
    b0[0] = (if aad.is_empty() { 0 } else { 0x40 }) | (((tag_len - 2) / 2) as u8) << 3 | (q - 1) as u8;
    b0[1..1 + nonce.len()].copy_from_slice(nonce);
    b0[16 - q..].copy_from_slice(&(msg.len() as u64).to_be_bytes()[8 - q..]);
    let mut auth = b0.to_vec();
    if !aad.is_empty() {
        if aad.len() < 0xff00 {
            auth.extend_from_slice(&(aad.len() as u16).to_be_bytes());
        } else {
            auth.extend_from_slice(&[0xff, 0xfe]);
            auth.extend_from_slice(&(aad.len() as u32).to_be_bytes());
        }
        auth.extend_from_slice(aad);
        auth.resize(auth.len().div_ceil(16) * 16, 0);
    }
    auth.extend_from_slice(msg);
    auth.resize(auth.len().div_ceil(16) * 16, 0);
    let mut mac = [0u8; 16];
    for block in auth.chunks_exact(16) {
        for (m, b) in mac.iter_mut().zip(block) {
            *m ^= b;
        }
        enc(&mut mac);
    }

    // CTR: block 0 encrypts the tag, blocks 1.. the message.
    let mut ctr = [0u8; 16];
    ctr[0] = (q - 1) as u8;
    ctr[1..1 + nonce.len()].copy_from_slice(nonce);
    let keystream = |i: u64| {
        let mut b = ctr;
        let n = i.to_be_bytes();
        b[16 - q..].copy_from_slice(&n[8 - q..]);
        enc(&mut b);
        b
    };
    let s0 = keystream(0);
    let tag: Vec<u8> = mac.iter().zip(s0).map(|(m, s)| m ^ s).take(tag_len).collect();
    let mut ct = msg.to_vec();
    for (i, block) in ct.chunks_mut(16).enumerate() {
        let s = keystream(i as u64 + 1);
        for (c, s) in block.iter_mut().zip(s) {
            *c ^= s;
        }
    }
    (ct, tag)
}

/// Extra CCM cases: an AAD longer than `0xFEFF` bytes, which changes the
/// encoding of its length. The AAD is the shared `LONG_AAD` static. Numbered
/// from 100000 so they cannot collide with Wycheproof.
fn ccm_long_aad(out: &mut Out, key_len: usize, msg: &[u8]) -> Vec<String> {
    let aad = long_aad();
    [(13usize, 16usize, 100usize), (7, 8, 33)]
        .iter()
        .enumerate()
        .map(|(i, &(nonce_len, tag_len, msg_len))| {
            let key = pattern(0xc0 + i as u8, key_len);
            let nonce = pattern(0xd0 + i as u8, nonce_len);
            let msg = &msg[..msg_len];
            let (ct, tag) = ccm(&key, &nonce, &aad, msg, tag_len);
            format!(
                "Aead {{ tc_id: {}, key: {}, nonce: {}, aad: LONG_AAD, msg: {}, ct: {}, tag: {}, result: Expected::Valid }}",
                100_000 + i,
                out.bytes(&key),
                out.bytes(&nonce),
                out.bytes(msg),
                out.bytes(&ct),
                out.bytes(&tag)
            )
        })
        .collect()
}

/// Raw RSA under the private keys of the Wycheproof PKCS#1 files, one suite per
/// key size.
fn rsa(out: &mut Out, w: &Wycheproof) {
    use num_bigint_dig::BigUint;
    for bits in [2048usize, 3072, 4096] {
        let file = w.load(&format!("rsa_pkcs1_{bits}"));
        let key = &file["testGroups"][0]["privateKey"];
        let int = |name: &str| BigUint::from_bytes_be(&hex(key[name].as_str().unwrap()));
        let n = int("modulus");
        let e = int("publicExponent");
        let d = int("privateExponent");
        let (p, q) = (int("prime1"), int("prime2"));
        let (dp, dq, qinv) = (int("exponent1"), int("exponent2"), int("coefficient"));
        let size = bits / 8;
        assert_eq!(n.bits(), bits);
        assert_eq!(&p * &q, n);
        let fixed = |x: &BigUint, len: usize| -> Vec<u8> {
            let b = x.to_bytes_be();
            assert!(b.len() <= len);
            let mut v = vec![0u8; len - b.len()];
            v.extend_from_slice(&b);
            v
        };
        let cases: Vec<String> = (0..2u8)
            .map(|i| {
                let m = BigUint::from_bytes_be(&pattern(0xe0 + i, size)) % &n;
                let c = m.modpow(&e, &n);
                assert_eq!(c.modpow(&d, &n), m);
                format!(
                    "Rsa {{ n: {}, e: {}, d: {}, p: {}, q: {}, dp: {}, dq: {}, qinv: {}, m: {}, c: {} }}",
                    out.bytes(&fixed(&n, size)),
                    out.bytes(&e.to_bytes_be()),
                    out.bytes(&fixed(&d, size)),
                    out.bytes(&fixed(&p, size / 2)),
                    out.bytes(&fixed(&q, size / 2)),
                    out.bytes(&fixed(&dp, size / 2)),
                    out.bytes(&fixed(&dq, size / 2)),
                    out.bytes(&fixed(&qinv, size / 2)),
                    out.bytes(&fixed(&m, size)),
                    out.bytes(&fixed(&c, size)),
                )
            })
            .collect();
        out.suite(&format!("rsa_{bits}"), "Rsa", cases);
    }
}

/// A short Weierstrass curve `y² = x³ + a·x + b` over `GF(p)`, with generic
/// (slow, non-constant-time) arithmetic: it computes reference values only.
struct Weierstrass {
    name: &'static str,
    p: num_bigint_dig::BigUint,
    a: num_bigint_dig::BigUint,
    b: num_bigint_dig::BigUint,
    g: (num_bigint_dig::BigUint, num_bigint_dig::BigUint),
    n: num_bigint_dig::BigUint,
    /// Size of a coordinate or scalar, in bytes.
    size: usize,
}

type Affine = Option<(num_bigint_dig::BigUint, num_bigint_dig::BigUint)>;

impl Weierstrass {
    fn new(name: &'static str, p: &str, a: &str, b: &str, gx: &str, gy: &str, n: &str) -> Self {
        use num_bigint_dig::BigUint;
        let int = |s: &str| BigUint::from_bytes_be(&hex(s));
        let c = Self {
            name,
            p: int(p),
            a: int(a),
            b: int(b),
            g: (int(gx), int(gy)),
            n: int(n),
            size: hex(p).len(),
        };
        // The parameters are typed in by hand: check that they agree with each other.
        let (gx, gy) = &c.g;
        let lhs = (gy * gy) % &c.p;
        let rhs = (gx * gx * gx + &c.a * gx + &c.b) % &c.p;
        assert_eq!(lhs, rhs, "{name}: generator not on curve");
        assert!(c.mul(&c.n, &Some(c.g.clone())).is_none(), "{name}: order wrong");
        c
    }

    fn inv(x: &num_bigint_dig::BigUint, m: &num_bigint_dig::BigUint) -> num_bigint_dig::BigUint {
        use num_bigint_dig::BigUint;
        // `m` is prime.
        x.modpow(&(m - BigUint::from(2u8)), m)
    }

    fn add(&self, p: &Affine, q: &Affine) -> Affine {
        let (Some((x1, y1)), Some((x2, y2))) = (p, q) else {
            return p.clone().or_else(|| q.clone());
        };
        let m = &self.p;
        let lambda = if x1 == x2 {
            if (y1 + y2) % m == num_bigint_dig::BigUint::default() {
                return None;
            }
            let num = (num_bigint_dig::BigUint::from(3u8) * x1 * x1 + &self.a) % m;
            let den = Self::inv(&((num_bigint_dig::BigUint::from(2u8) * y1) % m), m);
            (num * den) % m
        } else {
            let num = (y2 + m - y1) % m;
            let den = Self::inv(&((x2 + m - x1) % m), m);
            (num * den) % m
        };
        let x3 = (&lambda * &lambda + m + m - x1 - x2) % m;
        let y3 = (lambda * ((x1 + m - &x3) % m) + m - y1) % m;
        Some((x3, y3))
    }

    fn mul(&self, k: &num_bigint_dig::BigUint, p: &Affine) -> Affine {
        let mut r: Affine = None;
        let mut q = p.clone();
        let bits = k.to_radix_le(2);
        for &bit in &bits {
            if bit == 1 {
                r = self.add(&r, &q);
            }
            q = self.add(&q, &q);
        }
        r
    }

    fn scalar_bytes(&self, s: &num_bigint_dig::BigUint) -> Vec<u8> {
        let b = s.to_bytes_be();
        let mut v = vec![0u8; self.size - b.len()];
        v.extend_from_slice(&b);
        v
    }

    /// Uncompressed SEC1.
    fn point_bytes(&self, p: &Affine) -> Vec<u8> {
        let (x, y) = p.as_ref().expect("point at infinity");
        let mut v = vec![4u8];
        v.extend_from_slice(&self.scalar_bytes(x));
        v.extend_from_slice(&self.scalar_bytes(y));
        v
    }

    /// A scalar in `1..n` from `seed`.
    fn scalar(&self, seed: u8) -> num_bigint_dig::BigUint {
        use num_bigint_dig::BigUint;
        BigUint::from_bytes_be(&pattern(seed, self.size)) % (&self.n - BigUint::from(1u8)) + BigUint::from(1u8)
    }

    /// The leftmost `bits(n)` bits of `digest`, as an integer (FIPS 186-4).
    fn truncate(&self, digest: &[u8]) -> num_bigint_dig::BigUint {
        let z = num_bigint_dig::BigUint::from_bytes_be(digest);
        let extra = (digest.len() * 8).saturating_sub(self.n.bits());
        z >> extra
    }
}

/// Known answers on every curve the hardware drivers support: key pairs, a
/// shared point, and a signature with a fixed nonce over digests of
/// various lengths.
fn ec_kat(out: &mut Out, msg: &[u8]) {
    use num_bigint_dig::BigUint;
    use sha2::Digest;

    let curves = [
        Weierstrass::new(
            "p192",
            "fffffffffffffffffffffffffffffffeffffffffffffffff",
            "fffffffffffffffffffffffffffffffefffffffffffffffc",
            "64210519e59c80e70fa7e9ab72243049feb8deecc146b9b1",
            "188da80eb03090f67cbf20eb43a18800f4ff0afd82ff1012",
            "07192b95ffc8da78631011ed6b24cdd573f977a11e794811",
            "ffffffffffffffffffffffff99def836146bc9b1b4d22831",
        ),
        Weierstrass::new(
            "p224",
            "ffffffffffffffffffffffffffffffff000000000000000000000001",
            "fffffffffffffffffffffffffffffffefffffffffffffffffffffffe",
            "b4050a850c04b3abf54132565044b0b7d7bfd8ba270b39432355ffb4",
            "b70e0cbd6bb4bf7f321390b94a03c1d356c21122343280d6115c1d21",
            "bd376388b5f723fb4c22dfe6cd4375a05a07476444d5819985007e34",
            "ffffffffffffffffffffffffffff16a2e0b8f03e13dd29455c5c2a3d",
        ),
        Weierstrass::new(
            "p256",
            "ffffffff00000001000000000000000000000000ffffffffffffffffffffffff",
            "ffffffff00000001000000000000000000000000fffffffffffffffffffffffc",
            "5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b",
            "6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296",
            "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5",
            "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551",
        ),
        Weierstrass::new(
            "p384",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000fffffffc",
            "b3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef",
            "aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7",
            "3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f",
            "ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973",
        ),
        Weierstrass::new(
            "p521",
            "01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "01fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc",
            "0051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00",
            "00c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66",
            "011839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650",
            "01fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa51868783bf2f966b7fcc0148f709a5d03bb5c9b8899c47aebb6fb71e91386409",
        ),
        Weierstrass::new(
            "secp256k1",
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000007",
            "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            "483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
            "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
        ),
    ];

    for (ci, c) in curves.iter().enumerate() {
        let one = BigUint::from(1u8);
        let g: Affine = Some(c.g.clone());
        // Digests: a SHA-256 (32 bytes), one the size of the curve, a SHA-512
        // (64 bytes, longer than every order) and a SHA-1 (20 bytes, shorter).
        let digests: [Vec<u8>; 4] = [
            sha2::Sha256::digest(&msg[..100]).to_vec(),
            pattern(0xf0 + ci as u8, c.size),
            sha2::Sha512::digest(&msg[..200]).to_vec(),
            sha1::Sha1::digest(&msg[..300]).to_vec(),
        ];
        let cases: Vec<String> = (0..4usize)
            .map(|i| {
                let seed = |k: u8| 0x10 * ci as u8 + 4 * i as u8 + k;
                let d = match i {
                    0 => one.clone(),
                    3 => &c.n - &one,
                    _ => c.scalar(seed(0)),
                };
                let d2 = c.scalar(seed(1));
                let k = c.scalar(seed(2));
                let q = c.mul(&d, &g);
                let q2 = c.mul(&d2, &g);
                let shared = c.mul(&d, &q2);
                assert_eq!(shared, c.mul(&d2, &q), "{}: key agreement", c.name);

                // ECDSA (FIPS 186-4 section 6.4.1).
                let digest = &digests[i];
                let z = c.truncate(digest) % &c.n;
                let (rx, _) = c.mul(&k, &g).unwrap();
                let r = rx % &c.n;
                let s = (Weierstrass::inv(&k, &c.n) * ((&z + &r * &d) % &c.n)) % &c.n;
                assert!(r != BigUint::default() && s != BigUint::default());
                // Check it verifies: (z s⁻¹) G + (r s⁻¹) Q has x = r.
                let s_inv = Weierstrass::inv(&s, &c.n);
                let u1 = (&z * &s_inv) % &c.n;
                let u2 = (&r * &s_inv) % &c.n;
                let (vx, _) = c.add(&c.mul(&u1, &g), &c.mul(&u2, &q)).unwrap();
                assert_eq!(vx % &c.n, r, "{}: signature does not verify", c.name);

                let mut sig = c.scalar_bytes(&r);
                sig.extend_from_slice(&c.scalar_bytes(&s));
                format!(
                    "EcKat {{ private: {}, public: {}, private2: {}, public2: {}, shared: {}, k: {}, digest: {}, sig: {} }}",
                    out.bytes(&c.scalar_bytes(&d)),
                    out.bytes(&c.point_bytes(&q)),
                    out.bytes(&c.scalar_bytes(&d2)),
                    out.bytes(&c.point_bytes(&q2)),
                    out.bytes(&c.point_bytes(&shared)),
                    out.bytes(&c.scalar_bytes(&k)),
                    out.bytes(digest),
                    out.bytes(&sig),
                )
            })
            .collect();
        out.suite(&format!("{}_kat", c.name), "EcKat", cases);
    }

    // The generic arithmetic against an independent implementation.
    {
        use elliptic_curve::sec1::ToSec1Point;
        let c = &curves[2];
        let d = c.scalar(0x77);
        let reference = p256::ProjectivePoint::GENERATOR
            * Option::<p256::Scalar>::from(<p256::Scalar as elliptic_curve::ff::PrimeField>::from_repr(
                c.scalar_bytes(&d).as_slice().try_into().unwrap(),
            ))
            .unwrap();
        let reference = p256::AffinePoint::from(reference).to_sec1_point(false);
        assert_eq!(c.point_bytes(&c.mul(&d, &Some(c.g.clone()))), reference.as_bytes());
    }
}

/// Curve arithmetic: scalar field operations and point operations on random
/// inputs, plus the edge cases around the group order.
fn ec_arith(out: &mut Out) {
    macro_rules! curve {
        ($name:literal, $curve:ident, $n:literal) => {{
            use elliptic_curve::ff::PrimeField;
            use elliptic_curve::group::Group;
            use elliptic_curve::sec1::ToSec1Point;
            use $curve::{AffinePoint, ProjectivePoint, Scalar};

            let scalar_bytes = |s: &Scalar| -> Vec<u8> { s.to_repr().to_vec() };
            let point_bytes = |p: &ProjectivePoint| -> Vec<u8> {
                let a = AffinePoint::from(*p);
                let e = a.to_sec1_point(false);
                e.as_bytes().to_vec()
            };
            let scalar_from = |bytes: &[u8]| -> Scalar {
                let repr: <Scalar as PrimeField>::Repr = bytes.try_into().unwrap();
                Option::from(Scalar::from_repr(repr)).unwrap()
            };

            // (a, b) pairs: random ones, then edge cases.
            let n_minus_1 = -Scalar::ONE;
            let two = Scalar::ONE + Scalar::ONE;
            let pairs: Vec<(Scalar, Scalar)> = {
                let mut v = Vec::new();
                for i in 0..4u8 {
                    let mut a = pattern(0x40 + i, $n);
                    let mut b = pattern(0x50 + i, $n);
                    // Keep them below the order.
                    a[0] &= 0x3f;
                    b[0] &= 0x3f;
                    v.push((scalar_from(&a), scalar_from(&b)));
                }
                v.push((Scalar::ONE, Scalar::ONE));
                v.push((Scalar::ONE, n_minus_1));
                v.push((n_minus_1, n_minus_1));
                v.push((two, n_minus_1));
                v
            };
            let cases: Vec<String> = pairs
                .iter()
                .map(|(a, b)| {
                    let g = ProjectivePoint::GENERATOR;
                    let p = g * a;
                    let q = g * b;
                    let sum = p + q;
                    let a_q = q * a;
                    let lincomb = g * a + q * b;
                    let point_or_infinity = |p: &ProjectivePoint| -> Vec<u8> {
                        if bool::from(p.is_identity()) {
                            Vec::new()
                        } else {
                            point_bytes(p)
                        }
                    };
                    let (a_b, b_b) = (out.bytes(&scalar_bytes(a)), out.bytes(&scalar_bytes(b)));
                    let (p_b, q_b) = (out.bytes(&point_or_infinity(&p)), out.bytes(&point_or_infinity(&q)));
                    let sum_b = out.bytes(&point_or_infinity(&sum));
                    let aq_b = out.bytes(&point_or_infinity(&a_q));
                    let lin_b = out.bytes(&point_or_infinity(&lincomb));
                    let add = out.bytes(&scalar_bytes(&(a + b)));
                    let sub = out.bytes(&scalar_bytes(&(a - b)));
                    let mul = out.bytes(&scalar_bytes(&(a * b)));
                    let inv = out.bytes(&scalar_bytes(&Option::from(elliptic_curve::Field::invert(a)).unwrap()));
                    format!(
                        "EcArith {{ a: {a_b}, b: {b_b}, a_g: {p_b}, b_g: {q_b}, sum: {sum_b}, a_times_b_g: {aq_b}, lincomb: {lin_b}, \
                         a_plus_b: {add}, a_minus_b: {sub}, a_times_b: {mul}, a_inv: {inv} }}"
                    )
                })
                .collect();
            out.suite($name, "EcArith", cases);
        }};
    }
    curve!("p256_arith", p256, 32);
    curve!("p384_arith", p384, 48);
}

fn x25519_keygen(out: &mut Out) {
    let cases: Vec<String> = (0..6u8)
        .map(|i| {
            let private: [u8; 32] = pattern(0x60 + i, 32).try_into().unwrap();
            let public = x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(private));
            format!(
                "KeyPair {{ private: {}, public: {} }}",
                out.bytes(&private),
                out.bytes(public.as_bytes())
            )
        })
        .collect();
    out.suite("x25519_keygen", "KeyPair", cases);
}

/// Ed25519 key derivation and deterministic signing over prefixes of the message.
fn ed25519_sign(out: &mut Out, msg: &[u8]) {
    use ed25519_dalek::Signer;
    let cases: Vec<String> = [0usize, 1, 3, 32, 63, 64, 65, 127, 128, 255, 1000, MSG_LEN]
        .iter()
        .enumerate()
        .map(|(i, &len)| {
            let private: [u8; 32] = pattern(0x80 + i as u8, 32).try_into().unwrap();
            let sk = ed25519_dalek::SigningKey::from_bytes(&private);
            let public = sk.verifying_key().to_bytes();
            let sig = sk.sign(&msg[..len]).to_bytes();
            format!(
                "Sign {{ private: {}, public: {}, msg_len: {len}, sig: {} }}",
                out.bytes(&private),
                out.bytes(&public),
                out.bytes(&sig)
            )
        })
        .collect();
    out.suite("ed25519_sign", "Sign", cases);
}

// =============================================================================

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let w = Wycheproof::checkout(&out_dir);
    let mut out = Out::new(&out_dir);

    let msg = message();
    let msg_expr = out.bytes(&msg);
    let blob = out.flush_blob();
    writeln!(out.src, "/// The message the generated vectors are computed over.").unwrap();
    writeln!(
        out.src,
        "pub static MESSAGE: &[u8] = {{\n    static BLOB: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{blob}\"));\n    {msg_expr}\n}};"
    )
    .unwrap();

    let aad_expr = out.bytes(&long_aad());
    let blob = out.flush_blob();
    writeln!(
        out.src,
        "/// The additional data of the generated CCM cases with a long AAD."
    )
    .unwrap();
    writeln!(
        out.src,
        "pub static LONG_AAD: &[u8] = {{\n    static BLOB: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{blob}\"));\n    {aad_expr}\n}};"
    )
    .unwrap();

    digests(&mut out, &msg);
    for (file, suite) in [
        ("hmac_sha1", "hmac_sha1"),
        ("hmac_sha224", "hmac_sha224"),
        ("hmac_sha256", "hmac_sha256"),
        ("hmac_sha384", "hmac_sha384"),
        ("hmac_sha512", "hmac_sha512"),
        ("hmac_sha512_224", "hmac_sha512_224"),
        ("hmac_sha512_256", "hmac_sha512_256"),
    ] {
        mac(&mut out, &w, file, suite, |_| true);
    }

    aes_ecb_ctr(&mut out, &msg);
    cbc(&mut out, &w);
    aead(&mut out, &w, "aes_gcm", "aes_gcm", |_, _| Vec::new());
    // The AAD length encoding does not depend on the key size, and the AAD is
    // large: the 128-bit suite alone carries it.
    aead(&mut out, &w, "aes_ccm", "aes_ccm", |out, key_len| match key_len {
        16 => ccm_long_aad(out, key_len, &msg),
        _ => Vec::new(),
    });
    // CMAC groups with a 192-bit key are for AES-192, which does not exist
    // here; every other key size is kept so that rejection is tested.
    mac(&mut out, &w, "aes_cmac", "aes_cmac_128", |g| {
        key_size(g) != 192 && key_size(g) != 256
    });
    mac(&mut out, &w, "aes_cmac", "aes_cmac_256", |g| {
        key_size(g) != 192 && key_size(g) != 128
    });

    for rounds in [8, 12, 20] {
        chacha(&mut out, &msg, rounds);
    }
    for rounds in [8, 12] {
        chacha_poly1305(&mut out, &msg, rounds);
    }
    aead_suite(&mut out, &w, "chacha20_poly1305", "chacha20_poly1305", |_| true, &[]);

    rsa(&mut out, &w);

    ec_arith(&mut out);
    ec_kat(&mut out, &msg);
    dh(&mut out, &w, "ecdh_secp224r1_ecpoint", "p224_ecdh");
    dh(&mut out, &w, "ecdh_secp256r1_ecpoint", "p256_ecdh");
    dh(&mut out, &w, "ecdh_secp384r1_ecpoint", "p384_ecdh");
    dh(&mut out, &w, "ecdh_secp521r1_ecpoint", "p521_ecdh");
    ecdsa(&mut out, &w, "ecdsa_secp192r1_sha256_p1363", "p192_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp224r1_sha256_p1363", "p224_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp256r1_sha256_p1363", "p256_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp384r1_sha384_p1363", "p384_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp521r1_sha512_p1363", "p521_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp256k1_sha256_p1363", "secp256k1_ecdsa");
    dh(&mut out, &w, "x25519", "x25519");
    x25519_keygen(&mut out);
    eddsa(&mut out, &w, "ed25519", "ed25519");
    ed25519_sign(&mut out, &msg);

    let mut src = String::new();
    writeln!(
        src,
        "const fn sub(blob: &'static [u8], off: usize, len: usize) -> &'static [u8] {{ blob.split_at(off).1.split_at(len).0 }}"
    )
    .unwrap();
    src.push_str(&out.src);
    fs::write(out_dir.join("vectors.rs"), src).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
