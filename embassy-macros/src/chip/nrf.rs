use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_path = embassy_prefix.append("embassy").path();
    let embassy_nrf_path = embassy_prefix.append("embassy_nrf").path();

    quote!(
        use #embassy_nrf_path::{interrupt, peripherals, rtc};

        unsafe { #embassy_nrf_path::system::configure(#config) };

        let mut rtc = rtc::RTC::new(unsafe { <peripherals::RTC1 as #embassy_path::util::Steal>::steal() }, interrupt::take!(RTC1));
        let rtc = unsafe { make_static(&mut rtc) };
        rtc.start();
        let mut alarm = rtc.alarm0();

        unsafe { #embassy_path::time::set_clock(rtc) };

        let alarm = unsafe { make_static(&mut alarm) };
        executor.set_alarm(alarm);
    )
}
