pub trait Steal {
    unsafe fn steal() -> Self;
}
