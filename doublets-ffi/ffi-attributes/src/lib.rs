#![feature(box_syntax)]
#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]

mod expand;
mod prepare;

use proc_macro::{Level, Span};
use std::{collections::HashMap, marker::PhantomData};

use darling::FromMeta;

use syn::punctuated::Punctuated;

use crate::kw::attributes;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token::Token,
    Attribute, Ident, ItemFn, LitStr, Token,
};

mod kw {
    syn::custom_keyword!(types);
    syn::custom_keyword!(name);
    syn::custom_keyword!(attributes);
}

#[derive(Clone, Default, Debug)]
struct SpecializeArgs {
    name: Option<LitStr>,
    param: Option<Ident>,
    aliases: HashMap<Ident, Ident>,
    attributes: Vec<Attribute>,
    /// Errors describing any unrecognized parse inputs that we skipped.
    parse_warnings: Vec<syn::Error>,
}

impl SpecializeArgs {
    pub(crate) fn warnings(&self) -> impl Iterator<Item = proc_macro::Diagnostic> + '_ {
        self.parse_warnings.iter().map(|err| {
            let msg = format!("found unrecognized input, {}", err);
            proc_macro::Diagnostic::spanned::<Vec<Span>, _>(
                vec![err.span().unwrap()],
                Level::Warning,
                msg,
            )
        })
    }
}

impl Parse for SpecializeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(input.error("expected only a single `name` argument"));
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                args.name = Some(name);
            } else if lookahead.peek(kw::types) {
                if !args.aliases.is_empty() {
                    return Err(input.error("expected only a single `types` argument"));
                }
                let AliasArg { param, aliases } = input.parse::<AliasArg>()?;
                args.param = Some(param);
                args.aliases = aliases;
            } else if lookahead.peek(kw::attributes) {
                if !args.attributes.is_empty() {
                    return Err(input.error("expected only a single `attributes` argument"));
                }
                let _ = input.parse::<kw::attributes>()?;
                let content;
                let _ = syn::parenthesized!(content in input);
                args.attributes = content.call(Attribute::parse_outer)?;
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                // Parse the unrecognized tokens stream,
                // and ignore it away so we can keep parsing.
                args.parse_warnings.push(lookahead.error());
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }

        Ok(args)
    }
}

struct StrArg<T> {
    value: LitStr,
    _marker: PhantomData<T>,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _marker: PhantomData,
        })
    }
}
struct AliasLine(Ident, Ident);

impl Parse for AliasLine {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty = input.parse::<Ident>()?;
        let _ = input.parse::<Token![=>]>()?;
        let str = input.parse::<Ident>()?;
        Ok(Self(ty, str))
    }
}

struct AliasArg {
    param: Ident,
    aliases: HashMap<Ident, Ident>,
}

impl Parse for AliasArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::types>()?;
        let _ = input.parse::<Token![::]>()?;
        let _ = input.parse::<Token![<]>()?;
        let param = input.parse::<Ident>()?;
        let _ = input.parse::<Token![>]>()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let aliases: Punctuated<AliasLine, Token![,]> =
            content.parse_terminated(AliasLine::parse)?;

        let mut map = HashMap::new();
        for AliasLine(ty, lit) in aliases {
            #[allow(clippy::map_entry)]
            if map.contains_key(&ty) {
                return Err(syn::Error::new(
                    ty.span().join(lit.span()).unwrap(),
                    "tried to ad lias to same field twice",
                ));
            } else {
                map.insert(ty, lit);
            }
        }

        Ok(Self {
            param,
            aliases: map,
        })
    }
}

fn specialize_precise(
    args: SpecializeArgs,
    item: proc_macro::TokenStream,
) -> syn::Result<proc_macro::TokenStream> {
    let input = syn::parse::<ItemFn>(item)?;
    Ok(expand::gen_function(input, args).into())
}

#[proc_macro_attribute]
pub fn specialize_for(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as SpecializeArgs);
    specialize_precise(args, item.clone()).unwrap_or_else(|_err| todo!())
}
