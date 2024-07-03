use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
struct AutoDebugStruct {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), AutoDebugFields>,
}

#[derive(Debug, FromField)]
#[darling(attributes(debug))]
struct AutoDebugFields {
    ident: Option<syn::Ident>,
    skip: Option<bool>,
}

pub(crate) fn process_auto_debug(input: DeriveInput) -> TokenStream {
    let AutoDebugStruct {
        ident,
        generics,
        data: Data::Struct(fields),
    } = AutoDebugStruct::from_derive_input(&input).expect("Can not parse input.")
    else {
        panic!("AutoDebug only works on struct.")
    };

    let fds = fields.iter().map(|field| {
        if field.skip.unwrap_or(false) {
            quote! {}
        } else {
            let ident = field.ident.as_ref().unwrap();
            quote! {
                .field(stringify!(#ident), &self.#ident)
            }
        }
    });

    quote! {
        impl ::core::fmt::Debug for #ident #generics {
            #[inline]
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.debug_struct(stringify!(#ident))
                #(#fds)*
                .finish()
            }
        }
    }
}
