use rust_json_parser::JsonParser;

fn main() {
    let input = "true"; // &'static str => borrow - who owns "true"?
    // let input: String = String::from("true"); --> now input owns "true"
    let result = JsonParser::new().parse(input);
    println!("Input JSON: {input}");
    println!("\nResult:");
    match result {
        Ok(value) => println!("Value: {:?}", value),
        Err(err) => println!("Error: {}", err),
    }
}
