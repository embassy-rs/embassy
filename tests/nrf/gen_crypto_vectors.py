#!/usr/bin/env python3
# Generates src/crypto_vectors.rs, the known-answer vectors for the crypto HIL tests.
# Requires the `cryptography` package.
import os

from cryptography.hazmat.primitives import cmac, hashes, hmac
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.ciphers.aead import AESCCM, AESGCM, ChaCha20Poly1305

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def pattern(n, seed):
    return bytes(((seed * 31 + i * 7 + (i >> 3)) & 0xff) for i in range(n))


def hexarr(b):
    return '[' + ', '.join('0x%02x' % x for x in b) + ']'


out = []
out.append('//! Known-answer vectors for the crypto HIL tests, generated with Python `cryptography`')
out.append('//! (see `gen_crypto_vectors.py`).')
out.append('#![allow(dead_code)]')
out.append('')
out.append('/// Deterministic test data.')
out.append('pub fn pattern(buf: &mut [u8], seed: u8) {')
out.append('    for (i, b) in buf.iter_mut().enumerate() {')
out.append('        *b = (seed as u32 * 31 + i as u32 * 7 + (i as u32 >> 3)) as u8;')
out.append('    }')
out.append('}')
out.append('')
out.append('pub struct CcmVector { pub nonce_len: usize, pub tag_len: usize, pub aad_len: usize, pub pt_len: usize, pub ct: &\'static [u8], pub tag: &\'static [u8] }')
out.append('pub struct AeadVector { pub aad_len: usize, pub pt_len: usize, pub ct: &\'static [u8], pub tag: [u8; 16] }')
out.append('')

for name, algo, dl in [('SHA1', hashes.SHA1(), 20), ('SHA224', hashes.SHA224(), 28), ('SHA256', hashes.SHA256(), 32), ('SHA384', hashes.SHA384(), 48), ('SHA512', hashes.SHA512(), 64), ('SHA512_224', hashes.SHA512_224(), 28), ('SHA512_256', hashes.SHA512_256(), 32)]:
    lens = [0, 1, 3, 55, 56, 63, 64, 65, 100, 111, 112, 119, 120, 127, 128, 129, 1000, 1024, 1500, 2049]
    out.append('/// (message length, digest) with message = pattern(len, 1).')
    out.append(f'pub const {name}: &[(usize, [u8; {dl}])] = &[')
    for n in lens:
        h = hashes.Hash(algo)
        h.update(pattern(n, 1))
        out.append(f'    ({n}, {hexarr(h.finalize())}),')
    out.append('];')
    out.append('')

for name, algo, dl in [('HMAC_SHA1', hashes.SHA1(), 20), ('HMAC_SHA224', hashes.SHA224(), 28), ('HMAC_SHA256', hashes.SHA256(), 32), ('HMAC_SHA384', hashes.SHA384(), 48), ('HMAC_SHA512', hashes.SHA512(), 64)]:
    out.append('/// (key length, message length, mac) with key = pattern(key_len, 0x55), message = pattern(len, 2).')
    out.append(f'pub const {name}: &[(usize, usize, [u8; {dl}])] = &[')
    for kl in [0, 1, 16, 32, 64, 65, 128, 129, 200]:
        for ml in [0, 1, 64, 1000]:
            m = hmac.HMAC(pattern(kl, 0x55), algo)
            m.update(pattern(ml, 2))
            out.append(f'    ({kl}, {ml}, {hexarr(m.finalize())}),')
    out.append('];')
    out.append('')

ctrs = [bytes(12) + bytes.fromhex('fffffff8'), bytes.fromhex('ff' * 16), bytes(8) + bytes.fromhex('0000000ffffffff0')]
for i, ctr in enumerate(ctrs):
    out.append(f'pub const CTR_IV{i}: [u8; 16] = {hexarr(ctr)};')
