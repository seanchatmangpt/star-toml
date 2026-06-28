use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut field_validations = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        match &data_struct.fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    let has_validate =
                        field.attrs.iter().any(|attr| attr.path().is_ident("validate"));
                    if has_validate {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_name_str = field_name.to_string();
                        let ty = &field.ty;

                        let is_option = path_contains_ident(ty, "Option");
                        let is_vec = path_contains_ident(ty, "Vec");

                        let validation = if is_option {
                            quote! {
                                if let Some(ref inner) = self.#field_name {
                                    v.field(#field_name_str, |v| inner.validate(v));
                                }
                            }
                        } else if is_vec {
                            quote! {
                                v.field(#field_name_str, |v| {
                                    for (i, x) in self.#field_name.iter().enumerate() {
                                        v.index(i, |v| x.validate(v));
                                    }
                                });
                            }
                        } else {
                            quote! {
                                v.field(#field_name_str, |v| self.#field_name.validate(v));
                            }
                        };
                        field_validations.push(validation);
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                for (idx, field) in fields_unnamed.unnamed.iter().enumerate() {
                    let has_validate =
                        field.attrs.iter().any(|attr| attr.path().is_ident("validate"));
                    if has_validate {
                        let index = syn::Index::from(idx);
                        let field_name_str = idx.to_string();
                        let ty = &field.ty;

                        let is_option = path_contains_ident(ty, "Option");
                        let is_vec = path_contains_ident(ty, "Vec");

                        let validation = if is_option {
                            quote! {
                                if let Some(ref inner) = self.#index {
                                    v.field(#field_name_str, |v| inner.validate(v));
                                }
                            }
                        } else if is_vec {
                            quote! {
                                v.field(#field_name_str, |v| {
                                    for (i, x) in self.#index.iter().enumerate() {
                                        v.index(i, |v| x.validate(v));
                                    }
                                });
                            }
                        } else {
                            quote! {
                                v.field(#field_name_str, |v| self.#index.validate(v));
                            }
                        };
                        field_validations.push(validation);
                    }
                }
            }
            Fields::Unit => {}
        }
    }

    let expanded = quote! {
        impl #impl_generics star_toml::Validate for #name #ty_generics #where_clause {
            fn validate(&self, v: &mut star_toml::Validator) {
                #( #field_validations )*
            }
        }
    };

    TokenStream::from(expanded)
}

fn path_contains_ident(ty: &syn::Type, ident_str: &str) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path.path.segments.iter().any(|seg| seg.ident == ident_str)
    } else {
        false
    }
}
