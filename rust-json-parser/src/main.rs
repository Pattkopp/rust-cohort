use rust_json_parser::JsonParser;

fn main() {
    let input = r#""The quick brown fox jumps over the lazy dog""#;
    let result = JsonParser::new(input);
    println!("Input JSON: {input}");
    println!("\nResult:");
    match result {
        Ok(value) => println!("Value: {:?}", value),
        Err(err) => println!("Error: {}", err),
    }
}
