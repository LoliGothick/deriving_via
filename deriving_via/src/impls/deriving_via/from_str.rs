use proc_macro2::TokenStream;
use quote::quote;
use syn::GenericParam;

use crate::utils::extract_single_field;

pub(crate) fn extract(input: &syn::DeriveInput, via: Option<&syn::Type>) -> TokenStream {
    let struct_name = &input.ident;
    let generics = {
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
    let where_clause = &input.generics.where_clause;
    let field = extract_single_field(input);

    let field_name = &field.ident;
    let field_ty = &field.ty;

    match via.unwrap_or(field_ty) {
        syn::Type::Path(path) if path.path.is_ident("String") => field_name
            .as_ref()
            .map(|field_name| {
                quote! {
                    impl #generics std::str::FromStr for #struct_name #generic_params #where_clause {
                        type Err = std::convert::Infallible;

                        fn from_str(__: &str) -> std::result::Result<Self, Self::Err> {
                            Ok(Self { #field_name: __.to_owned() })
                        }
                    }
                }
            })
            .unwrap_or_else(|| {
                quote! {
                    impl #generics std::str::FromStr for #struct_name #generic_params #where_clause {
                        type Err = std::convert::Infallible;

                        fn from_str(__: &str) -> std::result::Result<Self, Self::Err> {
                            Ok(Self(__.to_owned()))
                        }
                    }
                }
            }),
        ty => {
            quote! {
                impl #generics std::str::FromStr for #struct_name #generic_params #where_clause {
                    type Err = <#ty as std::str::FromStr>::Err;

                    fn from_str(__: &str) -> std::result::Result<Self, Self::Err> {
                        let intermediate: #ty = __.parse()?;
                        Ok(intermediate.into())
                    }
                }
            }
        }
    }
}