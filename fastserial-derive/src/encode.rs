//! Derive macro implementations for fastserial Encode trait.
//!
//! This module provides the procedural macro for automatically implementing
//! the `Encode` trait for custom structs and enums. It generates optimized
//! formatting code, writing string and byte buffers directly to avoid intermediate allocations.
//!
//! # Features
//!
//! - **Hardcoded JSON logic**: For maximum speed, field keys and delimiters (`{`, `"`, `:`)
//!   are written directly as byte arrays.
//! - **Serde-compatible Attributes**:
//!   - `#[fastserial(rename_all = "camelCase")]`: Supports `camelCase`, `snake_case`, `PascalCase`, etc.
//!   - `#[fastserial(rename = "...")]`: Custom name for a specific field.
//!   - `#[fastserial(skip)]`: Ignore field during serialization.
//!   - `#[fastserial(skip_serializing_if = "path")]`: Conditionally skip a field (e.g., `Option::is_none`).
//!     Dynamically handles comma placement.
//! - **Enum Tagging**: Supports Internal, External, Adjacent, and Untagged representations.
//!
//! # Example
//!
//! ```ignore
//! use fastserial::Encode;
//!
//! #[derive(Encode)]
//! #[fastserial(rename_all = "camelCase")]
//! struct Point {
//!     x: i32,
//!     y: i32,
//!     #[fastserial(skip_serializing_if = "Option::is_none")]
//!     name: Option<String>,
//! }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

fn compute_schema_hash(type_name: &str, fields: &[(String, String)]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in type_name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for (field_name, field_type) in fields {
        let mut field_hash: u64 = 0xcbf29ce484222325;
        for byte in field_name.bytes() {
            field_hash ^= byte as u64;
            field_hash = field_hash.wrapping_mul(0x100000001b3);
        }
        hash ^= field_hash;
        let mut type_hash: u64 = 0xcbf29ce484222325;
        for byte in field_type.bytes() {
            type_hash ^= byte as u64;
            type_hash = type_hash.wrapping_mul(0x100000001b3);
        }
        hash ^= type_hash;
    }
    hash
}

struct FieldInfo {
    ident: syn::Ident,
    ty: syn::Type,
    encoded_name: String,
    skip: bool,
    skip_serializing_if: Option<String>,
}

fn parse_field_attrs(field: &syn::Field, rename_all: &crate::case::RenameRule) -> FieldInfo {
    let field_name = field.ident.as_ref().unwrap().clone();
    let field_ty = field.ty.clone();
    let mut encoded_name = rename_all.apply_to_field(&field_name.to_string());
    let mut skip = false;
    let mut skip_serializing_if = None;

    for attr in &field.attrs {
        if attr.meta.path().is_ident("fastserial") {
            let _ = attr.parse_nested_meta(|meta: syn::meta::ParseNestedMeta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("rename") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    encoded_name = lit.value();
                } else if meta.path.is_ident("skip_serializing_if") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    skip_serializing_if = Some(lit.value());
                }
                // Ignore default, alias, deny_unknown_fields — no effect on encode
                Ok(())
            });
        }
    }

    FieldInfo {
        ident: field_name,
        ty: field_ty,
        encoded_name,
        skip,
        skip_serializing_if,
    }
}

enum EnumTagging {
    External,
    Internal { tag: String },
    Adjacent { tag: String, content: String },
    Untagged,
}

struct ContainerAttrs {
    tagging: EnumTagging,
    rename_all: crate::case::RenameRule,
}

fn parse_container_attrs(attrs: &[syn::Attribute]) -> ContainerAttrs {
    let mut tag: Option<String> = None;
    let mut content: Option<String> = None;
    let mut untagged = false;
    let mut rename_all = crate::case::RenameRule::None;

    for attr in attrs {
        if attr.meta.path().is_ident("fastserial") {
            let _ = attr.parse_nested_meta(|meta: syn::meta::ParseNestedMeta| {
                if meta.path.is_ident("tag") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    tag = Some(lit.value());
                } else if meta.path.is_ident("content") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    content = Some(lit.value());
                } else if meta.path.is_ident("untagged") {
                    untagged = true;
                } else if meta.path.is_ident("rename_all") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    if let Some(rule) = crate::case::RenameRule::from_str(&lit.value()) {
                        rename_all = rule;
                    }
                }
                Ok(())
            });
        }
    }

    let tagging = if untagged {
        EnumTagging::Untagged
    } else if let Some(t) = tag {
        if let Some(c) = content {
            EnumTagging::Adjacent { tag: t, content: c }
        } else {
            EnumTagging::Internal { tag: t }
        }
    } else {
        EnumTagging::External
    };

    ContainerAttrs {
        tagging,
        rename_all,
    }
}

