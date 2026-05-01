use rust_json_parser::parse_json;

fn main() {
    let input = r#""The quick brown fox jumps over the lazy dog""#;
    let result = parse_json(input);
    println!("Input JSON: {input}");
    println!("\nResult:");
    match result {
        Ok(value) => println!("Value: {:?}", value),
        Err(err) => println!("Error: {}", err),
    }
}
