use fastserial::arena::Arena;
use fastserial::json;
use fastserial::tape::TapeNode;

#[test]
fn test_tape_node() {
    let mut data = br#"
    {
        "name": "fastserial",
        "speed": 100,
        "features": ["zero-copy", "simd", "tape"],
        "is_fast": true,
        "nested": {
            "key": "value"
        }
    }
    "#
    .to_vec();

    let arena = Arena::new();
    let root: TapeNode = json::decode(&mut data, &arena).unwrap();

    let name = root.get("name").unwrap().as_str().unwrap();
    assert_eq!(name, "fastserial");

    let is_fast = root.get("is_fast").unwrap();
    assert_eq!(*is_fast, TapeNode::Bool(true));

    let speed = root.get("speed").unwrap();
    assert_eq!(*speed, TapeNode::U64(100));

    let features = root.get("features").unwrap().as_array().unwrap();
    assert_eq!(features.len(), 3);
    assert_eq!(features[0].as_str().unwrap(), "zero-copy");

    let nested = root.get("nested").unwrap();
    assert_eq!(nested.get("key").unwrap().as_str().unwrap(), "value");
}
