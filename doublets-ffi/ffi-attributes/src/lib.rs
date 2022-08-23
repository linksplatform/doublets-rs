#![feature(box_syntax)]
#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]

mod expand;
mod prepare;

use proc_macro::{Level, Span};
use std::{collections::HashMap, marker::PhantomData};

use darling::FromMeta;


use syn::punctuated::Punctuated;


use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token::Token, Ident, ItemFn, LitStr, Token,
};

mod kw {
    syn::custom_keyword!(types);
    syn::custom_keyword!(name);
}

#[derive(Clone, Default, Debug)]
struct SpecializeArgs {
    name: Option<LitStr>,
    param: Option<Ident>,
    aliases: HashMap<Ident, Ident>,
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
                    "tried to skip the same field twice",
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

    /*
       let input = parse_macro_input!(input as ItemFn);
       let input_clone: ItemFn = input.clone();
       let ident = input.sig.ident;
       // TODO: use args
       let args = match MacroArgs::from_list(&attr_args) {
           Ok(v) => v,
           Err(e) => {
               return TokenStream::from(e.write_errors());
           }
       };
       //println!("{:?}", args.types);

       let inputs = input.sig.inputs;
       let generic_name = {
           let mut generics_names: Vec<_> = input
               .sig
               .generics
               .params
               .iter()
               .map(|param| match param {
                   GenericParam::Lifetime(_) => {
                       panic!("`lifetime` generic is not supported")
                   }
                   GenericParam::Const(_) => {
                       panic!("`const` generic is not supported")
                   }
                   GenericParam::Type(ty) => ty.ident.to_string(),
               })
               .collect();
           assert_eq!(generics_names.len(), 1);
           generics_names.remove(0)
       };

       let fn_pat = args.name;
       let asterisk_count = fn_pat.chars().filter(|c| *c == '*').count();
       assert_eq!(asterisk_count, 1);

       let mut out = quote! { #input_clone };

       for ty in args.types {
           let ty_str = ty.as_str();
           let ty_tt: proc_macro2::TokenStream = ty.parse().unwrap();
           let fn_pat: proc_macro2::TokenStream = fn_pat
               .replace(
                   '*',
                   match &args.convention {
                       Conventions::csharp => csharp_convention(ty.clone()),
                       Conventions::rust => rust_convention(ty.clone()),
                       _ => {
                           panic!("unknown convention")
                       }
                   }
                   .as_str(),
               )
               .parse()
               .unwrap();

           let mut inputs: Punctuated<FnArg, _> = inputs.clone();

           let output_ty: proc_macro2::TokenStream = match &input.sig.output {
               ReturnType::Default => "()".parse().unwrap(),
               ReturnType::Type(_, ty) => {
                   ty_from_to((**ty).clone(), &generic_name, ty_str).to_token_stream()
               }
           };

           inputs.iter_mut().for_each(|arg| match arg {
               FnArg::Receiver(_) => {
                   panic!("function with `self` is not supported")
               }
               FnArg::Typed(pat_type) => {
                   pat_type.ty = box ty_from_to(*(pat_type.ty).clone(), &generic_name, &ty);
               }
           });

           let _generic_name: proc_macro2::TokenStream = generic_name.parse().unwrap();
           let input_args: Vec<_> = inputs
               .iter()
               .map(|arg| match arg {
                   FnArg::Receiver(_) => {
                       unreachable!()
                   }
                   FnArg::Typed(ty) => ty.pat.to_token_stream(),
               })
               .collect();

           out = quote! {
               #out
               #[no_mangle]
               pub unsafe extern "C" fn #fn_pat(#inputs) -> #output_ty {
                   #ident::<#ty_tt>(#(#input_args),*)
               }
           };
       }

       //println!("{}", out);

       out.into()

    */
}
