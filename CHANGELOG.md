# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Enum derive support for `Encode` and `Decode`.
- `HashMap` and `BTreeMap` serialization/deserialization support.
- Tuple serialization/deserialization support.
- `json::Value` dynamic type for untyped JSON data.
- `json::encode_pretty` for pretty-printed JSON output.
- `decode_str` convenience function.
- `io::Write`-based `WriteBuffer` support.
- New field/struct attributes: `default`, `alias`, `deny_unknown_fields`.
- `SCHEMA_HASH` compile-time computation for schema fingerprinting.
- `encode_with_format` generation for derived structs.
- `msgpack` module export.

### Fixed
- Documentation: replaced incorrect `#[serial(...)]` attribute references with `#[fastserial(...)]`.

## [0.1.0] - 2026-04-17

### Added
- Initial release of `fastserial` core library.
- High-performance SIMD-accelerated JSON serialization/deserialization.
- Support for `binary` format (fastserial specific).
- `fastserial-derive` proc-macro for `Encode` and `Decode` traits.
- `sample-axum` project demonstrating integration with Axum and MySQL.
- Automated database schema initialization and seeding.
- Standardized `ApiResponse` and `AppError` handling in sample app.

### Changed
- Improved floating point parsing logic.
- Enhanced SIMD whitespace skipping performance.

### Fixed
- Fixed `i64::MIN` overflow in JSON parsing.
- Corrected unescaping logic for Unicode characters in strings.
- Fixed SIMD no-op in certain whitespace scenarios.
