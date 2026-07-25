use fastserial::json::{decode, encode};

macro_rules! test_numeric_boundary {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let _arena = fastserial::arena::Arena::new();
            // Test MAX
            let max_val: $ty = <$ty>::MAX;
            let mut json = encode(&max_val).expect("Failed to encode MAX");
            let decoded: $ty = decode(&mut json, &_arena).expect("Failed to decode MAX");
            assert_eq!(max_val, decoded);

            // Test MIN
            let min_val: $ty = <$ty>::MIN;
            let mut json = encode(&min_val).expect("Failed to encode MIN");
            let decoded: $ty = decode(&mut json, &_arena).expect("Failed to decode MIN");
            assert_eq!(min_val, decoded);

            // Test 0
            let zero: $ty = 0 as $ty;
            let mut json = encode(&zero).expect("Failed to encode 0");
            let decoded: $ty = decode(&mut json, &_arena).expect("Failed to decode 0");
            assert_eq!(zero, decoded);
        }
    };
}

test_numeric_boundary!(test_u8_boundary, u8);
test_numeric_boundary!(test_u16_boundary, u16);
test_numeric_boundary!(test_u32_boundary, u32);
test_numeric_boundary!(test_u64_boundary, u64);
test_numeric_boundary!(test_i8_boundary, i8);
test_numeric_boundary!(test_i16_boundary, i16);
test_numeric_boundary!(test_i32_boundary, i32);
test_numeric_boundary!(test_i64_boundary, i64);

#[test]
fn test_f32_boundary() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let val: f32 = 1.2345;
    let mut json = encode(&val).unwrap();
    let decoded: f32 = decode(&mut json, &_arena).unwrap();
    assert!((val - decoded).abs() < 0.0001);
}

#[test]
fn test_f64_boundary() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let val: f64 = 1.23456789;
    let mut json = encode(&val).unwrap();
    let decoded: f64 = decode(&mut json, &_arena).unwrap();
    assert!((val - decoded).abs() < 0.0000001);
}

#[test]
fn test_unit_type() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let val: () = ();
    let mut json = encode(&val).unwrap();
    assert_eq!(json, b"null");
    let _: () = decode(&mut json, &_arena).unwrap();
}
