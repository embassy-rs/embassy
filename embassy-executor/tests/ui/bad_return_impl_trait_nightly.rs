#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]
use core::future::Future;

#[embassy_executor::task]
fn task() -> impl Future<Output = u32> {
    async { 5 }
}

fn main() {}
