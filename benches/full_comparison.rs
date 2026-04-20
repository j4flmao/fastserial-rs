use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fastserial::{Decode, Encode, json};
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerdeUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub balance: f64,
}

fn create_user() -> User {
    User {
        id: 12345,
        username: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    }
}

fn create_serde_user() -> SerdeUser {
    SerdeUser {
        id: 12345,
        username: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    }
}

fn fastserial_encode_small(c: &mut Criterion) {
    let user = create_user();
    c.bench_function("fastserial_encode_small", |b| {
        b.iter(|| json::encode(std::hint::black_box(&user)))
    });
}

fn serde_encode_small(c: &mut Criterion) {
    let user = create_serde_user();
    c.bench_function("serde_encode_small", |b| {
        b.iter(|| serde_json::to_vec(std::hint::black_box(&user)))
    });
}

fn fastserial_decode_small(c: &mut Criterion) {
    let encoded = json::encode(&create_user()).unwrap();
    c.bench_function("fastserial_decode_small", |b| {
        b.iter(|| json::decode::<User>(std::hint::black_box(&encoded)))
    });
}

fn serde_decode_small(c: &mut Criterion) {
    let user = create_serde_user();
    let encoded = serde_json::to_vec(&user).unwrap();
    c.bench_function("serde_decode_small", |b| {
        b.iter(|| serde_json::from_slice::<SerdeUser>(std::hint::black_box(&encoded)))
    });
}

fn fastserial_encode_large(c: &mut Criterion) {
    let users: Vec<User> = (0..100)
        .map(|i| User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    c.bench_function("fastserial_encode_large", |b| {
        b.iter(|| json::encode(std::hint::black_box(&users)))
    });
}

fn serde_encode_large(c: &mut Criterion) {
    let users: Vec<SerdeUser> = (0..100)
        .map(|i| SerdeUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    c.bench_function("serde_encode_large", |b| {
        b.iter(|| serde_json::to_vec(std::hint::black_box(&users)))
    });
}

fn fastserial_decode_large(c: &mut Criterion) {
    let users: Vec<User> = (0..100)
        .map(|i| User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    let encoded = json::encode(&users).unwrap();

    c.bench_function("fastserial_decode_large", |b| {
        b.iter(|| json::decode::<Vec<User>>(std::hint::black_box(&encoded)))
    });
}

fn serde_decode_large(c: &mut Criterion) {
    let users: Vec<SerdeUser> = (0..100)
        .map(|i| SerdeUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    let encoded = serde_json::to_vec(&users).unwrap();

    c.bench_function("serde_decode_large", |b| {
        b.iter(|| serde_json::from_slice::<Vec<SerdeUser>>(std::hint::black_box(&encoded)))
    });
}

fn fastserial_encode_long_string(c: &mut Criterion) {
    let user = User {
        id: 1,
        username: "this_is_a_very_long_username_that_contains_many_characters_to_test_escape_performance".to_string(),
        email: "this_is_a_very_long_email_address_that_contains_many_characters_to_test_escape_performance@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    c.bench_function("fastserial_encode_long_string", |b| {
        b.iter(|| json::encode(std::hint::black_box(&user)))
    });
}

fn serde_encode_long_string(c: &mut Criterion) {
    let user = SerdeUser {
        id: 1,
        username: "this_is_a_very_long_username_that_contains_many_characters_to_test_escape_performance".to_string(),
        email: "this_is_a_very_long_email_address_that_contains_many_characters_to_test_escape_performance@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    c.bench_function("serde_encode_long_string", |b| {
        b.iter(|| serde_json::to_vec(std::hint::black_box(&user)))
    });
}

fn fastserial_encode_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("fastserial_encode_many");
    let user = create_user();

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    std::hint::black_box(json::encode(&user).unwrap());
                }
            });
        });
    }
    group.finish();
}

fn serde_encode_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_encode_many");
    let user = create_serde_user();

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    std::hint::black_box(serde_json::to_vec(&user).unwrap());
                }
            });
        });
    }
    group.finish();
}

fn fastserial_decode_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("fastserial_decode_many");
    let encoded = json::encode(&create_user()).unwrap();

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    std::hint::black_box(json::decode::<User>(&encoded).unwrap());
                }
            });
        });
    }
    group.finish();
}

fn serde_decode_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_decode_many");
    let user = create_serde_user();
    let encoded = serde_json::to_vec(&user).unwrap();

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    std::hint::black_box(serde_json::from_slice::<SerdeUser>(&encoded).unwrap());
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    fastserial_encode_small,
    serde_encode_small,
    fastserial_decode_small,
    serde_decode_small,
    fastserial_encode_large,
    serde_encode_large,
    fastserial_decode_large,
    serde_decode_large,
    fastserial_encode_long_string,
    serde_encode_long_string,
    fastserial_encode_many,
    serde_encode_many,
    fastserial_decode_many,
    serde_decode_many
);
criterion_main!(benches);
