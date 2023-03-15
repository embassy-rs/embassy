use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, ItemFn, ReturnType, Type};

use crate::util::ctxt::Ctxt;

#[derive(Debug, FromMeta)]
struct Args {
    #[darling(default)]
    pool_size: Option<usize>,
}

pub fn run(args: syn::AttributeArgs, f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    let args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let pool_size: usize = args.pool_size.unwrap_or(1);

    let ctxt = Ctxt::new();

    if f.sig.asyncness.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        ctxt.error_spanned_by(&f.sig, "task functions must not be generic");
    }
    if !f.sig.generics.where_clause.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must not have `where` clauses");
    }
    if !f.sig.abi.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must not have an ABI qualifier");
    }
    if !f.sig.variadic.is_none() {
        ctxt.error_spanned_by(&f.sig, "task functions must not be variadic");
    }
    match &f.sig.output {
        ReturnType::Default => {}
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => {}
            Type::Never(_) => {}
            _ => ctxt.error_spanned_by(
                &f.sig,
                "task functions must either not return a value, return `()` or return `!`",
            ),
        },
    }

    if pool_size < 1 {
        ctxt.error_spanned_by(&f.sig, "pool_size must be 1 or greater");
    }

    let mut arg_names = Vec::new();
    let mut fargs = f.sig.inputs.clone();

    for arg in fargs.iter_mut() {
        match arg {
            syn::FnArg::Receiver(_) => {
                ctxt.error_spanned_by(arg, "task functions must not have receiver arguments");
            }
            syn::FnArg::Typed(t) => match t.pat.as_mut() {
                syn::Pat::Ident(id) => {
                    arg_names.push(id.ident.clone());
                    id.mutability = None;
                }
                _ => {
                    ctxt.error_spanned_by(arg, "pattern matching in task arguments is not yet supported");
                }
            },
        }
    }

    ctxt.check()?;

    let task_ident = f.sig.ident.clone();
    let task_inner_ident = format_ident!("__{}_task", task_ident);

    let mut task_inner = f;
    let visibility = task_inner.vis.clone();
    task_inner.vis = syn::Visibility::Inherited;
    task_inner.sig.ident = task_inner_ident.clone();

    let mut task_outer: ItemFn = parse_quote! {
        #visibility fn #task_ident(#fargs) -> ::embassy_executor::SpawnToken<impl Sized> {
            type Fut = impl ::core::future::Future + 'static;
            static POOL: ::embassy_executor::raw::TaskPool<Fut, #pool_size> = ::embassy_executor::raw::TaskPool::new();
            unsafe { POOL._spawn_async_fn(move || #task_inner_ident(#(#arg_names,)*)) }
        }
    };

    task_outer.attrs.append(&mut task_inner.attrs.clone());

    let result = quote! {
        // This is the user's task function, renamed.
        // We put it outside the #task_ident fn below, because otherwise
        // the items defined there (such as POOL) would be in scope
        // in the user's code.
        #[doc(hidden)]
        #task_inner

        #task_outer
    };

    Ok(result)
}
