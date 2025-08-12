use std::str::FromStr;

use darling::export::NestedMeta;
use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::visit::{self, Visit};
use syn::{Expr, ExprLit, Lit, LitInt, ReturnType, Type, Visibility};

use crate::util::*;

#[derive(Debug, FromMeta, Default)]
struct Args {
    #[darling(default)]
    pool_size: Option<syn::Expr>,
    /// Use this to override the `embassy_executor` crate path. Defaults to `::embassy_executor`.
    #[darling(default)]
    embassy_executor: Option<syn::Expr>,
}

pub fn run(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut errors = TokenStream::new();

    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let f: ItemFn = match syn::parse2(item.clone()) {
        Ok(x) => x,
        Err(e) => return token_stream_with_error(item, e),
    };

    let args = match NestedMeta::parse_meta_list(args) {
        Ok(x) => x,
        Err(e) => return token_stream_with_error(item, e),
    };

    let args = match Args::from_list(&args) {
        Ok(x) => x,
        Err(e) => {
            errors.extend(e.write_errors());
            Args::default()
        }
    };

    let pool_size = args.pool_size.unwrap_or(Expr::Lit(ExprLit {
        attrs: vec![],
        lit: Lit::Int(LitInt::new("1", Span::call_site())),
    }));

    let embassy_executor = args
        .embassy_executor
        .unwrap_or(Expr::Verbatim(TokenStream::from_str("::embassy_executor").unwrap()));

    let returns_impl_trait = match &f.sig.output {
        ReturnType::Type(_, ty) => matches!(**ty, Type::ImplTrait(_)),
        _ => false,
    };
    if f.sig.asyncness.is_none() && !returns_impl_trait {
        error(&mut errors, &f.sig, "task functions must be async");
    }
    if !f.sig.generics.params.is_empty() {
        error(&mut errors, &f.sig, "task functions must not be generic");
    }
    if !f.sig.generics.where_clause.is_none() {
        error(&mut errors, &f.sig, "task functions must not have `where` clauses");
    }
    if !f.sig.abi.is_none() {
        error(&mut errors, &f.sig, "task functions must not have an ABI qualifier");
    }
    if !f.sig.variadic.is_none() {
        error(&mut errors, &f.sig, "task functions must not be variadic");
    }
    if f.sig.asyncness.is_some() {
        match &f.sig.output {
            ReturnType::Default => {}
            ReturnType::Type(_, ty) => match &**ty {
                Type::Tuple(tuple) if tuple.elems.is_empty() => {}
                Type::Never(_) => {}
                _ => error(
                    &mut errors,
                    &f.sig,
                    "task functions must either not return a value, return `()` or return `!`",
                ),
            },
        }
    }

    let mut args = Vec::new();
    let mut fargs = f.sig.inputs.clone();

    for arg in fargs.iter_mut() {
        match arg {
            syn::FnArg::Receiver(_) => {
                error(&mut errors, arg, "task functions must not have `self` arguments");
            }
            syn::FnArg::Typed(t) => {
                check_arg_ty(&mut errors, &t.ty);
                match t.pat.as_mut() {
                    syn::Pat::Ident(id) => {
                        id.mutability = None;
                        args.push((id.clone(), t.attrs.clone()));
                    }
                    _ => {
                        error(
                            &mut errors,
                            arg,
                            "pattern matching in task arguments is not yet supported",
                        );
                    }
                }
            }
        }
    }

    // Copy the generics + where clause to avoid more spurious errors.
    let generics = &f.sig.generics;
    let where_clause = &f.sig.generics.where_clause;
    let unsafety = &f.sig.unsafety;
    let visibility = &f.vis;

    // assemble the original input arguments,
    // including any attributes that may have
    // been applied previously
    let mut full_args = Vec::new();
    for (arg, cfgs) in args {
        full_args.push(quote!(
            #(#cfgs)*
            #arg
        ));
    }

    let task_ident = f.sig.ident.clone();
    let task_inner_ident = format_ident!("__{}_task", task_ident);

    let task_inner_future_output = match &f.sig.output {
        ReturnType::Default => quote! {-> impl ::core::future::Future<Output = ()>},
        // Special case the never type since we can't stuff it into a `impl Future<Output = !>`
        ReturnType::Type(arrow, maybe_never)
            if f.sig.asyncness.is_some() && matches!(**maybe_never, Type::Never(_)) =>
        {
            quote! {
                #arrow impl ::core::future::Future<Output=#embassy_executor::_export::Never>
            }
        }
        ReturnType::Type(arrow, maybe_never) if matches!(**maybe_never, Type::Never(_)) => quote! {
            #arrow #maybe_never
        },
        // Grab the arrow span, why not
        ReturnType::Type(arrow, typ) if f.sig.asyncness.is_some() => quote! {
            #arrow impl ::core::future::Future<Output = #typ>
        },
        // We assume that if `f` isn't async, it must return `-> impl Future<...>`
        // This is checked using traits later
        ReturnType::Type(arrow, typ) => quote! {
            #arrow #typ
        },
    };

    // We have to rename the function since it might be recursive;
    let mut task_inner_function = f.clone();
    let task_inner_function_ident = format_ident!("__{}_task_inner_function", task_ident);
    task_inner_function.sig.ident = task_inner_function_ident.clone();
    task_inner_function.vis = Visibility::Inherited;

    let task_inner_body = if errors.is_empty() {
        quote! {
            #task_inner_function

            // SAFETY: All the preconditions to `#task_ident` apply to
            //         all contexts `#task_inner_ident` is called in
            #unsafety {
                #task_inner_function_ident(#(#full_args,)*)
            }
        }
    } else {
        quote! {
            async {::core::todo!()}
        }
    };

    let task_inner = quote! {
        #visibility fn #task_inner_ident #generics (#fargs)
        #task_inner_future_output
        #where_clause
        {
            #task_inner_body
        }
    };

    let spawn = if returns_impl_trait {
        quote!(spawn)
    } else {
        quote!(_spawn_async_fn)
    };

    #[cfg(feature = "nightly")]
    let mut task_outer_body = quote! {
        trait _EmbassyInternalTaskTrait {
            type Fut: ::core::future::Future<Output: #embassy_executor::_export::TaskReturnValue> + 'static;
            fn construct(#fargs) -> Self::Fut;
        }

        impl _EmbassyInternalTaskTrait for () {
            type Fut = impl core::future::Future<Output: #embassy_executor::_export::TaskReturnValue> + 'static;
            fn construct(#fargs) -> Self::Fut {
                #task_inner_ident(#(#full_args,)*)
            }
        }

        const POOL_SIZE: usize = #pool_size;
        static POOL: #embassy_executor::raw::TaskPool<<() as _EmbassyInternalTaskTrait>::Fut, POOL_SIZE> = #embassy_executor::raw::TaskPool::new();
        unsafe { POOL.#spawn(move || <() as _EmbassyInternalTaskTrait>::construct(#(#full_args,)*)) }
    };
    #[cfg(not(feature = "nightly"))]
    let mut task_outer_body = quote! {
        const fn __task_pool_get<F, Args, Fut>(_: F) -> &'static #embassy_executor::raw::TaskPool<Fut, POOL_SIZE>
        where
            F: #embassy_executor::_export::TaskFn<Args, Fut = Fut>,
            Fut: ::core::future::Future + 'static,
        {
            unsafe { &*POOL.get().cast() }
        }

        const POOL_SIZE: usize = #pool_size;
        static POOL: #embassy_executor::_export::TaskPoolHolder<
            {#embassy_executor::_export::task_pool_size::<_, _, _, POOL_SIZE>(#task_inner_ident)},
            {#embassy_executor::_export::task_pool_align::<_, _, _, POOL_SIZE>(#task_inner_ident)},
        > = unsafe { ::core::mem::transmute(#embassy_executor::_export::task_pool_new::<_, _, _, POOL_SIZE>(#task_inner_ident)) };
        unsafe { __task_pool_get(#task_inner_ident).#spawn(move || #task_inner_ident(#(#full_args,)*)) }
    };

    let task_outer_attrs = &f.attrs;

    if !errors.is_empty() {
        task_outer_body = quote! {
            #![allow(unused_variables, unreachable_code)]
            let _x: #embassy_executor::SpawnToken<()> = ::core::todo!();
            _x
        };
    }

    let result = quote! {
        // This is the user's task function, renamed.
        // We put it outside the #task_ident fn below, because otherwise
        // the items defined there (such as POOL) would be in scope
        // in the user's code.
        #[doc(hidden)]
        #task_inner

        #(#task_outer_attrs)*
        #visibility #unsafety fn #task_ident #generics (#fargs) -> #embassy_executor::SpawnToken<impl Sized> #where_clause{
            #task_outer_body
        }

        #errors
    };

    result
}

fn check_arg_ty(errors: &mut TokenStream, ty: &Type) {
    struct Visitor<'a> {
        errors: &'a mut TokenStream,
    }

    impl<'a, 'ast> Visit<'ast> for Visitor<'a> {
        fn visit_type_reference(&mut self, i: &'ast syn::TypeReference) {
            // Only check for elided lifetime here. If not elided, it's checked by `visit_lifetime`.
            if i.lifetime.is_none() {
                error(
                    self.errors,
                    i.and_token,
                    "Arguments for tasks must live forever. Try using the `'static` lifetime.",
                )
            }
            visit::visit_type_reference(self, i);
        }

        fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
            if i.ident.to_string() != "static" {
                error(
                    self.errors,
                    i,
                    "Arguments for tasks must live forever. Try using the `'static` lifetime.",
                )
            }
        }

        fn visit_type_impl_trait(&mut self, i: &'ast syn::TypeImplTrait) {
            error(self.errors, i, "`impl Trait` is not allowed in task arguments. It is syntax sugar for generics, and tasks can't be generic.");
        }
    }

    Visit::visit_type(&mut Visitor { errors }, ty);
}
