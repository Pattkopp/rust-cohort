use rust_json_parser::parse_json;

fn main() {
    let input = r#""missing end quote"#;
    let result = parse_json(input);
    println!("Input JSON: {input}");
    println!("\nResult:");
    println!("{:?}", result);
}
