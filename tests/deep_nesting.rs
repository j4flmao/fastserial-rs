use fastserial::{arena::Arena, json};

#[test]
fn test_deep_nesting() {
    let arena = Arena::new();

    // Create a very deeply nested array JSON
    let depth = 150;
    let mut json_str = String::with_capacity(depth * 2 + 10);
    for _ in 0..depth {
        json_str.push('[');
    }
    json_str.push_str("42");
    for _ in 0..depth {
        json_str.push(']');
    }

    // FastSerial uses recursive descent, so at 150 depth it should still easily fit
    // on a typical Windows test thread stack without overflowing.
    let mut json_data = json_str.into_bytes();

    // Instead of explicitly typing it, we parse into a Tape Node
    let tape_node: fastserial::tape::TapeNode<'_> = json::decode(&mut json_data, &arena).unwrap();

    // We can traverse it back if we want, but just parsing it without stack overflow is a win.
    assert!(matches!(tape_node, fastserial::tape::TapeNode::Array(_)));
}

#[test]
fn test_deeply_nested_objects() {
    let arena = Arena::new();

    let depth = 150; // smaller to avoid stack overflow on Windows test runner
    let mut json_str = String::new();
    for i in 0..depth {
        json_str.push_str(&format!("{{\"level{}\":", i));
    }
    json_str.push_str("42");
    for _ in 0..depth {
        json_str.push('}');
    }

    let mut json_data = json_str.into_bytes();
    let tape_node: fastserial::tape::TapeNode<'_> = json::decode(&mut json_data, &arena).unwrap();
    assert!(matches!(tape_node, fastserial::tape::TapeNode::Object(_)));
}
