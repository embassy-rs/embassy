#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

struct Foo<T>(T);

#[embassy_executor::task]
async fn foo(_x: Foo<impl Sized + 'static>) {}

fn main() {}
