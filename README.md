# FastSerial

[![Crates.io](https://img.shields.io/crates/v/fastserial.svg)](https://crates.io/crates/fastserial)
[![Documentation](https://docs.rs/fastserial/badge.svg)](https://docs.rs/fastserial)
[![License](https://img.shields.io/crates/l/fastserial.svg)](https://github.com/j4flmao/fastserial-rs/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.94%2B-blue.svg)](https://github.com/j4flmao/fastserial-rs)
[![Build Status](https://github.com/j4flmao/fastserial-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/j4flmao/fastserial-rs/actions)
[![Stars](https://img.shields.io/github/stars/j4flmao/fastserial-rs.svg)](https://github.com/j4flmao/fastserial-rs/stargazers)

`fastserial` is a high-performance, zero-copy serialization and deserialization framework for Rust. It is an **ambitious project** designed as an alternative for high-throughput use cases like REST APIs, game engines, and real-time data processing, focusing on minimizing overhead and maximizing efficiency.

## 🚀 Key Features

- **Ambitious Performance**: Designed for high throughput by using specialized code generation and SIMD-accelerated scanning.
- **Zero-Copy Deserialization**: Borrow directly from input buffers (e.g., `&str`, `&[u8]`) to avoid heap allocations.
- **SIMD-First Design**: Native AVX2 and SSE4.2 support for parsing and escaping.
- **Minimal Dependencies**: Fast compilation and small binary footprint.
- **No-Std Support**: Optimized for embedded environments and WASM.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fastserial = "0.1"
```

Or for the latest version:

```toml
[dependencies]
fastserial = "0.1"
```

> **Note**: This library requires Rust 1.94 or later for full SIMD support (AVX2, SSE4.2).

## 🛠️ Usage

```rust
use fastserial::{Encode, Decode, json};

#[derive(Encode, Decode, Debug, PartialEq)]
struct User<'a> {
    id: u64,
    username: &'a str,
    email: String,
    #[fastserial(skip)]
    password_hash: String,
}

fn main() -> Result<(), fastserial::Error> {
    let user = User {
        id: 1,
        username: "j4flmao",
        email: "dev@fastserial.rs".into(),
        password_hash: "secret_hash".into(),
    };

    // Serialize to JSON
    let json_data = json::encode(&user)?;
    println!("{}", String::from_utf8_lossy(&json_data));

    // Deserialize back (zero-copy for username)
    let decoded: User = json::decode(&json_data)?;
    assert_eq!(user.username, decoded.username);

    Ok(())
}
```

## 🛠️ Development

This project uses a `Makefile` to simplify common development tasks.

```bash
# Run all quality checks (fmt, lint, test, build)
make all

# Run specific tasks
make test        # Run all tests
make lint        # Run clippy
make fmt         # Format code
make build       # Build workspace
make doc         # Generate documentation
make run-sample  # Run the sample-axum application
```

## 📊 Performance Goals

**fastserial** aims for high performance by focusing on specific optimizations like SIMD and zero-copy. For detailed performance aspirations and initial experimental numbers, see [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| Scenarios | Design Choice | Target |
|-----------|---------------|--------|
| **JSON** | SIMD + Zero-copy | High throughput |
| **Binary** | Direct Mapping | Ultra-low latency |
| **Memory** | Borrowing | Minimal allocations |

## ⚙️ Configuration

- `std` (default): Enables `std` support.
- `json` (default): Enables JSON codec.
- `binary` (default): Enables the FastSerial binary format.
- `msgpack`: Enables MessagePack codec.
- `chrono`: Enables support for `chrono` types.
- `HashMap` / `BTreeMap` serialization support.
- `Tuple` serialization support.
- `json::Value` dynamic type for untyped JSON.
- `json::encode_pretty` for human-readable output.

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development workflow and how to get started.

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
