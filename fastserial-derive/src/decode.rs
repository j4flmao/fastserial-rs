use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_decode(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (_, ty_gens, where_clause) = input.generics.split_for_impl();

    let mut impl_generics = input.generics.clone();
    let mut has_de = false;
    let mut lifetimes = Vec::new();

    for param in &impl_generics.params {
        if let syn::GenericParam::Lifetime(lt) = param {
            if lt.lifetime.ident == "de" {
                has_de = true;
            } else {
                lifetimes.push(lt.lifetime.clone());
            }
        }
    }

    if !has_de {
        let mut de_param: syn::LifetimeParam = syn::parse_quote!('de);

        for lt in lifetimes {
            de_param.bounds.push(lt);
        }

        impl_generics
            .params
            .insert(0, syn::GenericParam::Lifetime(de_param));
    }
    let (impl_gens, _, _) = impl_generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return quote! {
                compile_error!("Decode derive only works on structs");
            };
        }
    };

    let mut field_inits = quote! {};
    let mut decode_body = quote! {};
    let mut field_defaults = quote! {};

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let mut field_name_str = field_name.to_string();
        let mut skip = false;

        for attr in &field.attrs {
            if attr.path().is_ident("fastserial") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        skip = true;
                    } else if meta.path.is_ident("rename") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        field_name_str = lit.value();
                    }
                    Ok(())
                });
            }
        }

        if skip {
            field_defaults.extend(quote! {
                #field_name: Default::default(),
            });
            continue;
        }

        let var_name = format!("f_{}", field_name);
        let var_ident = syn::Ident::new(&var_name, proc_macro2::Span::call_site());

        field_inits.extend(quote! {
            let mut #var_ident: Option<#field_ty> = None;
        });

        field_defaults.extend(quote! {
            #field_name: #var_ident.ok_or(::fastserial::Error::MissingField { name: #field_name_str })?,
        });

        decode_body.extend(quote! {
            bs if bs == #field_name_str.as_bytes() => {
                #var_ident = Some(::fastserial::Decode::decode(r)?);
            }
        });
    }

    quote! {
        impl #impl_gens ::fastserial::Decode<'de> for #name #ty_gens #where_clause {
            #[inline(always)]
            fn decode(r: &mut ::fastserial::io::ReadBuffer<'de>) -> ::core::result::Result<Self, ::fastserial::Error> {
                #field_inits

                ::fastserial::codec::json::skip_whitespace(r);
                r.expect_byte(b'{')?;

                let mut first = true;
                loop {
                    ::fastserial::codec::json::skip_whitespace(r);

                    if r.peek() == b'}' {
                        r.advance(1);
                        break;
                    }

                    if !first {
                        r.expect_byte(b',')?;
                        ::fastserial::codec::json::skip_whitespace(r);
                    }
                    first = false;

                    let key_bytes = ::fastserial::codec::json::read_key_fast(r)?;
                    ::fastserial::codec::json::skip_whitespace(r);
                    r.expect_byte(b':')?;
                    ::fastserial::codec::json::skip_whitespace(r);

                    match key_bytes {
                        #decode_body
                        _ => {
                            ::fastserial::codec::json::skip_value(r)?;
                        }
                    }
                }

                Ok(Self {
                    #field_defaults
                })
            }
        }

        impl #impl_gens #name #ty_gens #where_clause {
            pub const SCHEMA_HASH: u64 = 0;
        }
    }
}
