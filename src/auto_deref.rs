use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deref))]
struct AutoDerefStruct {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), AutoDerefFields>,
    field: Option<syn::Ident>,
    mutable: Option<bool>,
}

#[derive(Debug, FromField)]
struct AutoDerefFields {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub(crate) fn process_auto_deref(input: DeriveInput) -> TokenStream {
    let AutoDerefStruct {
        ident,
        generics,
        data: Data::Struct(fields),
        field,
        mutable,
    } = AutoDerefStruct::from_derive_input(&input).expect("can not parse input.")
    else {
        panic!("AutoDeref only works on struct.")
    };

    let (fd, ty) = if let Some(field) = field {
        match fields.iter().find(|f| f.ident.as_ref().unwrap() == &field) {
            Some(f) => (field, &f.ty),
            None => panic!("Field {:?} not found in the data structure.", field),
        }
    } else if fields.len() == 1 {
        let f = fields.iter().next().unwrap();
        (f.ident.as_ref().unwrap().clone(), &f.ty)
    } else {
        panic!("AutoDeref only works on structs with 1 field or with field attribute.")
    };

    let mut impls = vec![quote! {
        impl #generics std::ops::Deref for #ident #generics {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {
                &self.#fd
            }
        }
    }];

    if mutable.unwrap_or(false) {
        impls.push(quote! {
            impl #generics std::ops::DerefMut for #ident #generics {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#fd
                }
            }
        })
    }

    quote! {#(#impls)*}
}
