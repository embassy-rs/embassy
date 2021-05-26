cfg_if::cfg_if! {
    if #[cfg(feature = "_stm32h7")] {
        mod h7;
        pub use h7::*;
    } else if #[cfg(feature = "_stm32l0")] {
        mod l0;
        pub use l0::*;
    } else {
        #[derive(Default)]
        pub struct Config {}
        pub unsafe fn init(_config: Config) {
        }
    }
}
