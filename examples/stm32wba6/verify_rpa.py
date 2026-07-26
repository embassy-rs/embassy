#!/usr/bin/env python3
"""Verify whether an RPA was generated from a given IRK.

Usage:
    python3 verify_rpa.py <IRK_hex_32_chars> <RPA_hex_12_chars>

Example:
    python3 verify_rpa.py 0123456789abcdef0123456789abcdef 4011223344aabbcc

Both arguments accept either bare hex or colon-separated (AA:BB:...).

Per BT Core Spec Vol 6 Part B 1.3.2.2:
    RPA layout (6 bytes, big-endian on wire): [hash(3) | prand(3)]
    Top 2 bits of prand[2] (MSB byte) are 0b01 for RPA.
    hash = ah(IRK, prand) = AES-128(IRK, 0..0 || prand)[13..16] (lower 3 bytes)

Requires: pip install pycryptodome
"""
import sys
from Crypto.Cipher import AES


def normalize_hex(s: str) -> bytes:
    return bytes.fromhex(s.replace(":", "").replace(" ", ""))


def ah(irk: bytes, prand: bytes) -> bytes:
    """BT Core Vol 3 Part H 2.2.2: random address hash function ah.
    irk: 16 bytes, big-endian (as printed in spec / Wireshark IDInformation).
    prand: 3 bytes, big-endian (most significant first as printed).
    Returns hash: 3 bytes, big-endian."""
    # AES-128 spec'd in big-endian throughout. Crypto.Cipher.AES treats key as
    # raw bytes; we just have to feed everything in spec order.
    # r' = 0b00..0 || prand (16 bytes total, big-endian, prand in low 3 bytes)
    r_prime = b"\x00" * 13 + prand
    cipher = AES.new(irk, AES.MODE_ECB)
    out = cipher.encrypt(r_prime)
    # ah(k, r) = LSB_24 of AES-128(k, r') — i.e. the low 3 bytes of the 16-byte output
    return out[-3:]


def verify(irk_hex: str, rpa_hex: str) -> None:
    irk = normalize_hex(irk_hex)
    rpa = normalize_hex(rpa_hex)
    if len(irk) != 16:
        sys.exit(f"IRK must be 16 bytes, got {len(irk)}")
    if len(rpa) != 6:
        sys.exit(f"RPA must be 6 bytes, got {len(rpa)}")

    # BT Core Spec Vol 6 Part B 1.3.2.2: when the 48-bit address is written MSB-first,
    # the HIGH 24 bits are prand and the LOW 24 bits are hash. Bit 47:46 (top 2 bits)
    # of the address therefore equal bit 23:22 of prand, which must be 0b01 for an RPA.
    prand_bytes = rpa[0:3]
    hash_bytes = rpa[3:6]

    print(f"IRK:          {irk.hex(':')}")
    print(f"RPA:          {rpa.hex(':')}")
    print(f"  hash bytes: {hash_bytes.hex(':')}")
    print(f"  prand:      {prand_bytes.hex(':')}")

    top2 = (prand_bytes[0] >> 6) & 0b11
    print(f"  prand top2: 0b{top2:02b} (expected 0b01 for RPA)")
    if top2 != 0b01:
        print("  WARNING: this address is not an RPA per spec.")

    computed = ah(irk, prand_bytes)
    print(f"Computed hash:{computed.hex(':')}")
    print(f"Match:        {'YES ✓' if computed == hash_bytes else 'NO ✗'}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    verify(sys.argv[1], sys.argv[2])
