use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::util::ctxt::Ctxt;

#[derive(Debug, FromMeta)]
struct Args {
    #[allow(unused)]
    #[darling(default)]
    config: Option<syn::LitStr>,
}

pub fn run(args: syn::AttributeArgs, f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    #[allow(unused_variables)]
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let fargs = f.sig.inputs.clone();

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "task functions must not be generic");
    }

    #[cfg(feature = "stm32")]
    let hal = Some(quote!(::embassy_stm32));
    #[cfg(feature = "nrf")]
    let hal = Some(quote!(::embassy_nrf));
    #[cfg(feature = "rp")]
    let hal = Some(quote!(::embassy_rp));
    #[cfg(not(any(feature = "stm32", feature = "nrf", feature = "rp")))]
    let hal: Option<TokenStream> = None;

    if hal.is_some() && fargs.len() != 2 {
        ctxt.error_spanned_by(&f.sig, "main function must have 2 arguments");
    }
    if hal.is_none() && fargs.len() != 1 {
        ctxt.error_spanned_by(&f.sig, "main function must have 1 argument");
    }

    ctxt.check()?;

    let f_body = f.block;

    #[cfg(feature = "wasm")]
    let main = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() -> Result<(), wasm_bindgen::JsValue> {
            static EXECUTOR: ::embassy_util::Forever<::embassy_executor::executor::Executor> = ::embassy_util::Forever::new();
            let executor = EXECUTOR.put(::embassy_executor::executor::Executor::new());

            executor.start(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            });

            Ok(())
        }
    };

    #[cfg(all(feature = "std", not(feature = "wasm")))]
    let main = quote! {
        fn main() -> ! {
            let mut executor = ::embassy_executor::executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    };

    #[cfg(all(not(feature = "std"), not(feature = "wasm")))]
    let main = {
        let config = args.config.map(|s| s.parse::<syn::Expr>().unwrap()).unwrap_or_else(|| {
            syn::Expr::Verbatim(quote! {
                Default::default()
            })
        });

        let (hal_setup, peris_arg) = match hal {
            Some(hal) => (
                quote!(
                    let p = #hal::init(#config);
                ),
                quote!(p),
            ),
            None => (quote!(), quote!()),
        };

        quote! {
            #[cortex_m_rt::entry]
            fn main() -> ! {
                #hal_setup

                let mut executor = ::embassy_executor::executor::Executor::new();
                let executor = unsafe { __make_static(&mut executor) };

                executor.run(|spawner| {
                    spawner.must_spawn(__embassy_main(spawner, #peris_arg));
                })
            }
        }
    };

    let result = quote! {
        #[::embassy_executor::task()]
        async fn __embassy_main(#fargs) {
            #f_body
        }

        unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
            ::core::mem::transmute(t)
        }

        #main
    };

    Ok(result)
}
