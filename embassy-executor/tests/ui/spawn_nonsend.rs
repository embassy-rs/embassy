#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

use embassy_executor::SendSpawner;

#[embassy_executor::task]
async fn task(non_send: *mut ()) {
    println!("{}", non_send as usize);
}

fn send_spawn(s: SendSpawner) {
    s.spawn(task(core::ptr::null_mut()).unwrap());
}

fn main() {}
