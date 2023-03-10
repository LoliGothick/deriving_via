use proc_macro2::TokenStream;
use quote::quote;
use syn::GenericParam;

use super::super::utils::extract_fields;

pub(crate) fn extract(input: &syn::DeriveInput, via: Option<syn::Type>) -> TokenStream {
    let struct_name = &input.ident;
    let _generics = {
        let lt = &input.generics.lt_token;
        let params = &input.generics.params;
        let gt = &input.generics.gt_token;

        quote! { #lt #params #gt }
    };
    let generic_params = {
        let lt = &input.generics.lt_token;
        let params = input.generics.params.iter().filter_map(|p| match p {
            GenericParam::Type(ty) => Some(&ty.ident),
            _ => None,
        });
        let gt = &input.generics.gt_token;

        quote! { #lt #(#params),* #gt }
    };
    let predicates = input
        .generics
        .where_clause
        .as_ref()
        .map(|wc| &wc.predicates);
    let generics_params = &input.generics.params;
    let (_, field_ty, constructor) = extract_fields(input);

    via.as_ref().map_or_else(
        || {
            quote! {
                impl<'de, #generics_params> serde::Deserialize<'de> for #struct_name #generic_params {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                        #predicates
                    {
                        Ok(#constructor(#field_ty::deserialize(deserializer)?.into()))
                    }
                }
            }
        },
        |via| {
            quote! {
                impl<'de, #generics_params> serde::Deserialize<'de> for #struct_name #generic_params {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                        #predicates
                    {
                        Ok(#via::deserialize(deserializer)?.into())
                    }
                }
            }
        },
    )
}
