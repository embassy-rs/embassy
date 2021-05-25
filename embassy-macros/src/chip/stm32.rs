use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_path = embassy_prefix.append("embassy").path();
    let embassy_stm32_path = embassy_prefix.append("embassy_stm32").path();

    quote!(
        use #embassy_stm32_path::{clock::Clock};

        let p = #embassy_stm32_path::init(#config);

        /*
        let mut rtc = #embass::RTC::new(unsafe { <peripherals::TIM2 as #embassy_path::util::Steal>::steal() }, interrupt::take!(TIM2));
        let rtc = unsafe { make_static(&mut rtc) };
        rtc.start();
        let mut alarm = rtc.alarm0();

        unsafe { #embassy_path::time::set_clock(rtc) };

        let alarm = unsafe { make_static(&mut alarm) };
        executor.set_alarm(alarm);
        */
    )
}
