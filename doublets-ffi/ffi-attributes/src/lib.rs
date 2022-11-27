#![feature(box_syntax)]
#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]

mod expand;
mod prepare;

use proc_macro::{Level, Span};
use std::collections::HashMap;

use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    Attribute, Ident, ItemFn, LitStr, Token, Type,
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
    aliases: Punctuated<AliasLine, Token![,]>,
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
                let name = input.parse::<StrArg<kw::name>>()?.lit;
                args.name = Some(name);
            } else if lookahead.peek(kw::types) {
                if !args.aliases.is_empty() {
                    return Err(input.error("expected only a single `types` argument"));
                }
                let AliasArg { param, aliases, .. } = input.parse::<AliasArg>()?;
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

// custom_kw = "literal"
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StrArg<T> {
    kw: T,
    // track issue: https://github.com/dtolnay/syn/issues/1209
    eq: syn::token::Eq,
    lit: LitStr,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            kw: input.parse()?,
            eq: input.parse()?,
            lit: input.parse()?,
        })
    }
}

// MyType<i32> => mu_type_suffix
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct AliasLine {
    ty: Type,
    to: Token![=>],
    ident: Ident,
}

impl Parse for AliasLine {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            to: input.parse()?,
            ident: input.parse()?,
        })
    }
}

// types::<G>(
//     u32 => integral,
//     f32 => floating,
//     (u32, Option<u32>) => magic,
// )
#[allow(dead_code)]
struct AliasArg {
    kw: kw::types,
    colon: Token![::],
    lt_token: Token![<],
    param: Ident,
    gt_toke: Token![>],
    paren_token: Paren,
    aliases: Punctuated<AliasLine, Token![,]>,
}

fn alias_validation(aliases: &Punctuated<AliasLine, Token![,]>) -> Result<(), syn::Error> {
    let mut map = HashMap::new();
    aliases.iter().try_for_each(|AliasLine { ty, ident, .. }| {
        if let Some(twice) = map.insert(ty.clone(), ident.clone()) {
            Err(syn::Error::new(
                ty.span().join(ident.span()).unwrap(),
                format!("tried to add alias to `{twice}` twice"),
            ))
        } else {
            Ok(())
        }
    })
}

impl Parse for AliasArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let new = Self {
            kw: input.parse()?,
            colon: input.parse()?,
            lt_token: input.parse()?,
            param: input.parse()?,
            gt_toke: input.parse()?,
            paren_token: syn::parenthesized!(content in input),
            aliases: content.parse_terminated(AliasLine::parse)?,
        };

        alias_validation(&new.aliases).map(|_| new)
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
    specialize_precise(args, item).unwrap_or_else(|_err| todo!())
}
