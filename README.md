# FastSerial

[![Crates.io](https://img.shields.io/crates/v/fastserial.svg)](https://crates.io/crates/fastserial)
[![Documentation](https://docs.rs/fastserial/badge.svg)](https://docs.rs/fastserial)
[![License](https://img.shields.io/crates/l/fastserial.svg)](https://github.com/j4flmao/fastserial-rs/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.94%2B-blue.svg)](https://github.com/j4flmao/fastserial-rs)
[![Build Status](https://github.com/j4flmao/fastserial-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/j4flmao/fastserial-rs/actions)
[![Stars](https://img.shields.io/github/stars/j4flmao/fastserial-rs.svg)](https://github.com/j4flmao/fastserial-rs/stargazers)

`fastserial` is a high-performance, zero-copy serialization and deserialization framework for Rust. It is an **ambitious project** designed as an alternative for high-throughput use cases like REST APIs, game engines, and real-time data processing, focusing on minimizing overhead and maximizing efficiency.

## 🚀 V2.0 New Features!

- **Serde-Compatible Macro Attributes**: Support for `#[fastserial(rename_all = "camelCase")]`, `#[fastserial(default)]`, and `#[fastserial(skip_serializing_if = "...")]`.
- **Zero-Allocation DOM (TapeNode)**: Parse JSON into an unstructured DOM (`TapeNode`) directly into an Arena without any `String` or `Vec` allocations, achieving 2.4x speedups over `serde_json::Value`.
- **Streaming & Async NDJSON**: Effortlessly process multi-gigabyte log files using `NdjsonStream` (Sync) or `AsyncNdjsonStream` (Async via `tokio`).
- **CBOR Support**: Full zero-copy encoding and decoding for the CBOR binary standard (RFC 8949).

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
fastserial = "0.2" # V2.0 features!
```

Or for the latest version:

```toml
[dependencies]
fastserial = { git = "https://github.com/j4flmao/fastserial-rs.git" }
```

> **Note**: This library requires Rust 1.94 or later for full SIMD support (AVX2, SSE4.2).

## 🛠️ Usage

### Struct Serialization

```rust
use fastserial::{Encode, Decode, json};

#[derive(Encode, Decode, Debug, PartialEq)]
#[fastserial(rename_all = "camelCase")]
struct User<'a> {
    id: u64,
    username: &'a str,
    #[fastserial(default)]
    email: String,
    #[fastserial(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
}
// ...
```

### Async Streaming (NDJSON)

```rust
use fastserial::stream::AsyncNdjsonStream;
use tokio::fs::File;
use tokio::io::BufReader;

#[tokio::main]
async fn main() {
    let file = File::open("large_logs.jsonl").await.unwrap();
    let reader = BufReader::new(file);
    let mut stream = AsyncNdjsonStream::<LogEntry, _>::new(reader);

    while let Some(entry) = stream.next().await {
        println!("Log: {:?}", entry);
    }
}
```

### Zero-Allocation DOM (Tape)

```rust
use fastserial::tape::TapeNode;
use fastserial::arena::Arena;
use fastserial::io::ReadBuffer;
use fastserial::Decode;

let arena = Arena::new();
let mut buf = ReadBuffer::new(br#"{"fast": true, "speed": 9999}"#);
let tape = TapeNode::decode(&mut buf, &arena).unwrap();

if let TapeNode::Object(map) = tape {
    assert_eq!(map.get("fast"), Some(&TapeNode::Bool(true)));
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
| **Memory** | Borrowing / Arena | Minimal allocations |

## ⚙️ Configuration

- `std` (default): Enables `std` support.
- `json` (default): Enables JSON codec.
- `binary` (default): Enables the FastSerial binary format.
- `cbor` (default): Enables the CBOR format support.
- `msgpack`: Enables MessagePack codec.
- `tokio`: Enables Async NDJSON streaming support.
- `chrono`: Enables support for `chrono` types.
- `HashMap` / `BTreeMap` serialization support.
- `Tuple` serialization support.
- `json::Value` dynamic type for untyped JSON.
- `tape::TapeNode` zero-allocation dynamic DOM.
- `json::encode_pretty` for human-readable output.

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development workflow and how to get started.

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
