use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_nrf_path = embassy_prefix.append("embassy_nrf").path();

    quote!(
        let p = #embassy_nrf_path::init(#config);
    )
}
