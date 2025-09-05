use proc_macro::TokenStream;
use proc_macro_error::abort;
use proc_macro_error::proc_macro_error;
use quote::ToTokens as _;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::parse_quote;

static NAMESPACE: &str = "cartesian";

#[proc_macro_error]
#[proc_macro_derive(Cartesian, attributes(cartesian))]
pub fn derive_cartesian(item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as syn::DeriveInput);

    let ident_cartesian = quote::format_ident!("{}Cartesian", item.ident);
    let ident_original = std::mem::replace(&mut item.ident, ident_cartesian);
    let ident_cartesian = &item.ident;

    let iter = match &mut item.data {
        syn::Data::Union(_) => unimplemented!(),
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Struct(data) => {
            let info = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| FieldInfo::new(index, field))
                .collect::<Vec<_>>();

            data.fields.iter_mut().zip(&info).for_each(|(field, info)| {
                let r#type = &field.ty;
                field.ty = match &info.r#type {
                    None => parse_quote!(Vec<#r#type>),
                    Some(FieldType::Flatten) => parse_quote!(::cartesian::IntoIter<#r#type>),
                    Some(FieldType::Single) => return,
                }
            });

            // Consistently use escaped identifiers as variable
            let moves = info.iter().map(
                |FieldInfo {
                     unescaped, escaped, ..
                 }| {
                    quote! {
                        let #escaped = self.#unescaped;
                    }
                },
            );
            let moves = quote!(#(#moves)*);

            let clones = info
                .iter()
                .map(|FieldInfo { escaped, .. }| {
                    quote! {
                        let #escaped = #escaped.clone();
                    }
                })
                .collect::<Vec<_>>();

            // Base case
            let fields = info.iter().map(
                |FieldInfo {
                     unescaped, escaped, ..
                 }| match unescaped {
                    syn::Member::Unnamed(_) => quote!(#unescaped: #escaped),
                    syn::Member::Named(_) => quote!(#escaped),
                },
            );
            let clone = clones
                .iter()
                .zip(&info)
                .rev()
                // Skip past enclosing singles
                .skip_while(|(_, info)| matches!(info.r#type, Some(FieldType::Single)))
                .skip(1)
                .map(|(clone, _)| clone);
            let base = quote! {
                #(#clone)*
                ::core::iter::once(
                    #ident_original { #(#fields),* }
                )
            };

            let inside = info
                .iter()
                .position(|info| matches!(info.r#type, None | Some(FieldType::Flatten)))
                .unwrap_or(info.len());

            // Inductive case
            let clones = info.iter().enumerate().map(|(index, info)| {
                if index <= inside {
                    return quote!();
                }

                match info.r#type {
                    Some(FieldType::Single) => clones[index].clone(),
                    None | Some(FieldType::Flatten) => {
                        let before = clones[0..index.saturating_sub(1)].iter();
                        let after = clones[index..].iter();
                        quote! {
                            #(#before)*
                            #(#after)*
                        }
                    }
                }
            });
            let inductive = info.iter().zip(clones).rev().fold(
                base,
                |inner,
                 (
                    FieldInfo {
                        escaped, r#type, ..
                    },
                    clones,
                )| {
                    let inner = match r#type {
                        None => quote! {
                            #escaped.into_iter().flat_map(move |#escaped| {
                                #inner
                            })
                        },
                        Some(FieldType::Flatten) => quote! {
                            #escaped.into_iter_cartesian().flat_map(move |#escaped| {
                                #inner
                            })
                        },
                        Some(FieldType::Single) => inner,
                    };

                    quote! {
                        #clones
                        #inner
                    }
                },
            );

            data.fields
                .iter_mut()
                .map(|field| &mut field.attrs)
                .for_each(remove_attr);

            quote! {
                #moves
                #inductive
            }
        }
    };

    quote! {
        impl ::cartesian::Cartesian for #ident_original {
            type IntoIter = #ident_cartesian;
            type Item = #ident_original;
        }

        #item

        impl ::cartesian::IntoIterCartesian for #ident_cartesian {
            type Item = #ident_original;
            fn into_iter_cartesian(self) -> impl Iterator<Item = Self::Item> {
                #iter
            }
        }
    }
    .into_token_stream()
    .into()
}

fn remove_attr(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| {
        if !matches!(attr.style, syn::AttrStyle::Outer) {
            return true;
        }

        !attr.path().is_ident(NAMESPACE)
    })
}

struct FieldInfo {
    unescaped: syn::Member,
    escaped: syn::Ident,
    r#type: Option<FieldType>,
}

impl FieldInfo {
    fn new(index: usize, field: &syn::Field) -> Self {
        let (unescaped, escaped) = match field.ident.as_ref() {
            None => (
                syn::Member::Unnamed(syn::Index {
                    index: index as u32,
                    span: proc_macro2::Span::call_site(),
                }),
                format_ident!("_{}", index),
            ),
            Some(ident) => (syn::Member::Named(ident.clone()), ident.clone()),
        };

        Self {
            unescaped,
            escaped,
            r#type: FieldType::new(field),
        }
    }
}

enum FieldType {
    Flatten,
    Single,
}

impl FieldType {
    fn new(field: &syn::Field) -> Option<Self> {
        let flatten = match_field(field, "flatten");
        let single = match_field(field, "single");

        if flatten as usize + single as usize > 1 {
            abort!(field, "Attributes [flatten, single] are mutually exclusive")
        } else if flatten {
            Some(Self::Flatten)
        } else if single {
            Some(Self::Single)
        } else {
            None
        }
    }
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
        .is_ok_and(|list| list.path.is_ident(NAMESPACE) && list.tokens.to_string() == name)
}