fn get_variant_name(variant: &syn::Variant) -> String {
    let mut name = variant.ident.to_string();
    for attr in &variant.attrs {
        if attr.meta.path().is_ident("fastserial") {
            let _ = attr.parse_nested_meta(|meta: syn::meta::ParseNestedMeta| {
                if meta.path.is_ident("rename") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    name = lit.value();
                }
                Ok(())
            });
        }
    }
    name
}

fn encode_variant_fields_as_object(
    fields: &Fields,
    prefix: &TokenStream,
    rename_all: &crate::case::RenameRule,
) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let mut body = quote! {};
            let any_skip_if = named.named.iter().any(|f| {
                parse_field_attrs(f, rename_all)
                    .skip_serializing_if
                    .is_some()
            });

            if any_skip_if {
                body.extend(quote! { let mut __fastserial_first = true; });
            }

            let mut first = true;
            for f in &named.named {
                let info = parse_field_attrs(f, rename_all);
                if info.skip {
                    continue;
                }
                let fname = &info.ident;
                let field_name_str = &info.encoded_name;

                if let Some(cond_path) = &info.skip_serializing_if {
                    let cond: syn::Path = syn::parse_str(cond_path).unwrap();
                    let key_first = format!("\"{}\":", field_name_str);
                    let key_next = format!(",\"{}\":", field_name_str);
                    body.extend(quote! {
                        if !#cond(&#prefix #fname) {
                            if __fastserial_first {
                                w.write_bytes(#key_first.as_bytes())?;
                                __fastserial_first = false;
                            } else {
                                w.write_bytes(#key_next.as_bytes())?;
                            }
                            #prefix #fname.encode(w)?;
                        }
                    });
                } else {
                    if any_skip_if {
                        let key_first = format!("\"{}\":", field_name_str);
                        let key_next = format!(",\"{}\":", field_name_str);
                        body.extend(quote! {
                            if __fastserial_first {
                                w.write_bytes(#key_first.as_bytes())?;
                                __fastserial_first = false;
                            } else {
                                w.write_bytes(#key_next.as_bytes())?;
                            }
                            #prefix #fname.encode(w)?;
                        });
                    } else {
                        let key = if first {
                            format!("\"{}\":", field_name_str)
                        } else {
                            format!(",\"{}\":", field_name_str)
                        };
                        body.extend(quote! {
                            w.write_bytes(#key.as_bytes())?;
                            #prefix #fname.encode(w)?;
                        });
                    }
                }
                first = false;
            }
            quote! {
                w.write_byte(b'{')?;
                #body
                w.write_byte(b'}')?;
            }
        }
        Fields::Unnamed(unnamed) => {
            if unnamed.unnamed.len() == 1 {
                quote! { #prefix 0.encode(w)?; }
            } else {
                let mut body = quote! {};
                let mut first = true;
                for (i, _f) in unnamed.unnamed.iter().enumerate() {
                    let idx = syn::Index::from(i);
                    if first {
                        body.extend(quote! {
                            #prefix #idx.encode(w)?;
                        });
                    } else {
                        body.extend(quote! {
                            w.write_byte(b',')?;
                            #prefix #idx.encode(w)?;
                        });
                    }
                    first = false;
                }
                quote! {
                    w.write_byte(b'[')?;
                    #body
                    w.write_byte(b']')?;
                }
            }
        }
        Fields::Unit => {
            quote! {}
        }
    }
}

fn derive_encode_enum(input: &DeriveInput, data: &syn::DataEnum) -> TokenStream {
    let name = &input.ident;
    let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
    let container_attrs = parse_container_attrs(&input.attrs);
    let tagging = container_attrs.tagging;

    let mut match_arms = quote! {};

    for variant in &data.variants {
        let vident = &variant.ident;
        let vname = get_variant_name(variant);

        let (pattern, encode_body) = match &tagging {
            EnumTagging::External => match &variant.fields {
                Fields::Unit => {
                    let pat = quote! { #name::#vident };
                    let body = quote! {
                        w.write_byte(b'"')?;
                        w.write_bytes(#vname.as_bytes())?;
                        w.write_byte(b'"')?;
                    };
                    (pat, body)
                }
                Fields::Named(named) => {
                    let field_idents: Vec<_> = named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let pat = quote! { #name::#vident { #(ref #field_idents),* } };
                    let key = format!("\"{}\":", vname);
                    let inner = encode_variant_fields_as_object(
                        &variant.fields,
                        &quote! {},
                        &container_attrs.rename_all,
                    );
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#key.as_bytes())?;
                        #inner
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
                Fields::Unnamed(unnamed) => {
                    let bindings: Vec<_> = (0..unnamed.unnamed.len())
                        .map(|i| {
                            syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                        })
                        .collect();
                    let pat = quote! { #name::#vident(#(ref #bindings),*) };
                    let key = format!("\"{}\":", vname);
                    if unnamed.unnamed.len() == 1 {
                        let body = quote! {
                            w.write_byte(b'{')?;
                            w.write_bytes(#key.as_bytes())?;
                            f0.encode(w)?;
                            w.write_byte(b'}')?;
                        };
                        (pat, body)
                    } else {
                        let mut inner = quote! { w.write_byte(b'[')?; };
                        for (i, b) in bindings.iter().enumerate() {
                            if i > 0 {
                                inner.extend(quote! { w.write_byte(b',')?; });
                            }
                            inner.extend(quote! { #b.encode(w)?; });
                        }
                        inner.extend(quote! { w.write_byte(b']')?; });
                        let body = quote! {
                            w.write_byte(b'{')?;
                            w.write_bytes(#key.as_bytes())?;
                            #inner
                            w.write_byte(b'}')?;
                        };
                        (pat, body)
                    }
                }
            },
            EnumTagging::Internal { tag } => match &variant.fields {
                Fields::Unit => {
                    let pat = quote! { #name::#vident };
                    let tag_entry = format!("\"{}\":\"{}\"", tag, vname);
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#tag_entry.as_bytes())?;
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
                Fields::Named(named) => {
                    let field_idents: Vec<_> = named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let pat = quote! { #name::#vident { #(ref #field_idents),* } };
                    let tag_entry = format!("\"{}\":\"{}\"", tag, vname);
                    let mut field_body = quote! {};
                    for f in &named.named {
                        let info = parse_field_attrs(f, &container_attrs.rename_all);
                        if info.skip {
                            continue;
                        }
                        let fname = &info.ident;
                        let key = format!(",\"{}\":", info.encoded_name);
                        field_body.extend(quote! {
                            w.write_bytes(#key.as_bytes())?;
                            #fname.encode(w)?;
                        });
                    }
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#tag_entry.as_bytes())?;
                        #field_body
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
                Fields::Unnamed(unnamed) => {
                    let bindings: Vec<_> = (0..unnamed.unnamed.len())
                        .map(|i| {
                            syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                        })
                        .collect();
                    let pat = quote! { #name::#vident(#(ref #bindings),*) };
                    let tag_entry = format!("\"{}\":\"{}\"", tag, vname);
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#tag_entry.as_bytes())?;
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
            },
            EnumTagging::Adjacent { tag, content } => match &variant.fields {
                Fields::Unit => {
                    let pat = quote! { #name::#vident };
                    let tag_entry = format!("\"{}\":\"{}\"", tag, vname);
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#tag_entry.as_bytes())?;
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
                Fields::Named(named) => {
                    let field_idents: Vec<_> = named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let pat = quote! { #name::#vident { #(ref #field_idents),* } };
                    let tag_entry = format!("\"{}\":\"{}\",\"{}\":", tag, vname, content);
                    let inner = encode_variant_fields_as_object(
                        &variant.fields,
                        &quote! {},
                        &container_attrs.rename_all,
                    );
                    let body = quote! {
                        w.write_byte(b'{')?;
                        w.write_bytes(#tag_entry.as_bytes())?;
                        #inner
                        w.write_byte(b'}')?;
                    };
                    (pat, body)
                }
                Fields::Unnamed(unnamed) => {
                    let bindings: Vec<_> = (0..unnamed.unnamed.len())
                        .map(|i| {
                            syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                        })
                        .collect();
                    let pat = quote! { #name::#vident(#(ref #bindings),*) };
                    let tag_entry = format!("\"{}\":\"{}\",\"{}\":", tag, vname, content);
                    if unnamed.unnamed.len() == 1 {
                        let body = quote! {
                            w.write_byte(b'{')?;
                            w.write_bytes(#tag_entry.as_bytes())?;
                            f0.encode(w)?;
                            w.write_byte(b'}')?;
                        };
                        (pat, body)
                    } else {
                        let mut inner = quote! { w.write_byte(b'[')?; };
                        for (i, b) in bindings.iter().enumerate() {
                            if i > 0 {
                                inner.extend(quote! { w.write_byte(b',')?; });
                            }
                            inner.extend(quote! { #b.encode(w)?; });
                        }
                        inner.extend(quote! { w.write_byte(b']')?; });
                        let body = quote! {
                            w.write_byte(b'{')?;
                            w.write_bytes(#tag_entry.as_bytes())?;
                            #inner
                            w.write_byte(b'}')?;
                        };
                        (pat, body)
                    }
                }
            },
            EnumTagging::Untagged => match &variant.fields {
                Fields::Unit => {
                    let pat = quote! { #name::#vident };
                    let body = quote! {
                        w.write_bytes(b"null")?;
                    };
                    (pat, body)
                }
                Fields::Named(named) => {
                    let field_idents: Vec<_> = named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    let pat = quote! { #name::#vident { #(ref #field_idents),* } };
                    let inner = encode_variant_fields_as_object(
                        &variant.fields,
                        &quote! {},
                        &container_attrs.rename_all,
                    );
                    (pat, inner)
                }
                Fields::Unnamed(unnamed) => {
                    let bindings: Vec<_> = (0..unnamed.unnamed.len())
                        .map(|i| {
                            syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                        })
                        .collect();
                    let pat = quote! { #name::#vident(#(ref #bindings),*) };
                    if unnamed.unnamed.len() == 1 {
                        let body = quote! { f0.encode(w)?; };
                        (pat, body)
                    } else {
                        let mut inner = quote! { w.write_byte(b'[')?; };
                        for (i, b) in bindings.iter().enumerate() {
                            if i > 0 {
                                inner.extend(quote! { w.write_byte(b',')?; });
                            }
                            inner.extend(quote! { #b.encode(w)?; });
                        }
                        inner.extend(quote! { w.write_byte(b']')?; });
                        (pat, inner)
                    }
                }
            },
        };

        match_arms.extend(quote! {
            #pattern => { #encode_body }
        });
    }

    let type_name_str = name.to_string();
    let mut hash_fields: Vec<(String, String)> = Vec::new();
    for variant in &data.variants {
        let vname = get_variant_name(variant);
        let vtype = match &variant.fields {
            Fields::Unit => "unit".to_string(),
            Fields::Named(_) => "struct".to_string(),
            Fields::Unnamed(_) => "tuple".to_string(),
        };
        hash_fields.push((vname, vtype));
    }
    let schema_hash = compute_schema_hash(&type_name_str, &hash_fields);

    quote! {
        impl #impl_gens ::fastserial::Encode for #name #ty_gens #where_clause {
            const SCHEMA_HASH: u64 = #schema_hash;

            #[inline(always)]
            fn encode<W: ::fastserial::io::WriteBuffer>(&self, w: &mut W) -> ::core::result::Result<(), ::fastserial::Error> {
                match self {
                    #match_arms
                }
                Ok(())
            }
        }
    }
}

pub fn derive_encode(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Enum(data) => derive_encode_enum(&input, data),
        Data::Struct(s) => {
            let fields = &s.fields;
            let mut field_infos: Vec<FieldInfo> = Vec::new();
            let container_attrs = parse_container_attrs(&input.attrs);

            for field in fields.iter() {
                field_infos.push(parse_field_attrs(field, &container_attrs.rename_all));
            }

            let mut encode_body = quote! {};
            let mut format_body = quote! {};
            let mut first = true;
            let mut n_fields: usize = 0;

            // Collect data for schema hash
            let type_name_str = name.to_string();
            let mut hash_fields: Vec<(String, String)> = Vec::new();

            let any_skip_if = field_infos.iter().any(|f| f.skip_serializing_if.is_some());

            if any_skip_if {
                encode_body.extend(quote! { let mut __fastserial_first = true; });
                format_body.extend(quote! { let mut __fastserial_first = true; });
            }

            for info in &field_infos {
                if info.skip {
                    continue;
                }
                n_fields += 1;
                let field_name = &info.ident;
                let field_name_str = &info.encoded_name;
                let ty = &info.ty;

                hash_fields.push((info.encoded_name.clone(), quote!(#ty).to_string()));

                if let Some(cond_path) = &info.skip_serializing_if {
                    let cond: syn::Path = syn::parse_str(cond_path).unwrap();
                    if any_skip_if {
                        let key_first = format!("\"{}\":", field_name_str);
                        let key_next = format!(",\"{}\":", field_name_str);
                        encode_body.extend(quote! {
                            if !#cond(&self.#field_name) {
                                if __fastserial_first {
                                    w.write_bytes(#key_first.as_bytes())?;
                                    __fastserial_first = false;
                                } else {
                                    w.write_bytes(#key_next.as_bytes())?;
                                }
                                self.#field_name.encode(w)?;
                            }
                        });
                        format_body.extend(quote! {
                            if !#cond(&self.#field_name) {
                                if __fastserial_first {
                                    F::write_field_key(&#field_name_str.as_bytes(), w)?;
                                    __fastserial_first = false;
                                } else {
                                    F::field_separator(w)?;
                                    F::write_field_key(&#field_name_str.as_bytes(), w)?;
                                }
                                self.#field_name.encode_with_format::<F, W>(w)?;
                            }
                        });
                    }
                } else {
                    if any_skip_if {
                        let key_first = format!("\"{}\":", field_name_str);
                        let key_next = format!(",\"{}\":", field_name_str);
                        encode_body.extend(quote! {
                            if __fastserial_first {
                                w.write_bytes(#key_first.as_bytes())?;
                                __fastserial_first = false;
                            } else {
                                w.write_bytes(#key_next.as_bytes())?;
                            }
                            self.#field_name.encode(w)?;
                        });
                        format_body.extend(quote! {
                            if __fastserial_first {
                                F::write_field_key(&#field_name_str.as_bytes(), w)?;
                                __fastserial_first = false;
                            } else {
                                F::field_separator(w)?;
                                F::write_field_key(&#field_name_str.as_bytes(), w)?;
                            }
                            self.#field_name.encode_with_format::<F, W>(w)?;
                        });
                    } else {
                        // Original compile-time comma logic for max performance
                        let key = if first {
                            format!("\"{}\":", field_name_str)
                        } else {
                            format!(",\"{}\":", field_name_str)
                        };

                        encode_body.extend(quote! {
                            w.write_bytes(#key.as_bytes())?;
                            self.#field_name.encode(w)?;
                        });

                        if first {
                            format_body.extend(quote! {
                                F::write_field_key(&#field_name_str.as_bytes(), w)?;
                                self.#field_name.encode_with_format::<F, W>(w)?;
                            });
                        } else {
                            format_body.extend(quote! {
                                F::field_separator(w)?;
                                F::write_field_key(&#field_name_str.as_bytes(), w)?;
                                self.#field_name.encode_with_format::<F, W>(w)?;
                            });
                        }
                    }
                }

                first = false;
            }

            let schema_hash = compute_schema_hash(&type_name_str, &hash_fields);

            quote! {
                impl #impl_gens ::fastserial::Encode for #name #ty_gens #where_clause {
                    const SCHEMA_HASH: u64 = #schema_hash;

                    #[inline(always)]
                    fn encode<W: ::fastserial::io::WriteBuffer>(&self, w: &mut W) -> ::core::result::Result<(), ::fastserial::Error> {
                        w.write_byte(b'{')?;
                        #encode_body
                        w.write_byte(b'}')
                    }

                    fn encode_with_format<F: ::fastserial::Format, W: ::fastserial::io::WriteBuffer>(
                        &self, w: &mut W
                    ) -> ::core::result::Result<(), ::fastserial::Error> {
                        F::begin_object(#n_fields, w)?;
                        #format_body
                        F::end_object(w)
                    }
                }
            }
        }
        _ => {
            quote! {
                compile_error!("Encode derive only works on structs and enums");
            }
        }
    }
}
