#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

struct Foo<'a>(&'a ());

#[embassy_executor::task]
async fn task()
where
    (): Sized,
{
}

fn main() {}
