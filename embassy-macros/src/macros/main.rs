use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::util::ctxt::Ctxt;
use crate::util::path::ModulePrefix;

#[cfg(feature = "stm32")]
const HAL: Option<&str> = Some("embassy_stm32");
#[cfg(feature = "nrf")]
const HAL: Option<&str> = Some("embassy_nrf");
#[cfg(feature = "rp")]
const HAL: Option<&str> = Some("embassy_rp");
#[cfg(not(any(feature = "stm32", feature = "nrf", feature = "rp")))]
const HAL: Option<&str> = None;

#[derive(Debug, FromMeta)]
struct Args {
    #[darling(default)]
    embassy_prefix: ModulePrefix,

    #[allow(unused)]
    #[darling(default)]
    config: Option<syn::LitStr>,
}

pub fn run(args: syn::AttributeArgs, f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let fargs = f.sig.inputs.clone();

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "task functions must not be generic");
    }

    if HAL.is_some() && fargs.len() != 2 {
        ctxt.error_spanned_by(&f.sig, "main function must have 2 arguments");
    }
    if HAL.is_none() && fargs.len() != 1 {
        ctxt.error_spanned_by(&f.sig, "main function must have 1 argument");
    }

    ctxt.check()?;

    let embassy_prefix = args.embassy_prefix;
    let embassy_prefix_lit = embassy_prefix.literal();
    let embassy_path = embassy_prefix.append("embassy").path();
    let f_body = f.block;

    #[cfg(feature = "wasm")]
    let main = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() -> Result<(), wasm_bindgen::JsValue> {
            static EXECUTOR: #embassy_path::util::Forever<#embassy_path::executor::Executor> = #embassy_path::util::Forever::new();
            let executor = EXECUTOR.put(#embassy_path::executor::Executor::new());

            executor.start(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            });

            Ok(())
        }
    };

    #[cfg(all(feature = "std", not(feature = "wasm")))]
    let main = quote! {
        fn main() -> ! {
            let mut executor = #embassy_path::executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    };

    #[cfg(all(not(feature = "std"), not(feature = "wasm")))]
    let main = {
        let config = args
            .config
            .map(|s| s.parse::<syn::Expr>().unwrap())
            .unwrap_or_else(|| {
                syn::Expr::Verbatim(quote! {
                    Default::default()
                })
            });

        let (hal_setup, peris_arg) = match HAL {
            Some(hal) => {
                let embassy_hal_path = embassy_prefix.append(hal).path();
                (
                    quote!(
                        let p = #embassy_hal_path::init(#config);
                    ),
                    quote!(p),
                )
            }
            None => (quote!(), quote!()),
        };

        quote! {
            #[cortex_m_rt::entry]
            fn main() -> ! {
                #hal_setup

                let mut executor = #embassy_path::executor::Executor::new();
                let executor = unsafe { __make_static(&mut executor) };

                executor.run(|spawner| {
                    spawner.must_spawn(__embassy_main(spawner, #peris_arg));
                })
            }
        }
    };

    let result = quote! {
        #[#embassy_path::task(embassy_prefix = #embassy_prefix_lit)]
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
