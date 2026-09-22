//! Strong-override of the ST full host's RPA resolver. Always compiled.
//!
//! The ST full BLE host (`libstm32wba_ble_stack_full.a`) resolves a peer's
//! Resolvable Private Address in `RndAddr_Check_Resolvable_Address`
//! (`random_addr.o`). That function drives the platform AES-ECB callback with the
//! IRK and the 24-bit random part arranged differently from the Core Spec
//! definition of `ah()`, so it cannot reproduce a real peer address and reports
//! "no match" for a valid RPA. The consequence is
//! `ACI_GAP_ADDR_NOT_RESOLVED_EVENT` on every bonded reconnect from a central
//! that uses a Resolvable Private Address, and the stored bond is never reused.
//!
//! The symbol is **weak** in the shipped archive and is referenced only by
//! `sdb.o`, so this strong definition replaces it at link time without modifying
//! ST's library in any way.
//!
//! Overriding a symbol of the certified stack means an image built from this
//! fork is no longer the certified configuration. That is deliberate: without
//! this override the host does not resolve an RPA at all, so a bonded reconnect
//! from a central that uses one fails.

unsafe extern "C" {
    /// Platform AES-128-ECB, implemented by the WPAN integration crate.
    fn BLEPLAT_AesEcbEncrypt(key: *const u8, input: *const u8, output: *mut u8);
}

/// `ah()` per Core Spec Vol 3 Part H 2.2.2, in the byte arrangement the spec's
/// own sample data uses:
///
/// * AES key   — the IRK with its bytes reversed relative to the on-air order
///   the host stores it in;
/// * AES block — 104 zero bits followed by the 24-bit random part (the leading
///   three octets of the displayed address);
/// * result    — the low 24 bits of the AES output.
///
/// `addr` is the 6-byte address in on-air order: `addr[0..2]` is the address
/// hash (least-significant octet first) and `addr[3..5]` the random part.
///
/// Returns 0 on match and non-zero otherwise, matching both the `memcmp` the
/// original returns and the caller's `cmp r0, #0` / `bne`.
///
/// # Safety
///
/// `irk` must point to 16 readable bytes and `addr` to 6 readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RndAddr_Check_Resolvable_Address(irk: *const u8, addr: *const u8) -> i32 {
    if irk.is_null() || addr.is_null() {
        return 1;
    }

    let mut key = [0u8; 16];
    let mut block = [0u8; 16];
    let mut out = [0u8; 16];

    for i in 0..16 {
        key[i] = *irk.add(15 - i);
    }

    // 104 zero bits, then the 24-bit random part in spec order.
    block[13] = *addr.add(5);
    block[14] = *addr.add(4);
    block[15] = *addr.add(3);

    BLEPLAT_AesEcbEncrypt(key.as_ptr(), block.as_ptr(), out.as_mut_ptr());

    // Compare the on-air hash with the low 24 bits of the output, least
    // significant octet first.
    let d0 = (out[15] != *addr.add(0)) as i32;
    let d1 = (out[14] != *addr.add(1)) as i32;
    let d2 = (out[13] != *addr.add(2)) as i32;
    d0 | d1 | d2
}
