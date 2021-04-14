use crate::path::ModulePrefix;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromMeta, Default)]
pub struct Args {
    #[darling(default)]
    pub embassy_prefix: ModulePrefix,
}

pub fn generate(args: &Args) -> TokenStream {
    let embassy_rp_path = args.embassy_prefix.append("embassy_rp").path();
    quote!(
        use #embassy_rp_path::{interrupt, peripherals};

        let mut config = #embassy_rp_path::system::Config::default();
        unsafe { #embassy_rp_path::system::configure(config) };
    )
}
