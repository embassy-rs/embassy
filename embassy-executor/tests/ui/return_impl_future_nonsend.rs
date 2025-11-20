#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

use core::future::Future;

use embassy_executor::SendSpawner;

#[embassy_executor::task]
fn task() -> impl Future<Output = ()> {
    // runs in spawning thread
    let non_send: *mut () = core::ptr::null_mut();
    async move {
        // runs in executor thread
        println!("{}", non_send as usize);
    }
}

fn send_spawn(s: SendSpawner) {
    s.spawn(task().unwrap());
}

fn main() {}
