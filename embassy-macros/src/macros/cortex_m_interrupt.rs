use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use std::iter;
use syn::ReturnType;
use syn::{Type, Visibility};

use crate::util::ctxt::Ctxt;

#[derive(Debug, FromMeta)]
struct Args {}

pub fn run(args: syn::AttributeArgs, mut f: syn::ItemFn) -> Result<TokenStream, TokenStream> {
    let _args = Args::from_list(&args).map_err(|e| e.write_errors())?;

    let ident = f.sig.ident.clone();
    let ident_s = ident.to_string();

    // XXX should we blacklist other attributes?

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                Type::Never(..) => true,
                _ => false,
            },
        };

    let ctxt = Ctxt::new();

    if !valid_signature {
        ctxt.error_spanned_by(
            &f.sig,
            "`#[interrupt]` handlers must have signature `[unsafe] fn() [-> !]`",
        );
    }

    ctxt.check()?;

    f.block.stmts = iter::once(
        syn::parse2(quote! {{
            // Check that this interrupt actually exists
            let __irq_exists_check: interrupt::#ident;
        }})
        .unwrap(),
    )
    .chain(f.block.stmts)
    .collect();

    let result = quote!(
        #[doc(hidden)]
        #[export_name = #ident_s]
        #[allow(non_snake_case)]
        #f
    );

    Ok(result)
}
