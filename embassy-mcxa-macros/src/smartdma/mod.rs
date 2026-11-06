use std::path::{Component, Path};

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token, parenthesized};

mod encode;
mod parse;

struct Input {
    program_path: TokenStream,
    source: LitStr,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let program_path = content.parse()?;
        input.parse::<Token![,]>()?;
        let source = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("expected one assembly string literal"));
        }
        Ok(Self { program_path, source })
    }
}

pub(super) fn expand(input: TokenStream, from_file: bool) -> syn::Result<TokenStream> {
    let Input { program_path, source } = syn::parse2(input)?;
    let (text, name, dependency) = if from_file {
        let requested = source.value();
        // is_absolute() alone misses Windows drive-relative (`C:foo`) and
        // root-relative (`\foo`) paths, which can replace parts of the base.
        if Path::new(&requested)
            .components()
            .any(|component| matches!(component, Component::Prefix(_) | Component::RootDir))
        {
            return Err(syn::Error::new(
                source.span(),
                "assembly file path must be relative to CARGO_MANIFEST_DIR",
            ));
        }
        let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| syn::Error::new(source.span(), "CARGO_MANIFEST_DIR is not set"))?;
        let path = Path::new(&manifest_dir).join(&requested);
        let text = std::fs::read_to_string(&path).map_err(|error| {
            syn::Error::new(
                source.span(),
                format!("cannot read assembly file `{requested}`: {error}"),
            )
        })?;
        let absolute = path
            .to_str()
            .ok_or_else(|| syn::Error::new(source.span(), "assembly file path is not valid UTF-8"))?;
        let dependency = quote! { const _: &str = ::core::include_str!(#absolute); };
        (text, requested, dependency)
    } else {
        (source.value(), "<inline SmartDMA>".to_owned(), TokenStream::new())
    };
    let context = parse::Source {
        name: &name,
        span: source.span(),
    };
    let assembly = parse::parse(&text, &context)?;
    let image_words = u32::try_from(assembly.statements.len())
        .map_err(|_| syn::Error::new(source.span(), "program exceeds the u32 word-offset limit"))?;
    let mut words = Vec::new();
    let mut relocations = Vec::new();
    for statement in &assembly.statements {
        let encoded = encode::encode(statement, &assembly.labels, image_words, &context)?;
        if let Some((target, kind)) = encoded.relocation {
            let index = u32::try_from(words.len())
                .map_err(|_| context.error(statement.location, "program exceeds the u32 word-offset limit"))?;
            let kind = match kind {
                encode::RelocationKind::Absolute32 => quote!(#program_path::RelocationKind::Absolute32),
                encode::RelocationKind::Goto => quote!(#program_path::RelocationKind::Goto),
            };
            relocations.push(quote!(#program_path::Relocation::new(#index, #target, #kind)));
        }
        words.push(encoded.word);
    }
    let entries = assembly
        .entries
        .iter()
        .map(|(name, offset)| quote!(#program_path::Entry::new(#name, #offset)));
    Ok(quote! {{
        #dependency
        // Force constant evaluation even when the invocation is used in a
        // runtime `let`: arrays containing const-constructor calls otherwise
        // need not be promoted to the static lifetime required by Program.
        const PROGRAM: #program_path::Program = #program_path::Program::from_parts(
            #program_path::Isa::EzhB,
            &[#(#words),*],
            &[#(#entries),*],
            &[#(#relocations),*],
        );
        PROGRAM
    }})
}
