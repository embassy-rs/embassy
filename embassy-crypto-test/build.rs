//! Generates the vector tables in `$OUT_DIR/vectors.rs` and the byte blobs
//! they point into, one per suite, `$OUT_DIR/blob<n>.bin`.
//!
//! Two sources: a Wycheproof checkout (cloned at a pinned commit), and vectors computed here
//! with the RustCrypto crates for the algorithms Wycheproof has no file for
//! (plain digests, AES-ECB, AES-CTR, curve arithmetic, X25519 key generation,
//! Ed25519 signing).

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

/// GCM and CCM: one suite per key size.
fn aead(out: &mut Out, w: &Wycheproof, file: &str, suite: &str) {
    for bits in [128, 256] {
        let cases: Vec<String> = w
            .tests(file)
            .iter()
            .filter(|(g, _)| key_size(g) == bits)
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
        out.suite(&format!("{suite}_{bits}"), "Aead", cases);
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
                "SHA-256" => sha2::Sha256::digest(&msg).to_vec(),
                "SHA-384" => sha2::Sha384::digest(&msg).to_vec(),
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
    aead(&mut out, &w, "aes_gcm", "aes_gcm");
    aead(&mut out, &w, "aes_ccm", "aes_ccm");
    // CMAC groups with a 192-bit key are for AES-192, which does not exist
    // here; every other key size is kept so that rejection is tested.
    mac(&mut out, &w, "aes_cmac", "aes_cmac_128", |g| {
        key_size(g) != 192 && key_size(g) != 256
    });
    mac(&mut out, &w, "aes_cmac", "aes_cmac_256", |g| {
        key_size(g) != 192 && key_size(g) != 128
    });

    ec_arith(&mut out);
    dh(&mut out, &w, "ecdh_secp256r1_ecpoint", "p256_ecdh");
    dh(&mut out, &w, "ecdh_secp384r1_ecpoint", "p384_ecdh");
    ecdsa(&mut out, &w, "ecdsa_secp256r1_sha256_p1363", "p256_ecdsa");
    ecdsa(&mut out, &w, "ecdsa_secp384r1_sha384_p1363", "p384_ecdsa");
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
