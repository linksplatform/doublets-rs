use proc_macro2::Ident;
use syn::{
    GenericArgument, ParenthesizedGenericArguments, PathArguments, ReturnType, Type, TypePath,
};

pub(crate) fn prepare_path(mut path: TypePath, from: &Ident, to: &Type) -> Type {
    if path.path.is_ident(from) {
        return to.clone();
    }
    path.path.segments.iter_mut().for_each(|seg| {
        match &mut seg.arguments {
            PathArguments::AngleBracketed(angle) => {
                for arg in &mut angle.args {
                    match arg {
                        GenericArgument::Type(gty) => {
                            *gty = replace_ty_in_param(gty.clone(), from, to);
                        }
                        _ => { /* ignore */ }
                    }
                }
            }
            PathArguments::Parenthesized(ParenthesizedGenericArguments {
                inputs, output, ..
            }) => {
                for input in inputs {
                    *input = replace_ty_in_param(input.clone(), from, to);
                }
                if let ReturnType::Type(_, box ty) = output {
                    *ty = replace_ty_in_param(ty.clone(), from, to);
                }
            }
            _ => { /* ignore */ }
        }
    });
    Type::Path(path)
}

pub(crate) fn replace_ty_in_param(ty: Type, from: &Ident, to: &Type) -> Type {
    match ty {
        Type::Path(path) => prepare_path(path, from, to),
        Type::Array(mut arr) => {
            *arr.elem = replace_ty_in_param(*arr.elem, from, to);
            Type::Array(arr)
        }
        Type::Ptr(mut ptr) => {
            *ptr.elem = replace_ty_in_param(*ptr.elem, from, to);
            Type::Ptr(ptr)
        }
        Type::Reference(mut refer) => {
            *refer.elem = replace_ty_in_param(*refer.elem, from, to);
            Type::Reference(refer)
        }
        Type::Slice(mut slice) => {
            *slice.elem = replace_ty_in_param(*slice.elem, from, to);
            Type::Slice(slice)
        }
        Type::Tuple(mut tuple) => {
            for elem in &mut tuple.elems {
                *elem = replace_ty_in_param(elem.clone(), from, to);
            }
            Type::Tuple(tuple)
        }
        _ => {
            todo!()
        }
    }
}
