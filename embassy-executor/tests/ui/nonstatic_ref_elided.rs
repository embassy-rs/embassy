#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

#[embassy_executor::task]
async fn foo(_x: &u32) {}

fn main() {}
