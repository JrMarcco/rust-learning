use crate::{
    auto_debug::process_auto_debug, auto_deref::process_auto_deref, enum_from::process_enum_from,
    enum_from_darling::process_enum_from_darling,
};
use proc_macro::TokenStream;
use syn::DeriveInput;

mod auto_debug;
mod auto_deref;
mod enum_from;
mod enum_from_darling;

#[proc_macro_derive(EnumFrom)]
pub fn derive_enum_from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    process_enum_from(input).into()
}

#[proc_macro_derive(EnumFromDarling)]
pub fn derive_enum_from_darling(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    process_enum_from_darling(input).into()
}

#[proc_macro_derive(AutoDeref, attributes(deref))]
pub fn derive_auto_derref(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    process_auto_deref(input).into()
}

#[proc_macro_derive(AutoDebug, attributes(debug))]
pub fn derive_auto_debug(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    process_auto_debug(input).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_for_git_commit() {
        assert_eq!(1 + 1, 2);
    }
}
