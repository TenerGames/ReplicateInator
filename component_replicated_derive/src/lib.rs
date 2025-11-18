use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, parse_quote, Fields, Field};

#[proc_macro_attribute]
pub fn component_replicated(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    fn process_fields<'a, I>(iter: I)
    where
        I: Iterator<Item = &'a mut Field>
    {
        for field in iter {
            let attr_index = field.attrs.iter().position(|attr| {
                attr.path().is_ident("dont_replicate")
            });

            if let Some(idx) = attr_index {
                field.attrs.remove(idx);
                
                field.attrs.push(parse_quote!(#[serde(skip)]));
            }
        }
    }

    match &mut item_struct.fields {
        Fields::Named(fields) => {
            process_fields(fields.named.iter_mut());
        },
        Fields::Unnamed(fields) => {
            process_fields(fields.unnamed.iter_mut());
        },
        Fields::Unit => {

        }
    }


    let struct_name = &item_struct.ident;
    let (impl_generics, type_generics, where_clause) = item_struct.generics.split_for_impl();

    item_struct.attrs.push(parse_quote!(
        #[derive(
            bevy::prelude::Component,
            bevy::prelude::Reflect,
            Clone,
            Default,
            serde::Serialize,
            serde::Deserialize,
        )]
    ));

    item_struct.attrs.push(parse_quote!(
        #[reflect(Component)]
    ));

    let expanded = quote! {
        #item_struct

        impl #impl_generics ComponentReplicated for #struct_name #type_generics #where_clause {}
    };

    TokenStream::from(expanded)
}