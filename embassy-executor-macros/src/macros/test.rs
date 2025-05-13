use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let gen = quote! {
        #[test]
        fn #fn_name() {
            embassy_executor::run(|spawner| async move {
                #input_fn
            });
        }
    };

    gen.into()
}
