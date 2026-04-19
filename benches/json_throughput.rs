use criterion::{Criterion, criterion_group, criterion_main};
use fastserial::{Decode, Encode, json};

#[derive(Encode, Decode, Debug, Clone)]
struct BenchmarkStruct {
    id: u64,
    name: String,
    email: String,
    active: bool,
    balance: f64,
}

fn create_test_data() -> BenchmarkStruct {
    BenchmarkStruct {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    }
}

fn bench_encode_json(c: &mut Criterion) {
    let data = create_test_data();
    let mut group = c.benchmark_group("fastserial_encode");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let items: Vec<_> = (0..size).map(|_| data.clone()).collect();
                b.iter(|| {
                    for item in &items {
                        let _ = json::encode(std::hint::black_box(item));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_decode_json(c: &mut Criterion) {
    let data = create_test_data();
    let encoded = json::encode(&data).unwrap();

    let mut group = c.benchmark_group("fastserial_decode");

    for size in [1, 10, 100, 1000].iter() {
        let items: Vec<_> = (0..*size).map(|_| encoded.clone()).collect();

        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let slices: Vec<_> = items.iter().take(size).collect();
                b.iter(|| {
                    for slice in &slices {
                        let _ =
                            json::decode::<BenchmarkStruct>(std::hint::black_box(slice.as_slice()));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_serde_json_encode(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct SerdeStruct {
        id: u64,
        name: String,
        email: String,
        active: bool,
        balance: f64,
    }

    let data = SerdeStruct {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    let mut group = c.benchmark_group("serde_json_encode");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let items: Vec<_> = (0..size).map(|_| &data).collect();
                b.iter(|| {
                    for item in &items {
                        let _ = serde_json::to_vec(std::hint::black_box(item));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_serde_json_decode(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct SerdeStruct {
        id: u64,
        name: String,
        email: String,
        active: bool,
        balance: f64,
    }

    let data = SerdeStruct {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    let encoded = serde_json::to_vec(&data).unwrap();

    let mut group = c.benchmark_group("serde_json_decode");

    for size in [1, 10, 100, 1000].iter() {
        let items: Vec<_> = (0..*size).map(|_| encoded.clone()).collect();

        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let slices: Vec<_> = items.iter().take(size).collect();
                b.iter(|| {
                    for slice in &slices {
                        let _ = serde_json::from_slice::<SerdeStruct>(std::hint::black_box(
                            slice.as_slice(),
                        ));
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encode_json,
    bench_decode_json,
    bench_serde_json_encode,
    bench_serde_json_decode
);
criterion_main!(benches);
