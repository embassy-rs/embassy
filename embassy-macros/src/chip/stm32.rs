use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_path = embassy_prefix.append("embassy").path();
    let embassy_stm32_path = embassy_prefix.append("embassy_stm32").path();

    quote!(
        use #embassy_stm32_path::{interrupt, peripherals, clock::Clock, time::Hertz};

        let (p, mut c) = #embassy_stm32_path::init(#config);

        let clock = unsafe { make_static(&mut c) };
        clock.start();

        let mut alarm = clock.alarm1();
        unsafe { #embassy_path::time::set_clock(clock) };

        let alarm = unsafe { make_static(&mut alarm) };
        executor.set_alarm(alarm);
    )
}
