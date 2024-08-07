use darling::ast::{Data, Fields, Style};
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
struct EnumFromDarling {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumVariants, ()>,
}

#[derive(Debug, FromVariant)]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<EnumVariantFields>,
}

#[derive(Debug, FromField)]
struct EnumVariantFields {
    ty: syn::Type,
}

pub(crate) fn process_enum_from_darling(input: DeriveInput) -> TokenStream {
    let EnumFromDarling {
        ident,
        generics,
        data: Data::Enum(variants),
    } = EnumFromDarling::from_derive_input(&input).expect("Can not parse input.")
    else {
        panic!("EnumFromDarling only works on enums.")
    };

    let impls = variants.iter().map(|variant| {
        let var_ident = &variant.ident;
        let style = &variant.fields.style;

        match style {
            Style::Tuple if variant.fields.len() == 1 => {
                let field = variant
                    .fields
                    .iter()
                    .next()
                    .expect("Should have one field.");
                let ty = &field.ty;

                quote! {
                    impl #generics From<#ty> for #ident #generics {
                        fn from(value: #ty) -> Self {
                            #ident::#var_ident(value)
                        }
                    }
                }
            }
            _ => quote! {},
        }
    });

    quote! {
        #(#impls)*
    }
}
