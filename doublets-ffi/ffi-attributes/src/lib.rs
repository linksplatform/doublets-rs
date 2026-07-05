
use proc_macro::TokenStream;

use darling::FromMeta;
use quote::{quote, ToTokens};

use syn::{punctuated::Punctuated};

use syn::{
    parse_macro_input, AttributeArgs, FnArg, GenericArgument, GenericParam, Ident, ItemFn,
    PathArguments, ReturnType, Type,
};

// Simple type mapping structure for the new syntax
// We'll handle parsing manually in the macro

fn csharp_convention(s: String) -> String {
    match s.as_str() {
        "i8" => "SByte",
        "u8" => "Byte",
        "i16" => "Int16",
        "u16" => "UInt16",
        "i32" => "Int32",
        "u32" => "UInt32",
        "i64" => "Int64",
        "u64" => "UInt64",
        s => {
            panic!("{} is incompatible with doublets-ffi type", s)
        }
    }
    .to_string()
}

#[derive(FromMeta, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
enum Conventions {
    csharp,
}

#[derive(FromMeta)]
struct MacroArgs {
    #[darling(default)]
    convention: Option<Conventions>,
    #[darling(multiple)]
    types: Vec<String>,
    name: String,
}

fn ty_from_to(ty: Type, from: &str, to: &str) -> Type {
    match ty {
        Type::Array(arr) => ty_from_to(*arr.elem, from, to),
        Type::Path(mut path) => {
            path.path.segments.iter_mut().for_each(|seg| {
                if seg.ident.to_string().as_str() == from {
                    seg.ident = Ident::from_string(to).unwrap();
                }
                match seg.arguments {
                    PathArguments::AngleBracketed(ref mut angle) => {
                        for arg in angle.args.iter_mut() {
                            match arg {
                                GenericArgument::Type(gty) => {
                                    *gty = ty_from_to(gty.clone(), from, to);
                                }
                                _ => {
                                    panic!("not doublets-ffi compatible generic")
                                }
                            }
                        }
                    }
                    PathArguments::Parenthesized(_) => {
                        todo!()
                    }
                    _ => { /* ignore */ }
                }
            });
            Type::Path(path)
        }
        Type::Ptr(mut ptr) => {
            *ptr.elem = ty_from_to(*ptr.elem, from, to);
            Type::Ptr(ptr)
        }
        Type::Reference(mut refer) => {
            *refer.elem = ty_from_to(*refer.elem, from, to);
            Type::Reference(refer)
        }
        _ => {
            panic!("unexpected doublets-ffi type");
        }
    }
}

#[proc_macro_attribute]
pub fn specialize_for(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
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

    // Handle the convention-based syntax (backward compatible)
    let type_pairs: Vec<(String, String)> = args.types.into_iter()
        .map(|ty| {
            let ffi_name = match &args.convention {
                Some(Conventions::csharp) => csharp_convention(ty.clone()),
                None => {
                    // Default to csharp convention if no convention specified
                    csharp_convention(ty.clone())
                }
            };
            (ty, ffi_name)
        })
        .collect();

    for (rust_type, ffi_name) in type_pairs {
        let ty_str = rust_type.as_str();
        let ty_tt: proc_macro2::TokenStream = rust_type.parse().unwrap();
        let fn_pat: proc_macro2::TokenStream = fn_pat
            .replace('*', &ffi_name)
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
                pat_type.ty = Box::new(ty_from_to(*(pat_type.ty).clone(), &generic_name, &rust_type));
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
}

#[proc_macro_attribute]
pub fn specialize(args: TokenStream, input: TokenStream) -> TokenStream {
    let name_pattern = parse_macro_input!(args as syn::LitStr);
    let input = parse_macro_input!(input as ItemFn);
    let input_clone: ItemFn = input.clone();
    let ident = input.sig.ident;
    
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

    let fn_pat = name_pattern.value();
    let asterisk_count = fn_pat.chars().filter(|c| *c == '*').count();
    assert_eq!(asterisk_count, 1);

    let mut out = quote! { #input_clone };

    // Default types for embedded annotations
    let default_types = vec![
        ("u8", "Byte"),
        ("u16", "UInt16"), 
        ("u32", "UInt32"),
        ("u64", "UInt64"),
    ];

    for (rust_type, ffi_name) in default_types {
        let ty_str = rust_type;
        let ty_tt: proc_macro2::TokenStream = rust_type.parse().unwrap();
        let fn_pat: proc_macro2::TokenStream = fn_pat
            .replace('*', ffi_name)
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
                pat_type.ty = Box::new(ty_from_to(*(pat_type.ty).clone(), &generic_name, rust_type));
            }
        });

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

    out.into()
}
