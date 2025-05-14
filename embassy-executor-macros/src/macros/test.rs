use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

pub fn test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let task_name = format_ident!("__{}_task", fn_name);

    let gen = quote! {
        #[::embassy_executor::task()]
        #[allow(clippy::future_not_send)]
        async fn #task_name() {
            #input_fn
        }


        #[test]
        fn #fn_name() {
            let mut executor = ::embassy_executor::Executor::new();
            executor.run(|spawner| {
                spawner.spawn(#task_name()).unwrap();
            });
        }
    };

    gen.into()
}
