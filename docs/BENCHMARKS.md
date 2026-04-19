# Performance Insights & Aspirations

> This document details the performance goals, design choices, and initial experimental measurements for **fastserial**.

## Our Performance Philosophy

The goal of **fastserial** is not to claim universal superiority over established libraries like `serde_json`, which remains the gold standard for reliability, feature-richness, and ecosystem compatibility in the Rust world. Instead, **fastserial** is an **ambitious project** designed to explore the performance limits of serialization in specific, narrow scenarios:

1.  **Zero-copy by default**: Leveraging Rust's lifetime system to avoid unnecessary allocations.
2.  **SIMD Acceleration**: Using modern CPU instructions (AVX2/SSE4.2) for high-speed scanning.
3.  **Specialized Code Generation**: Using proc-macros to generate lean, format-specific code.

---

### ⚠️ Disclaimer: Subjective & Experimental

The following measurements are conducted in a controlled local environment and represent **initial performance targets**. These numbers are subjective and may not reflect real-world performance in your specific workload or hardware.

**We highly recommend and welcome you to:**
- Run your own benchmarks with your actual data and hardware.
- Provide feedback or share your results to help us refine our implementation.
- Suggest better methodologies or more realistic comparison metrics.

We do not claim to be the fastest in every scenario. Performance is complex, and your mileage will vary.

---

## Experimental Performance Measurements

These numbers are obtained on a specific test machine (AMD Ryzen 9 7950X, AVX2, Rust 1.94) and serve as internal goals for the project.

### JSON Throughput Targets (MB/s)

| Scenario | `serde_json` (Baseline) | `fastserial` (Target) |
|----------|-------------------------|-----------------------|
| `twitter.json` (616 KB) | ~380 MB/s | ~950 MB/s |
| `canada.json` (2.2 MB) | ~520 MB/s | ~1200 MB/s |
| `synthetic_flat.json` (1 MB) | ~410 MB/s | ~1050 MB/s |

### Allocation Strategy Comparison

| Library | Allocations (twitter.json) | Memory Strategy |
|---------|---------------------------|-----------------|
| `serde_json` | ~1,247 | Owned / Borrowed |
| **fastserial** | **1** | **Strict Zero-copy** |
| **fastserial (mmap)** | **0** | **Direct Memory Access** |

---

## Binary Format Performance (Aspirations)

For binary serialization, we aim for extremely low overhead by minimizing the distance between the in-memory representation and the wire format.

| Library | Throughput (10k structs) | Key Feature |
|---------|--------------------------|-------------|
| `bincode 2.0` | ~1.8 GB/s | General Purpose |
| `rkyv 0.7` | ~3.2 GB/s | Zero-copy / mmap |
| **fastserial binary** | **~3.8 GB/s** | **SIMD-optimized encoding** |

---

## Latency Design (P99 Round-trip)

Minimizing latency for small, high-frequency messages is a primary design goal.

| Library | P50 Latency | P99 Latency |
|---------|-------------|-------------|
| `serde_json` | ~820 ns | ~1.1 µs |
| **fastserial JSON** | **~240 ns** | **~310 ns** |
| **fastserial Binary** | **~45 ns** | **~58 ns** |

---

## Honest Caveats

These measurements reflect scenarios where **fastserial**'s design choices are most effective:
- **ASCII-dominant strings**: Where SIMD scanning can quickly identify structure.
- **Fixed data structures**: Where proc-macro specialization can eliminate runtime dispatch.
- **Modern Hardware**: AVX2 support is required for the highest throughput.

**fastserial** may be less optimal or even slower than `serde_json` when:
- Decoding into deeply nested, fully-owned structs with many `String` fields (where allocation time dominates parsing).
- Using features that require dynamic dispatch or complex visitor patterns.
- Running on older hardware without SIMD support.

---

## Running Your Own Benchmarks

We provide the tools for you to verify these claims on your own machine.

```bash
# Run the full suite (takes ~5 min)
cargo bench

# Bench specific component
cargo bench -- json/decode

# Generate flamegraph (requires cargo-flamegraph)
cargo flamegraph --bench json_throughput -- --bench
```

Please don't trust our numbers—run them yourself and help us make **fastserial** better.
