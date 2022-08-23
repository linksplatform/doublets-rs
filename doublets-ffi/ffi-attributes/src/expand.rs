use crate::{prepare, FromMeta, SpecializeArgs};
use proc_macro::Diagnostic;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use syn::{
    punctuated::Punctuated, FnArg, GenericParam, ItemFn, PatType, ReturnType, Signature, Token, TypeParam,
};

fn filter_generics<'gen>(
    generics: impl Iterator<Item = &'gen GenericParam> + 'gen,
    param: &'gen Ident,
) -> impl Iterator<Item = GenericParam> + 'gen {
    generics
        .filter(move |&par| {
            if let GenericParam::Type(TypeParam { ident, .. }) = par {
                ident != param
            } else {
                true
            }
        })
        .cloned()
}

fn prepare_fn_args<'args>(
    fn_args: impl Iterator<Item = &'args mut FnArg>,
    param: &Ident,
    ident: &Ident,
) {
    for arg in fn_args {
        match arg {
            FnArg::Typed(PatType { box ty, .. }) => {
                *ty = prepare::replace_ty_in_param(ty.clone(), param, ident);
            }
            FnArg::Receiver(_) => {
                todo!()
            }
        }
    }
}

fn prepare_output_type(output: &mut ReturnType, param: &Ident, ident: &Ident) {
    if let ReturnType::Type(_, box ty) = output {
        *ty = prepare::replace_ty_in_param(ty.clone(), param, ident);
    }
}

fn args_idents<'args>(
    args: impl Iterator<Item = &'args FnArg>,
) -> impl Iterator<Item = &'args dyn ToTokens> {
    args.map(|arg| match arg {
        FnArg::Typed(PatType { box pat, .. }) => pat as &dyn ToTokens,
        _ => {
            todo!()
        }
    })
}

fn build_fn(input: ItemFn, real_fn: &Ident, param: &Ident) -> TokenStream {
    let ItemFn {
        attrs, vis, sig, ..
    } = input;

    let Signature {
        output: return_type,
        inputs: params,
        unsafety,
        asyncness,
        constness,
        abi,
        ident,
        generics:
            syn::Generics {
                params: gen_params,
                where_clause,
                ..
            },
        ..
    } = sig;

    let args = args_idents(params.iter());
    quote! {
        #(#attrs) *
        #vis #constness #unsafety #asyncness #abi fn #ident<#gen_params>(#params) #return_type
        #where_clause
        {
            #real_fn::<#param>(#(#args),*)
        }
    }
}

fn gen_new_def(mut fn_list: TokenStream, input: ItemFn, args: SpecializeArgs) -> TokenStream {
    let real_name = input.sig.ident.to_string();
    let name_pat = args
        .name
        .map(|name| name.to_token_stream().to_string())
        .unwrap_or_else(|| real_name + "_*");
    let name_pat = name_pat.trim_matches('"');

    for (ty, lit) in args.aliases {
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = input.clone();

        let Signature {
            output: mut return_type,
            inputs: mut params,
            generics: syn::Generics {
                params: gen_params, ..
            },
            ..
        } = sig.clone();

        let param = args.param.as_ref().unwrap();
        let generics: Punctuated<_, Token![,]> =
            Punctuated::from_iter(filter_generics(gen_params.iter(), param));
        prepare_fn_args(params.iter_mut(), param, &ty);
        prepare_output_type(&mut return_type, param, &ty);

        let new_ident: Ident =
            Ident::from_string(&name_pat.replace('*', &lit.to_token_stream().to_string())).unwrap();

        let real_fn = sig.ident.clone();
        let sig = Signature {
            ident: new_ident,
            output: return_type,
            inputs: params,
            generics: syn::Generics {
                params: generics,
                ..sig.generics
            },
            ..sig
        };
        let new_fn = build_fn(
            ItemFn {
                attrs,
                vis,
                sig,
                block,
            },
            &real_fn,
            &lit,
        );
        fn_list = quote! {
            #fn_list
            #new_fn
        }
    }

    fn_list
}

pub(crate) fn gen_function(input: ItemFn, args: SpecializeArgs) -> TokenStream {
    args.warnings().for_each(Diagnostic::emit);

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input.clone();

    let Signature {
        output: return_type,
        inputs: params,
        unsafety,
        asyncness,
        constness,
        abi,
        ident,
        generics:
            syn::Generics {
                params: gen_params,
                where_clause,
                ..
            },
        ..
    } = sig.clone();

    let this = quote! {
        #(#attrs) *
        #vis #constness #unsafety #asyncness #abi fn #ident<#gen_params>(#params) #return_type
        #where_clause
        {
            #block
        }
    };

    gen_new_def(this, input, args)
}
