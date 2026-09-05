#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use cipher::InOutBuf;

#[cfg(any(
    feature = "driver-md5",
    feature = "driver-sha1",
    feature = "driver-sha2",
    feature = "driver-hmac-sha1",
    feature = "driver-hmac-sha2",
    feature = "driver-aes128",
    feature = "driver-aes256",
    feature = "driver-aes128cbc",
    feature = "driver-aes256cbc",
    feature = "driver-aes128ctr",
    feature = "driver-aes256ctr",
    feature = "driver-aes128gcm",
    feature = "driver-aes256gcm",
    feature = "driver-aes128ccm",
    feature = "driver-aes256ccm",
    feature = "driver-aes128cmac",
    feature = "driver-aes256cmac",
))]
mod driver_rustcrypto;

#[cfg(feature = "ec")]
pub mod ec;

#[cfg(feature = "p256")]
pub mod p256;

mod aes;
mod hash;

pub use aes::*;
pub use hash::*;

#[inline]
fn unwrap_inout<'inp, 'out>(buf: InOutBuf<'inp, 'out, u8>) -> embassy_crypto_driver::InOutBuf<'inp, 'out, u8> {
    let len = buf.len();
    let (in_ptr, out_ptr) = buf.into_raw();
    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, len) }
}