out.append(f'pub const CBC_IV: [u8; 16] = {hexarr(pattern(16, 4))};')
out.append('')
for bits, seed in [(128, 0x11), (256, 0x22)]:
    key = pattern(bits // 8, seed)
    out.append(f'pub const KEY{bits}: [u8; {bits // 8}] = {hexarr(key)};')
    pt = pattern(1024, 3)
    c = Cipher(algorithms.AES(key), modes.ECB()).encryptor()
    ct = c.update(pt) + c.finalize()
    out.append(f'/// AES-{bits}-ECB of pattern(1024, 3).')
    out.append(f'pub const AES{bits}_ECB: [u8; 1024] = {hexarr(ct)};')
    c = Cipher(algorithms.AES(key), modes.CBC(pattern(16, 4))).encryptor()
    ct = c.update(pt) + c.finalize()
    out.append(f'/// AES-{bits}-CBC of pattern(1024, 3) with CBC_IV.')
    out.append(f'pub const AES{bits}_CBC: [u8; 1024] = {hexarr(ct)};')
    for i, ctr in enumerate(ctrs):
        pt = pattern(1000, 5)
        c = Cipher(algorithms.AES(key), modes.CTR(ctr)).encryptor()
        ct = c.update(pt) + c.finalize()
        out.append(f'/// AES-{bits}-CTR of pattern(1000, 5) with CTR_IV{i}.')
        out.append(f'pub const AES{bits}_CTR{i}: [u8; 1000] = {hexarr(ct)};')
    out.append('/// (message length, tag) with message = pattern(len, 6).')
    out.append(f'pub const AES{bits}_CMAC: &[(usize, [u8; 16])] = &[')
    for n in [0, 1, 15, 16, 17, 31, 32, 33, 64, 100, 1000, 1024]:
        c = cmac.CMAC(algorithms.AES(key))
        c.update(pattern(n, 6))
        out.append(f'    ({n}, {hexarr(c.finalize())}),')
    out.append('];')
    out.append('/// CCM: nonce = pattern(nonce_len, 7), aad = pattern(aad_len, 8), plaintext = pattern(pt_len, 9).')
    out.append(f'pub const AES{bits}_CCM: &[CcmVector] = &[')
    for (nl, tl, al, pl) in [(13, 16, 0, 0), (13, 8, 8, 24), (7, 4, 0, 1), (13, 16, 300, 1000), (11, 10, 15, 16), (12, 12, 16, 17), (8, 6, 1, 33), (13, 14, 0xff00 + 5, 0)]:
        nonce, aad, pt = pattern(nl, 7), pattern(al, 8), pattern(pl, 9)
        ct = AESCCM(key, tag_length=tl).encrypt(nonce, pt, aad)
        out.append(f'    CcmVector {{ nonce_len: {nl}, tag_len: {tl}, aad_len: {al}, pt_len: {pl}, ct: &{hexarr(ct[:pl])}, tag: &{hexarr(ct[pl:])} }},')
    out.append('];')
    out.append('/// GCM: nonce = pattern(12, 7), aad = pattern(aad_len, 8), plaintext = pattern(pt_len, 9).')
    out.append(f'pub const AES{bits}_GCM: &[AeadVector] = &[')
    for (al, pl) in [(0, 0), (8, 24), (0, 1), (300, 1000), (15, 16), (16, 17), (1, 33)]:
        nonce, aad, pt = pattern(12, 7), pattern(al, 8), pattern(pl, 9)
        ct = AESGCM(key).encrypt(nonce, pt, aad)
        out.append(f'    AeadVector {{ aad_len: {al}, pt_len: {pl}, ct: &{hexarr(ct[:pl])}, tag: {hexarr(ct[pl:])} }},')
    out.append('];')
    out.append('')

key = pattern(32, 10)
nonce = pattern(12, 11)
out.append(f'pub const CHACHA_KEY: [u8; 32] = {hexarr(key)};')
out.append(f'pub const CHACHA_NONCE: [u8; 12] = {hexarr(nonce)};')
out.append('/// (initial counter, ciphertext) with plaintext = pattern(len, 12).')
out.append('pub const CHACHA20: &[(u32, &[u8])] = &[')
for counter, n in [(0, 0), (0, 1), (0, 64), (1, 65), (7, 200), (0, 1000), (0xffff_fffd, 128), (0xffff_ffff, 64)]:
    c = Cipher(algorithms.ChaCha20(key, counter.to_bytes(4, 'little') + nonce), None).encryptor()
    ct = c.update(pattern(n, 12)) + c.finalize()
    out.append(f'    ({counter:#x}, &{hexarr(ct)}),')
out.append('];')
out.append('/// ChaCha20-Poly1305: key = CHACHA_KEY, nonce = CHACHA_NONCE, aad = pattern(aad_len, 8), plaintext = pattern(pt_len, 9).')
out.append('pub const CHACHA20_POLY1305: &[AeadVector] = &[')
for (al, pl) in [(0, 0), (8, 24), (0, 1), (300, 1000), (63, 64), (64, 65), (1, 129)]:
    aad, pt = pattern(al, 8), pattern(pl, 9)
    ct = ChaCha20Poly1305(key).encrypt(nonce, pt, aad)
    out.append(f'    AeadVector {{ aad_len: {al}, pt_len: {pl}, ct: &{hexarr(ct[:pl])}, tag: {hexarr(ct[pl:])} }},')
out.append('];')
open('src/crypto_vectors.rs', 'w').write('\n'.join(out) + '\n')
