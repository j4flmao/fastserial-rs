# FastSerial Derive

Procedural macros for the `fastserial` crate.

This crate provides the `Encode` and `Decode` derive macros which generate highly optimized serialization and deserialization code for your structs, supporting both JSON and FastSerial's binary format.

## Installation

This crate is a dependency of `fastserial` and is usually not used directly.

```toml
[dependencies]
fastserial = "0.1"
```

> **Note**: Requires Rust 1.94 or later.

## Usage

```rust
use fastserial::{Encode, Decode};

#[derive(Encode, Decode)]
struct MyData {
    id: u32,
    name: String,
}
```

## Attributes

### `#[fastserial(skip)]`

Skips a field during both encoding and decoding.

- **Encoding**: The field will not be included in the output.
- **Decoding**: The field will be initialized with its `Default::default()` value.

```rust
#[derive(Encode, Decode)]
struct User {
    username: String,
    #[fastserial(skip)]
    password_hash: String, // This won't be serialized/deserialized
}
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
