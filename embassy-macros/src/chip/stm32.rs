use crate::path::ModulePrefix;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(embassy_prefix: &ModulePrefix, config: syn::Expr) -> TokenStream {
    let embassy_stm32_path = embassy_prefix.append("embassy_stm32").path();

    quote!(
        let p = #embassy_stm32_path::init(#config);
    )
}
