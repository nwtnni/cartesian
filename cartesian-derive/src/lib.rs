use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::ToTokens as _;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::parse_quote;

#[proc_macro_derive(Cartesian)]
pub fn derive_cartesian(item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as syn::DeriveInput);

    match &mut item.data {
        syn::Data::Union(_) => unimplemented!(),
        syn::Data::Struct(data) => {
            data.fields
                .iter_mut()
                .map(|field| &mut field.ty)
                .for_each(|ty| {
                    *ty = parse_quote!(Vec::<#ty>);
                })
        }
        syn::Data::Enum(_) => todo!(),
    }

    let ident_cartesian = quote::format_ident!("{}Cartesian", item.ident);
    let ident_original = std::mem::replace(&mut item.ident, ident_cartesian);
    let ident_cartesian = &item.ident;

    let iter = match &item.data {
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
                .iter()
                .map(|field| field.ident.as_ref())
                .enumerate()
                .map(|(index, field)| match field {
                    None => (
                        format_ident!("_{}", index),
                        Literal::usize_unsuffixed(index).into_token_stream(),
                    ),
                    Some(ident) => (ident.clone(), ident.clone().into_token_stream()),
                })
                .rev()
                .fold(inner, |inner, (outer_ident, outer_access)| {
                    quote! {
                        self.#outer_access.iter().flat_map(move |#outer_ident| {
                            #inner
                        })
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
