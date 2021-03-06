use quote;
use syn;

use helpers::{unique_attr, extract_attrs, is_disabled};

pub fn enum_message_inner(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let variants = match ast.body {
        syn::Body::Enum(ref v) => v,
        _ => panic!("EnumMessage only works on Enums"),
    };

    let mut arms = Vec::new();
    let mut detailed_arms = Vec::new();
    let mut serializations = Vec::new();

    for variant in variants {
        let messages = unique_attr(&variant.attrs, "strum", "message");
        let detailed_messages = unique_attr(&variant.attrs, "strum", "detailed_message");
        let ident = &variant.ident;

        use syn::VariantData::*;
        let params = match variant.data {
            Unit => quote::Ident::from(""),
            Tuple(..) => quote::Ident::from("(..)"),
            Struct(..) => quote::Ident::from("{..}"),
        };

        // You can't disable getting the serializations.
        {
            let mut serialization_variants = extract_attrs(&variant.attrs, "strum", "serialize");
            if serialization_variants.len() == 0 {
                serialization_variants.push(ident.as_ref());
            }

            let count = serialization_variants.len();
            serializations.push(quote!{
                &#name::#ident #params => {
                    static ARR: [&'static str; #count] = [#(#serialization_variants),*];
                    &ARR
                }
            });
        }

        // But you can disable the messages.
        if is_disabled(&variant.attrs) {
            continue;
        }

        if let Some(msg) = messages {
            let params = params.clone();

            // Push the simple message.
            let tokens = quote!{ &#name::#ident #params => ::std::option::Option::Some(#msg) };
            arms.push(tokens.clone());

            if detailed_messages.is_none() {
                detailed_arms.push(tokens);
            }
        }

        if let Some(msg) = detailed_messages {
            let params = params.clone();
            // Push the simple message.
            detailed_arms
                .push(quote!{ &#name::#ident #params => ::std::option::Option::Some(#msg) });
        }
    }

    if arms.len() < variants.len() {
        arms.push(quote!{ _ => ::std::option::Option::None });
    }

    if detailed_arms.len() < variants.len() {
        detailed_arms.push(quote!{ _ => ::std::option::Option::None });
    }

    quote!{
        impl #impl_generics ::strum::EnumMessage for #name #ty_generics #where_clause {
            fn get_message(&self) -> ::std::option::Option<&str> {
                match self {
                    #(#arms),*
                }
            }

            fn get_detailed_message(&self) -> ::std::option::Option<&str> {
                match self {
                    #(#detailed_arms),*
                }
            }

            fn get_serializations(&self) -> &[&str] {
                match self {
                    #(#serializations),*
                }
            }
        }
    }
}
