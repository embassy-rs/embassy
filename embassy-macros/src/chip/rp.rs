use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_path = embassy_prefix.append("embassy").path();
    let embassy_rp_path = embassy_prefix.append("embassy_rp").path();
    quote!(
        use #embassy_rp_path::{interrupt, peripherals};

        let p = #embassy_rp_path::init(#config);

        let alarm = unsafe { <#embassy_rp_path::peripherals::TIMER_ALARM0 as #embassy_path::util::Steal>::steal() };
        let mut alarm = #embassy_rp_path::timer::Alarm::new(alarm);
        let alarm = unsafe { make_static(&mut alarm) };
        executor.set_alarm(alarm);
    )
}
