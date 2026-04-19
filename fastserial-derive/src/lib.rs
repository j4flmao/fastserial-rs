//! # FastSerial Derive
//!
//! Procedural macros for the `fastserial` crate.
//!
//! This crate provides the `Encode` and `Decode` derive macros which generate
//! highly optimized serialization and deserialization code for your structs.
//!
//! ## Attributes
//!
//! ### `#[fastserial(skip)]`
//!
//! Skips a field during both encoding and decoding.
//!
//! - **Encoding**: The field will not be included in the output.
//! - **Decoding**: The field will be initialized with its `Default::default()` value.
//!
//! ```rust,ignore
//! use fastserial::{Encode, Decode};
//!
//! #[derive(Encode, Decode)]
//! struct User {
//!     username: String,
//!     #[fastserial(skip)]
//!     password_hash: String, // This won't be serialized
//! }
//! ```

extern crate proc_macro;

mod decode;
mod encode;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Encode, attributes(fastserial))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    encode::derive_encode(input).into()
}

#[proc_macro_derive(Decode, attributes(fastserial))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    decode::derive_decode(input).into()
}
