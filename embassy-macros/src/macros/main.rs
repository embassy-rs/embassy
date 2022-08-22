use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

use crate::util::ctxt::Ctxt;

#[derive(Debug, FromMeta)]
struct Args {}

pub fn run(args: syn::AttributeArgs, f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    #[allow(unused_variables)]
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let fargs = f.sig.inputs.clone();

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "main function must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "main function must not be generic");
    }

    if fargs.len() != 1 {
        ctxt.error_spanned_by(&f.sig, "main function must have 1 argument: the spawner.");
    }

    ctxt.check()?;

    let f_body = f.block;

    #[cfg(feature = "wasm")]
    let main = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() -> Result<(), wasm_bindgen::JsValue> {
            static EXECUTOR: ::embassy_executor::_export::StaticCell<::embassy_executor::Executor> = ::embassy_executor::_export::StaticCell::new();
            let executor = EXECUTOR.init(::embassy_executor::Executor::new());

            executor.start(|spawner| {
                spawner.spawn(__embassy_main(spawner)).unwrap();
            });

            Ok(())
        }
    };

    #[cfg(all(feature = "std", not(feature = "wasm")))]
    let main = quote! {
        fn main() -> ! {
            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
        }
    };

    #[cfg(all(not(feature = "std"), not(feature = "wasm")))]
    let main = quote! {
        #[cortex_m_rt::entry]
        fn main() -> ! {
            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { __make_static(&mut executor) };

            executor.run(|spawner| {
                spawner.must_spawn(__embassy_main(spawner));
            })
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
