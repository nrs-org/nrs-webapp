mod utils;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

#[proc_macro_derive(Fields, attributes(field))]
pub fn derives_fields(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = ast.ident;

    // -- get the fields
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        panic!("Only support Struct")
    };

    // -- Collect Elements
    let props = utils::get_props(fields);

    let props_all_idents: Vec<&Option<Ident>> = props.iter().map(|p| p.ident).collect();
    let props_all_names: Vec<&String> = props.iter().map(|p| &p.name).collect();

    let props_option_idents: Vec<&Option<Ident>> = props
        .iter()
        .filter(|p| p.is_option)
        .map(|p| p.ident)
        .collect();
    let props_option_names: Vec<&String> = props
        .iter()
        .filter(|p| p.is_option)
        .map(|p| &p.name)
        .collect();

    let props_not_option_idents: Vec<&Option<Ident>> = props
        .iter()
        .filter(|p| !p.is_option)
        .map(|p| p.ident)
        .collect();
    let props_not_option_names: Vec<&String> = props
        .iter()
        .filter(|p| !p.is_option)
        .map(|p| &p.name)
        .collect();

    // -- Vec push code for the (name, value)
    let ff_all_pushes = quote! {
        #(
            ff.push(sqlbindable::Field::new(#props_all_names, self.#props_all_idents)?);
        )*
    };

    let ff_not_option_pushes = quote! {
        #(
            ff.push(sqlbindable::Field::new(#props_not_option_names, self.#props_not_option_idents)?);
        )*
    };

    let ff_option_not_none_pushes = quote! {
        #(
            if let Some(val) = self.#props_option_idents {
                ff.push(sqlbindable::Field::new(#props_option_names, val)?);
            }
        )*
    };

    // -- Compose the final code
    let output = quote! {
        impl sqlbindable::HasFields for #struct_name {
            fn not_none_fields( self) -> core::result::Result<sqlbindable::FieldVec, sqlbindable::TryIntoExprError> {
                use sqlbindable::TryIntoExpr as _;

                let mut ff: Vec<sqlbindable::Field> = Vec::new();
                #ff_not_option_pushes
                #ff_option_not_none_pushes
                Ok(ff.into())
            }

            fn all_fields( self) -> core::result::Result<sqlbindable::FieldVec, sqlbindable::TryIntoExprError> {
                use sqlbindable::TryIntoExpr as _;

                let mut ff: Vec<sqlbindable::Field> = Vec::new();
                #ff_all_pushes
                Ok(ff.into())
            }

            fn field_names() -> &'static [&'static str] {
                &[#(
                #props_all_names,
                )*]
            }
        }
    };

    output.into()
}
