#!/usr/bin/env python3
# Generates src/pka_vectors.rs, the known-answer vectors for the PKA HIL tests.
# Requires the `cryptography` package.
import os
import random
import re

from cryptography.hazmat.primitives.asymmetric import rsa

os.chdir(os.path.dirname(os.path.abspath(__file__)))
random.seed(1234)


def be(x, n):
    return x.to_bytes(n, 'big')


def hexarr(b):
    lines = []
    for i in range(0, len(b), 12):
        lines.append('        ' + ' '.join('0x%02x,' % x for x in b[i:i + 12]))
    return '&[\n' + '\n'.join(lines) + '\n    ]'


# ---------------------------------------------------------------- elliptic curves
class Curve:
    def __init__(self, name, p, a, b, gx, gy, n, size):
        self.name, self.p, self.a, self.b, self.g, self.n = name, p, a, b, (gx, gy), n
        self.size = size

    def add(self, P, Q):
        if P is None:
            return Q
        if Q is None:
            return P
        if P[0] == Q[0] and (P[1] + Q[1]) % self.p == 0:
            return None
        if P == Q:
            l = (3 * P[0] * P[0] + self.a) * pow(2 * P[1], -1, self.p) % self.p
        else:
            l = (Q[1] - P[1]) * pow(Q[0] - P[0], -1, self.p) % self.p
        x = (l * l - P[0] - Q[0]) % self.p
        return (x, (l * (P[0] - x) - P[1]) % self.p)

    def mul(self, k, P):
        R = None
        while k:
            if k & 1:
                R = self.add(R, P)
            P = self.add(P, P)
            k >>= 1
        return R


def load_curves():
    """Reads the curve table of the driver, so the tests use the very same parameters."""
    src = open('../../embassy-nrf/src/pka/curve.rs').read()
    curves = []
    for m in re.finditer(r'pub static (\w+): Curve = Curve \{(.*?)\n\};', src, re.S):
        name, body = m.group(1), m.group(2)
        f = {}
        for fm in re.finditer(r'(\w+): &\[(.*?)\]', body, re.S):
            f[fm.group(1)] = int.from_bytes(bytes(int(x, 16) for x in re.findall(r'0x([0-9a-f]{2})', fm.group(2))), 'big')
        size = len(re.findall(r'0x[0-9a-f]{2}', re.search(r'p: &\[(.*?)\]', body, re.S).group(1)))
        curves.append(Curve(name, f['p'], f['a'], f['b'], f['gx'], f['gy'], f['n'], size))
    return curves


CURVES = load_curves()

out = []
out.append('//! Known-answer vectors for the PKA HIL tests, generated with `gen_pka_vectors.py`.')
out.append('#![allow(dead_code)]')
out.append('')
out.append('pub struct EcVector {')
out.append('    /// Private key.')
out.append('    pub d: &\'static [u8],')
out.append('    /// Public key, `d * G`.')
out.append('    pub qx: &\'static [u8],')
out.append('    pub qy: &\'static [u8],')
out.append('    /// Second key pair, for the key agreement.')
out.append('    pub d2: &\'static [u8],')
out.append('    pub q2x: &\'static [u8],')
out.append('    pub q2y: &\'static [u8],')
out.append('    /// Shared secret `d * d2 * G`.')
out.append('    pub sharedx: &\'static [u8],')
out.append('    pub sharedy: &\'static [u8],')
out.append('    /// Ephemeral key of the signature.')
out.append('    pub k: &\'static [u8],')
out.append('    /// Message hash, 32 bytes.')
out.append('    pub hash: &\'static [u8],')
out.append('    /// Signature of `hash` by `d` with `k`.')
out.append('    pub r: &\'static [u8],')
out.append('    pub s: &\'static [u8],')
out.append('}')
out.append('')

HASH = bytes((i * 37 + 11) & 0xff for i in range(32))
out.append(f'pub const HASH: [u8; 32] = {hexarr(HASH)[2:-1].join(["[", "]"])};'.replace('&[', '['))

for c in CURVES:
    d = random.randrange(1, c.n)
    d2 = random.randrange(1, c.n)
    k = random.randrange(1, c.n)
    Q = c.mul(d, c.g)
    Q2 = c.mul(d2, c.g)
    shared = c.mul(d, Q2)
    assert shared == c.mul(d2, Q)
    # ECDSA
    nbits = c.n.bit_length()
    z = int.from_bytes(HASH, 'big')
    if len(HASH) * 8 > nbits:
        z >>= len(HASH) * 8 - nbits
    R = c.mul(k, c.g)
    r = R[0] % c.n
    s = pow(k, -1, c.n) * (z + r * d) % c.n
    assert r and s
    sz = c.size
    out.append(f'/// {c.name}.')
    out.append(f'pub const {c.name}: EcVector = EcVector {{')
    for name, v in [('d', d), ('qx', Q[0]), ('qy', Q[1]), ('d2', d2), ('q2x', Q2[0]), ('q2y', Q2[1]),
                    ('sharedx', shared[0]), ('sharedy', shared[1]), ('k', k), ('r', r), ('s', s)]:
        out.append(f'    {name}: {hexarr(be(v, sz))},')
    out.append('    hash: &HASH,')
    out.append('};')
    out.append('')

# ---------------------------------------------------------------- RSA
out.append('pub struct RsaVector {')
out.append('    pub n: &\'static [u8],')
out.append('    pub e: &\'static [u8],')
out.append('    pub d: &\'static [u8],')
out.append('    pub p: &\'static [u8],')
out.append('    pub q: &\'static [u8],')
out.append('    pub dp: &\'static [u8],')
out.append('    pub dq: &\'static [u8],')
out.append('    pub qinv: &\'static [u8],')
out.append('    /// Plaintext representative, smaller than the modulus.')
out.append('    pub m: &\'static [u8],')
out.append('    /// `m ^ e mod n`.')
out.append('    pub c: &\'static [u8],')
out.append('}')
out.append('')

for bits in [1024, 2048, 3072]:
    key = rsa.generate_private_key(public_exponent=65537, key_size=bits)
    pn = key.private_numbers()
    n = pn.public_numbers.n
    e = pn.public_numbers.e
    sz = bits // 8
    m = random.randrange(1, n)
    c = pow(m, e, n)
    assert pow(c, pn.d, n) == m
    out.append(f'/// A {bits}-bit RSA key.')
    out.append(f'pub const RSA{bits}: RsaVector = RsaVector {{')
    out.append(f'    n: {hexarr(be(n, sz))},')
    out.append(f'    e: {hexarr(be(e, 4))},')
    out.append(f'    d: {hexarr(be(pn.d, sz))},')
    out.append(f'    p: {hexarr(be(pn.p, sz // 2))},')
    out.append(f'    q: {hexarr(be(pn.q, sz // 2))},')
    out.append(f'    dp: {hexarr(be(pn.dmp1, sz // 2))},')
    out.append(f'    dq: {hexarr(be(pn.dmq1, sz // 2))},')
    out.append(f'    qinv: {hexarr(be(pn.iqmp, sz // 2))},')
    out.append(f'    m: {hexarr(be(m, sz))},')
    out.append(f'    c: {hexarr(be(c, sz))},')
    out.append('};')
    out.append('')

open('src/pka_vectors.rs', 'w').write('\n'.join(out) + '\n')
print('ok')
