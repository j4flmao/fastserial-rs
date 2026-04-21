use fastserial::{Decode, Encode, json};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
    balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerdeUser {
    id: u64,
    username: String,
    email: String,
    active: bool,
    balance: f64,
}

fn format_ratio(ratio: f64, faster: bool) -> String {
    if faster {
        format!("{:.2}x FASTER", ratio)
    } else {
        format!("{:.2}x slower", ratio)
    }
}

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║        FastSerial vs Serde_JSON - Performance Report          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let user = User {
        id: 12345,
        username: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    let serde_user = SerdeUser {
        id: 12345,
        username: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        balance: 1234.56,
    };

    let iterations = 100_000;

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                  ENCODE BENCHMARK                          │");
    println!("├─────────────────────────────────────────────────────────────┤");

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(json::encode(&user).unwrap());
    }
    let fastserial_encode = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(serde_json::to_vec(&serde_user).unwrap());
    }
    let serde_encode = start.elapsed();

    let fastserial_ns = fastserial_encode.as_nanos() / iterations;
    let serde_ns = serde_encode.as_nanos() / iterations;
    let encode_ratio = serde_encode.as_nanos() as f64 / fastserial_encode.as_nanos() as f64;

    println!("│ Test: Single User struct                                     │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ FastSerial:  {:>10} ns/op                                    │",
        fastserial_ns
    );
    println!(
        "│ Serde_JSON:{:>10} ns/op                                    │",
        serde_ns
    );
    println!(
        "│ Result: {}",
        format_ratio(encode_ratio, encode_ratio > 1.0)
    );
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                  DECODE BENCHMARK                          │");
    println!("├─────────────────────────────────────────────────────────────┤");

    let fastserial_encoded = json::encode(&user).unwrap();
    let serde_encoded = serde_json::to_vec(&serde_user).unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(json::decode::<User>(&fastserial_encoded).unwrap());
    }
    let fastserial_decode = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(serde_json::from_slice::<SerdeUser>(&serde_encoded).unwrap());
    }
    let serde_decode = start.elapsed();

    let fastserial_decode_ns = fastserial_decode.as_nanos() / iterations;
    let serde_decode_ns = serde_decode.as_nanos() / iterations;
    let decode_ratio = serde_decode.as_nanos() as f64 / fastserial_decode.as_nanos() as f64;

    println!("│ Test: Single User struct                                     │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ FastSerial:  {:>10} ns/op                                    │",
        fastserial_decode_ns
    );
    println!(
        "│ Serde_JSON:{:>10} ns/op                                    │",
        serde_decode_ns
    );
    println!(
        "│ Result: {}",
        format_ratio(decode_ratio, decode_ratio > 1.0)
    );
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              LARGE DATA BENCHMARK                           │");
    println!("├─────────────────────────────────────────────────────────────┤");

    let users: Vec<User> = (0..100)
        .map(|i| User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    let serde_users: Vec<SerdeUser> = (0..100)
        .map(|i| SerdeUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
            balance: (i as f64) * 1.5,
        })
        .collect();

    let large_iterations = 10_000;

    let start = Instant::now();
    for _ in 0..large_iterations {
        std::hint::black_box(json::encode(&users).unwrap());
    }
    let fastserial_large_encode = start.elapsed();

    let start = Instant::now();
    for _ in 0..large_iterations {
        std::hint::black_box(serde_json::to_vec(&serde_users).unwrap());
    }
    let serde_large_encode = start.elapsed();

    let fastserial_large_encode_us = fastserial_large_encode.as_micros() / large_iterations;
    let serde_large_encode_us = serde_large_encode.as_micros() / large_iterations;
    let large_encode_ratio =
        serde_large_encode.as_nanos() as f64 / fastserial_large_encode.as_nanos() as f64;

    println!("│ Encode Vec<100 Users>                                       │");
    println!("├──────────────────────────────────���──────────────────────────┤");
    println!(
        "│ FastSerial:  {:>10} µs/op                                   │",
        fastserial_large_encode_us
    );
    println!(
        "│ Serde_JSON:{:>10} µs/op                                   │",
        serde_large_encode_us
    );
    println!(
        "│ Result: {}",
        format_ratio(large_encode_ratio, large_encode_ratio > 1.0)
    );
    println!("├─────────────────────────────────────────────────────────────┤");

    let fastserial_large_encoded = json::encode(&users).unwrap();
    let serde_large_encoded = serde_json::to_vec(&serde_users).unwrap();

    let start = Instant::now();
    for _ in 0..large_iterations {
        std::hint::black_box(json::decode::<Vec<User>>(&fastserial_large_encoded).unwrap());
    }
    let fastserial_large_decode = start.elapsed();

    let start = Instant::now();
    for _ in 0..large_iterations {
        std::hint::black_box(
            serde_json::from_slice::<Vec<SerdeUser>>(&serde_large_encoded).unwrap(),
        );
    }
    let serde_large_decode = start.elapsed();

    let fastserial_large_decode_us = fastserial_large_decode.as_micros() / large_iterations;
    let serde_large_decode_us = serde_large_decode.as_micros() / large_iterations;
    let large_decode_ratio =
        serde_large_decode.as_nanos() as f64 / fastserial_large_decode.as_nanos() as f64;

    println!("│ Decode Vec<100 Users>                                       │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ FastSerial:  {:>10} µs/op                                   │",
        fastserial_large_decode_us
    );
    println!(
        "│ Serde_JSON:{:>10} µs/op                                   │",
        serde_large_decode_us
    );
    println!(
        "│ Result: {}",
        format_ratio(large_decode_ratio, large_decode_ratio > 1.0)
    );
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STRING ESCAPE BENCHMARK                      │");
    println!("├─────────────────────────────────────────────────────────────┤");

    let user_long = User {
        id: 1,
        username: "this_is_a_very_long_username_that_contains_many_characters".to_string(),
        email: "this_is_a_very_long_email_address_that_contains_many_characters@example.com"
            .to_string(),
        active: true,
        balance: 1234.56,
    };

    let serde_user_long = SerdeUser {
        id: 1,
        username: "this_is_a_very_long_username_that_contains_many_characters".to_string(),
        email: "this_is_a_very_long_email_address_that_contains_many_characters@example.com"
            .to_string(),
        active: true,
        balance: 1234.56,
    };

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(json::encode(&user_long).unwrap());
    }
    let fastserial_str_encode = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(serde_json::to_vec(&serde_user_long).unwrap());
    }
    let serde_str_encode = start.elapsed();

    let fastserial_str_ns = fastserial_str_encode.as_nanos() / iterations;
    let serde_str_ns = serde_str_encode.as_nanos() / iterations;
    let str_encode_ratio =
        serde_str_encode.as_nanos() as f64 / fastserial_str_encode.as_nanos() as f64;

    println!("│ Encode with Long Strings                                   │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ FastSerial:  {:>10} ns/op                                    │",
        fastserial_str_ns
    );
    println!(
        "│ Serde_JSON:{:>10} ns/op                                    │",
        serde_str_ns
    );
    println!(
        "│ Result: {}",
        format_ratio(str_encode_ratio, str_encode_ratio > 1.0)
    );
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY                               ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║                                                          ║");
    println!("║  FastSerial ADVANTAGES:                                      ║");
    println!("║  • Zero-copy deserialization (borrows from input)          ║");
    println!("║  • SIMD-accelerated string scanning                        ║");
    println!("║  • Custom derive macros for generated code                ║");
    println!("║  • No_std support                                          ║");
    println!("║  • Binary format with schema validation                       ║");
    println!("║                                                          ║");
    println!("║  Performance Summary:                                       ║");
    println!(
        "║  • Encode: {}                                              ║",
        format_ratio(encode_ratio, encode_ratio > 1.0)
    );
    println!(
        "║  • Long Strings: {}                                          ║",
        format_ratio(str_encode_ratio, str_encode_ratio > 1.0)
    );
    println!(
        "║  • Large Data Encode: {}                                    ║",
        format_ratio(large_encode_ratio, large_encode_ratio > 1.0)
    );
    println!("║                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
}
