use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fastserial::arena::Arena;
use fastserial::json;
use fastserial::tape::TapeNode;
use fastserial::value::Value;

const TWEET_JSON: &str = include_str!("fixtures/twitter.json");

fn bench_dom(c: &mut Criterion) {
    let mut group = c.benchmark_group("DOM Parsing (Twitter)");

    // 1. serde_json::Value
    group.bench_function("serde_json::Value", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(black_box(TWEET_JSON)).unwrap();
            black_box(value);
        });
    });

    // 2. fastserial::value::Value
    group.bench_function("fastserial::Value", |b| {
        let mut data = TWEET_JSON.as_bytes().to_vec();
        b.iter(|| {
            let arena = Arena::new();
            let value: Value = json::decode(black_box(&mut data), &arena).unwrap();
            black_box(value);
        });
    });

    // 3. fastserial::tape::TapeNode
    group.bench_function("fastserial::TapeNode", |b| {
        let mut data = TWEET_JSON.as_bytes().to_vec();
        b.iter(|| {
            let arena = Arena::new();
            let value: TapeNode = json::decode(black_box(&mut data), &arena).unwrap();
            black_box(value);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_dom);
criterion_main!(benches);
