use rust_json_parser::JsonParser;

fn main() {
    let input = r#"{"say \"hi\"": 42}"#;
    let result = JsonParser::new().parse(input);
    println!("Input JSON: {input}");
    println!("\nResult:");
    match result {
        Ok(value) => println!("Value: {}", value.pretty_print(2)),
        Err(err) => println!("Error: {}", err),
    }
}
