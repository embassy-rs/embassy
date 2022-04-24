use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::util::ctxt::Ctxt;
use crate::util::path::ModulePrefix;

#[derive(Debug, FromMeta)]
struct Args {
    #[darling(default)]
    pool_size: Option<usize>,
    #[darling(default)]
    send: bool,
    #[darling(default)]
    embassy_prefix: ModulePrefix,
}

pub fn run(args: syn::AttributeArgs, mut f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let embassy_prefix = args.embassy_prefix.append("embassy");
    let embassy_path = embassy_prefix.path();

    let pool_size: usize = args.pool_size.unwrap_or(1);

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "task functions must not be generic");
    }
    if pool_size < 1 {
        ctxt.error_spanned_by(&f.sig, "pool_size must be 1 or greater");
    }

    let mut arg_names: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
        syn::punctuated::Punctuated::new();
    let mut fargs = f.sig.inputs.clone();

    for arg in fargs.iter_mut() {
        match arg {
            syn::FnArg::Receiver(_) => {
                ctxt.error_spanned_by(arg, "task functions must not have receiver arguments");
            }
            syn::FnArg::Typed(t) => match t.pat.as_mut() {
                syn::Pat::Ident(i) => {
                    arg_names.push(i.ident.clone());
                    i.mutability = None;
                }
                _ => {
                    ctxt.error_spanned_by(
                        arg,
                        "pattern matching in task arguments is not yet supporteds",
                    );
                }
            },
        }
    }

    ctxt.check()?;

    let task_ident = f.sig.ident.clone();
    let task_inner_ident = format_ident!("__{}_task", task_ident);
    let future_ident = format_ident!("__{}_Future", task_ident);
    let pool_ident = format_ident!("__{}_POOL", task_ident);
    let new_ts_ident = format_ident!("__{}_NEW_TASKSTORAGE", task_ident);

    let visibility = &f.vis;
    f.sig.ident = task_inner_ident.clone();
    let impl_ty = if args.send {
        quote!(impl ::core::future::Future + Send + 'static)
    } else {
        quote!(impl ::core::future::Future + 'static)
    };

    let attrs = &f.attrs;

    let spawn_token = quote!(#embassy_path::executor::SpawnToken);
    let task_storage = quote!(#embassy_path::executor::raw::TaskStorage);

    let result = quote! {

        #[allow(non_camel_case_types)]
        type #future_ident = #impl_ty;

        #(#attrs)*
        #visibility fn #task_ident(#fargs) -> #spawn_token<#future_ident> {
            #f

            #[allow(non_upper_case_globals)]
            #[allow(clippy::declare_interior_mutable_const)]
            const #new_ts_ident: #task_storage<#future_ident> = #task_storage::new();

            #[allow(non_upper_case_globals)]
            static #pool_ident: [#task_storage<#future_ident>; #pool_size] = [#new_ts_ident; #pool_size];

            unsafe { #task_storage::spawn_pool(&#pool_ident, move || #task_inner_ident(#arg_names)) }
        }
    };

    Ok(result)
}
