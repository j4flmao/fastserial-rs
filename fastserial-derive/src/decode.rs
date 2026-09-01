//! Derive macro implementations for fastserial Decode trait.
//!
//! This module provides the procedural macro for automatically implementing
//! the `Decode` trait for custom structs. It generates optimized decoding
//! code that uses binary search for O(log n) field matching.
//!
//! # Features
//!
//! - **SIMD-accelerated string scanning**: Uses SIMD instructions to find
//!   quoted strings and escape characters faster than linear scanning.
//! - **Whitespace skipping**: Optimized whitespace skipping using SIMD.
//! - **Field matching**: Binary search O(log n) comparison for field name matching.
//! - **Serde-compatible Attributes**: Full support for `#[fastserial(rename = "...")]`,
//!   `#[fastserial(rename_all = "...")]`, `#[fastserial(default)]`, and `#[fastserial(skip)]`.
//! - **Zero-copy Deserialization**: Borrows directly from the underlying buffer
//!   into strings (`&str`) to minimize allocations.
//! - **Error handling**: Comprehensive error types for malformed JSON, including
//!   detailed `MissingField` diagnostics.
//!
//! # Example
//!
//! ```ignore
//! use fastserial::Decode;
//!
//! #[derive(Decode)]
//! #[fastserial(rename_all = "camelCase")]
//! struct User {
//!     id: u64,
//!     #[fastserial(rename = "username")]
//!     name: String,
//!     #[fastserial(default)]
//!     age: u32,
//! }
//!
//! let json = br#"{"id":1,"username":"admin"}"#;
//! let user = User::decode(&mut ReadBuffer::new(json), &arena).unwrap();
//! assert_eq!(user.age, 0); // Fell back to default!
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

/// Generates the `Decode` trait implementation for a struct.
///
/// This function is called by the `#[derive(Decode)]` macro. It:
/// 1. Collects all fields from the struct
/// 2. Processes attributes (`#[fastserial(skip)]`, `#[fastserial(rename = "...")]`)
/// 3. Generates optimized decode code with field matching
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

    let container_attrs = crate::attrs::parse_container_attrs(&input.attrs);

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return quote! {
                compile_error!("Decode derive only works on structs");
            };
        }
    };

    let mut field_inits = quote! {};
    let mut field_defaults = quote! {};
    let mut key_strings: Vec<String> = Vec::new();
    let mut active_fields: Vec<(usize, String, syn::Ident)> = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_attrs = crate::attrs::parse_field_attrs_raw(field);
        
        let skip = field_attrs.skip;
        let mut field_name_str = container_attrs.rename_all.apply_to_field(&field_name.to_string());
        if let Some(r) = field_attrs.rename {
            field_name_str = r;
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

        // Determine default value
        if let Some(path_str) = &field_attrs.default_path {
            let path: syn::Path = syn::parse_str(path_str).unwrap();
            field_defaults.extend(quote! {
                #field_name: #var_ident.unwrap_or_else(#path),
            });
        } else if field_attrs.default {
            field_defaults.extend(quote! {
                #field_name: #var_ident.unwrap_or_else(Default::default),
            });
        } else if let Some(path_str) = &container_attrs.default_path {
            let path: syn::Path = syn::parse_str(path_str).unwrap();
            field_defaults.extend(quote! {
                #field_name: #var_ident.unwrap_or_else(|| #path().#field_name),
            });
        } else if container_attrs.default {
            field_defaults.extend(quote! {
                #field_name: #var_ident.unwrap_or_else(|| {
                    let __default: #name = Default::default();
                    __default.#field_name
                }),
            });
        } else {
            field_defaults.extend(quote! {
                #field_name: #var_ident.ok_or(::fastserial::Error::MissingField)?,
            });
        }

        active_fields.push((field_name_str.len(), field_name_str.clone(), var_ident));
        key_strings.push(field_name_str);
    }

    let key_count = key_strings.len();

    // Build a length-bucketed dispatcher:
    //
    //     match key.len() {
    //         3 => match key { b"foo" => ..., b"bar" => ..., _ => skip }
    //         5 => match key { b"hello" => ..., _ => skip }
    //         _ => skip,
    //     }
    //
    // This is dramatically faster than the previous match-with-guards approach
    // because (a) the outer length switch is a single integer compare, and (b)
    // the inner arms become byte-array literal patterns that LLVM can lower to
    // an integer compare instead of a `memcmp` call.
    let mut buckets: std::collections::BTreeMap<usize, Vec<(String, syn::Ident)>> =
        std::collections::BTreeMap::new();
    for (len, name, var) in &active_fields {
        buckets
            .entry(*len)
            .or_default()
            .push((name.clone(), var.clone()));
    }

    let mut decode_body = quote! {};
    for (len, entries) in &buckets {
        let len_lit = proc_macro2::Literal::usize_unsuffixed(*len);

        let mut inner = quote! {};
        for (name, var) in entries {
            let lit = syn::LitByteStr::new(name.as_bytes(), proc_macro2::Span::call_site());
            inner.extend(quote! {
                #lit => {
                    #var = Some(::fastserial::Decode::decode(r, _arena)?);
                }
            });
        }
        decode_body.extend(quote! {
            #len_lit => match key_bytes {
                #inner
                _ => { ::fastserial::codec::json::skip_value(r)?; }
            },
        });
    }

    quote! {
        impl #impl_gens ::fastserial::Decode<'de> for #name #ty_gens #where_clause {
            #[inline]
            fn decode(r: &mut ::fastserial::io::ReadBuffer<'de>, _arena: &'de ::fastserial::arena::Arena) -> ::core::result::Result<Self, ::fastserial::Error> {
                #field_inits

                r.expect_byte(b'{')?;
                ::fastserial::codec::json::skip_whitespace(r);

                if r.peek() == b'}' {
                    r.advance(1);
                } else {
                    loop {
                        let key_bytes = ::fastserial::codec::json::read_key_fast(r)?;
                        ::fastserial::codec::json::skip_colon(r)?;

                        match key_bytes.len() {
                            #decode_body
                            _ => {
                                ::fastserial::codec::json::skip_value(r)?;
                            }
                        }

                        if !::fastserial::codec::json::skip_comma_or_close(r, b'}')? {
                            r.advance(1);
                            break;
                        }
                    }
                }

                Ok(Self {
                    #field_defaults
                })
            }
        }

        impl #impl_gens #name #ty_gens #where_clause {
            pub const SCHEMA_HASH: u64 = #key_count as u64;
        }
    }
}
