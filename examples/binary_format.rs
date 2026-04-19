use fastserial::{Decode, Encode, binary};

#[derive(Encode, Decode, Debug, Clone)]
struct Config<'de> {
    name: &'de str,
    version: u32,
    debug_mode: bool,
    max_connections: u16,
}

fn main() {
    let config = Config {
        name: "MyApp",
        version: 1,
        debug_mode: true,
        max_connections: 100,
    };

    let bytes = binary::encode(&config).expect("Failed to encode");
    println!("Binary size: {} bytes", bytes.len());
    println!("Hex: {:02x?}", &bytes);

    let decoded: Config = binary::decode(&bytes).expect("Failed to decode");
    println!("Decoded: {:?}", decoded);
    assert_eq!(decoded.name, config.name);
    assert_eq!(decoded.version, config.version);
    assert_eq!(decoded.debug_mode, config.debug_mode);
    assert_eq!(decoded.max_connections, config.max_connections);

    println!("\nSchema hash: {:#x}", Config::SCHEMA_HASH);
}
