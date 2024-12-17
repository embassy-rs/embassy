#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

#[embassy_executor::task]
async fn foo(_x: impl Sized) {}

fn main() {}
