use serde_json::json;

fn main() {
    // Test 1: Control characters
    let data = json!({"control": "\x00\x1f\t\n"});
    let serialized = serde_json::to_string(&data).unwrap();
    println!("Control chars: {serialized}");

    // Test 2: Emoji (4-byte UTF-8)
    let data = json!({"emoji": "😀"});
    let serialized = serde_json::to_string(&data).unwrap();
    println!("Emoji: {serialized}");

    // Test 3: Parse \uXXXX
    let json_str = r#"{"test": "hello\u0020world"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    println!("Parsed \\u0020: {:?}", parsed["test"].as_str().unwrap());

    // Test 4: Roundtrip control chars
    let json_str = r#"{"test": "\u0000\u001f"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let reserialized = serde_json::to_string(&parsed).unwrap();
    println!("Roundtrip: {json_str} -> {reserialized}");
}
