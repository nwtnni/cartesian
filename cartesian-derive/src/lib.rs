use proc_macro::TokenStream;
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
            // Base case
            let fields = data.fields.iter().enumerate().map(|(index, field)| {
                let (unescaped, escaped) = field_access(index, field);
                quote!(#unescaped: #escaped.clone())
            });

            let inner = quote! {
                ::core::iter::once(
                    #ident_original { #(#fields),* }
                )
            };

            // Inductive case
            let iter = data
                .fields
                .iter_mut()
                .enumerate()
                .map(|(index, field)| {
                    let compose = match_field(field, "compose");
                    let skip = match_field(field, "skip");
                    let (unescaped, escaped) = field_access(index, field);
                    (compose, skip, unescaped, escaped)
                })
                .rev()
                .fold(inner, |inner, (compose, skip, unescaped, escaped)| {
                    if compose {
                        quote! {
                            self.#unescaped.cartesian().flat_map(move |#escaped| {
                                #inner
                            })
                        }
                    } else if skip {
                        quote! {
                            {
                                let #escaped = &self.#unescaped;
                                #inner
                            }
                        }
                    } else {
                        quote! {
                            self.#unescaped.iter().flat_map(move |#escaped| {
                                #inner
                            })
                        }
                    }
                });

            data.fields
                .iter_mut()
                .map(|field| &mut field.attrs)
                .for_each(remove_attr);

            iter
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

fn field_access(index: usize, field: &syn::Field) -> (syn::Member, syn::Ident) {
    match field.ident.as_ref() {
        None => (
            syn::Member::Unnamed(syn::Index {
                index: index as u32,
                span: proc_macro2::Span::call_site(),
            }),
            format_ident!("_{}", index),
        ),
        Some(ident) => (syn::Member::Named(ident.clone()), ident.clone()),
    }
}

fn remove_attr(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| {
        if !matches!(attr.style, syn::AttrStyle::Outer) {
            return true;
        }

        !attr.path().is_ident("cartesian")
    })
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
