use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::ToTokens as _;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::parse_quote;

#[proc_macro_derive(Cartesian, attributes(cartesian))]
pub fn derive_cartesian(item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as syn::DeriveInput);

    match &mut item.data {
        syn::Data::Union(_) => unimplemented!(),
        syn::Data::Struct(data) => {
            data.fields.iter_mut().for_each(|field| {
                let compose = match_field(field, "compose");
                let skip = match_field(field, "skip");

                if compose {
                    match &mut field.ty {
                        syn::Type::Path(path) => {
                            let segment = path.path.segments.last_mut().unwrap();
                            segment.ident = format_ident!("{}Cartesian", segment.ident);
                        }
                        // FIXME: use associated type on trait
                        _ => unimplemented!(),
                    }
                } else if !skip {
                    let ty = &field.ty;
                    field.ty = parse_quote!(Vec::<#ty>);
                }
            })
        }
        syn::Data::Enum(_) => todo!(),
    }

    let ident_cartesian = quote::format_ident!("{}Cartesian", item.ident);
    let ident_original = std::mem::replace(&mut item.ident, ident_cartesian);
    let ident_cartesian = &item.ident;

    let iter = match &mut item.data {
        syn::Data::Union(_) => unimplemented!(),
        syn::Data::Struct(data) => {
            let tuple = data.fields.iter().any(|field| field.ident.is_none());

            // Base case
            let inner = data
                .fields
                .iter()
                .map(|field| field.ident.as_ref())
                .enumerate()
                .map(|(index, field)| match field {
                    None => format_ident!("_{}", index),
                    Some(ident) => ident.clone(),
                });
            let inner = match tuple {
                true => quote!(#ident_original( #( #inner.clone() ),* )),
                false => quote!(#ident_original { #( #inner: #inner.clone() ),* }),
            };
            let inner = quote!(::core::iter::once(#inner));

            // Inductive case
            data.fields
                .iter_mut()
                .enumerate()
                .map(|(index, field)| {
                    let compose = match_field(field, "compose");
                    let skip = match_field(field, "skip");

                    let (ident, access) = match field.ident.as_ref() {
                        None => (
                            format_ident!("_{}", index),
                            Literal::usize_unsuffixed(index).into_token_stream(),
                        ),
                        Some(ident) => (ident.clone(), ident.clone().into_token_stream()),
                    };

                    (compose, skip, ident, access)
                })
                .rev()
                .fold(inner, |inner, (compose, skip, ident, access)| {
                    if compose {
                        quote! {
                            self.#access.cartesian().flat_map(move |#ident| {
                                #inner
                            })
                        }
                    } else if skip {
                        quote! {
                            {
                                let #ident = &self.#access;
                                #inner
                            }
                        }
                    } else {
                        quote! {
                            self.#access.iter().flat_map(move |#ident| {
                                #inner
                            })
                        }
                    }
                })
        }

        syn::Data::Enum(_) => todo!(),
    };

    quote! {
        #item

        impl #ident_cartesian {
            pub fn cartesian(&self) -> impl Iterator<Item = #ident_original> {
                #iter
            }
        }
    }
    .into_token_stream()
    .into()
}

fn match_field(field: &syn::Field, name: &str) -> bool {
    field.attrs.iter().any(|attr| match_attr(attr, name))
}

fn match_attr(attr: &syn::Attribute, name: &str) -> bool {
    if !matches!(attr.style, syn::AttrStyle::Outer) {
        return false;
    }

    attr.meta
        .require_list()
        .is_ok_and(|list| list.path.is_ident("cartesian") && list.tokens.to_string() == name)
}
