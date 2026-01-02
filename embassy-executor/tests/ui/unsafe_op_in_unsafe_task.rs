#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]
#![deny(unsafe_op_in_unsafe_fn)]

#[embassy_executor::task]
async unsafe fn task() {
    let x = 5;
    (&x as *const i32).read();
}

fn main() {}
