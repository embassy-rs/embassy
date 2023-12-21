use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, ReturnType};

use crate::util::ctxt::Ctxt;

pub fn run(f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "test functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "test functions must not be generic");
    }

    let args = f.sig.inputs.clone();

    if args.len() > 1 {
        ctxt.error_spanned_by(f.sig.inputs, "test function must take zero or one (spawner) arguments");
    }

    if f.sig.output != ReturnType::Default {
        ctxt.error_spanned_by(&f.sig.output, "test functions must not return a value");
    }

    ctxt.check()?;

    let test_name = f.sig.ident;
    let task_fn_body = f.block;
    let embassy_test_name = format_ident!("__embassy_test_{}", test_name);
    let embassy_test_launcher = format_ident!("__embassy_test_launcher_{}", test_name);

    let invocation = if args.len() == 1 {
        quote! { #embassy_test_name(spawner).await }
    } else {
        quote! { #embassy_test_name().await }
    };

    let result = quote! {

        #[::embassy_executor::task]
        async fn #embassy_test_launcher(spawner: ::embassy_executor::Spawner, runner: &'static mut ::embassy_executor::_export_testutils::TestRunner) {
            #invocation;
            runner.done();
        }

        async fn #embassy_test_name(#args) {
            #task_fn_body
        }

        #[test]
        fn #test_name() {
            let r = ::embassy_executor::_export_testutils::TestRunner::default();

            let r1: &'static mut ::embassy_executor::_export_testutils::TestRunner = unsafe { core::mem::transmute(&r) };

            r1.initialize(|spawner| {
                let r2: &'static mut ::embassy_executor::_export_testutils::TestRunner = unsafe { core::mem::transmute(&r) };
                spawner.spawn(#embassy_test_launcher(spawner, r2)).unwrap();
            });
            r1.run_until_done();
        }
    };
    Ok(result.into())
}
