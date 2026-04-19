use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fastserial::{Decode, Encode, binary};
use std::hint::black_box;

#[derive(Encode, Decode, Debug, Clone)]
struct BinaryStruct<'a> {
    id: u64,
    name: &'a str,
    email: &'a str,
    active: bool,
    balance: f64,
    tags: Vec<&'a str>,
}

fn create_test_data() -> BinaryStruct<'static> {
    BinaryStruct {
        id: 12345,
        name: "John Doe",
        email: "john.doe@example.com",
        active: true,
        balance: 1234.56,
        tags: vec!["premium", "verified", "active"],
    }
}

fn bench_encode_binary(c: &mut Criterion) {
    let data = create_test_data();
    let mut group = c.benchmark_group("binary_encode");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let items: Vec<_> = (0..size).map(|_| data.clone()).collect();
            b.iter(|| {
                for item in &items {
                    let _ = binary::encode(black_box(item));
                }
            });
        });
    }

    group.finish();
}

fn bench_decode_binary(c: &mut Criterion) {
    let data = create_test_data();
    let encoded = binary::encode(&data).unwrap();

    let mut group = c.benchmark_group("binary_decode");

    for size in [1, 10, 100, 1000].iter() {
        let items: Vec<_> = (0..*size).map(|_| encoded.clone()).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let slices: Vec<_> = items.iter().take(size).collect();
            b.iter(|| {
                for slice in &slices {
                    let _: Result<BinaryStruct<'_>, _> =
                        binary::decode(black_box(slice.as_slice()));
                }
            });
        });
    }

    group.finish();
}

fn bench_zero_copy_binary(c: &mut Criterion) {
    let data = create_test_data();
    let encoded = binary::encode(&data).unwrap();

    c.benchmark_group("binary_zero_copy")
        .bench_function("decode", |b| {
            b.iter(|| {
                let _: Result<BinaryStruct<'_>, _> = binary::decode(black_box(&encoded));
            });
        });
}

fn bench_binary_sizes(c: &mut Criterion) {
    #[derive(Encode, Decode, Debug, Clone)]
    struct SmallStruct {
        id: u64,
        active: bool,
    }

    #[derive(Encode, Decode, Debug, Clone)]
    struct MediumStruct {
        id: u64,
        name: String,
        active: bool,
        balance: f64,
    }

    #[derive(Encode, Decode, Debug, Clone)]
    struct LargeStruct<'a> {
        id: u64,
        name: &'a str,
        email: &'a str,
        bio: &'a str,
        active: bool,
        balance: f64,
        tags: Vec<&'a str>,
    }

    let small = SmallStruct {
        id: 1,
        active: true,
    };
    let medium = MediumStruct {
        id: 1,
        name: "John".to_string(),
        active: true,
        balance: 1.0,
    };
    let large = LargeStruct {
        id: 1,
        name: "John Doe",
        email: "john@example.com",
        bio: "Bio text",
        active: true,
        balance: 1.0,
        tags: vec!["a", "b", "c"],
    };

    let mut group = c.benchmark_group("binary_sizes");

    group.bench_function("small_struct", |b| {
        b.iter(|| {
            let enc = binary::encode(black_box(&small)).unwrap();
            let _: Result<SmallStruct, _> = binary::decode(black_box(&enc));
        });
    });

    group.bench_function("medium_struct", |b| {
        b.iter(|| {
            let enc = binary::encode(black_box(&medium)).unwrap();
            let _: Result<MediumStruct, _> = binary::decode(black_box(&enc));
        });
    });

    group.bench_function("large_struct", |b| {
        b.iter(|| {
            let enc = binary::encode(black_box(&large)).unwrap();
            let _: Result<LargeStruct<'_>, _> = binary::decode(black_box(&enc));
        });
    });

    group.finish();
}

fn bench_simd_performance(c: &mut Criterion) {
    use fastserial::simd;

    let mut group = c.benchmark_group("simd_operations");

    group.bench_function("scan_quote_scalar", |b| {
        let data = br#""hello world this is a long string with "" quotes""#.to_vec();
        b.iter(|| {
            simd::scan_quote_or_backslash(black_box(&data));
        });
    });

    group.bench_function("skip_whitespace_scalar", |b| {
        let data = b"                                                  hello".to_vec();
        b.iter(|| {
            simd::skip_whitespace(black_box(&data));
        });
    });

    group.bench_function("is_all_ascii_scalar", |b| {
        let data = b"Hello World This Is A Test String For ASCII Checking".to_vec();
        b.iter(|| {
            simd::is_all_ascii(black_box(&data));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_encode_binary,
    bench_decode_binary,
    bench_zero_copy_binary,
    bench_binary_sizes,
    bench_simd_performance
);
criterion_main!(benches);
