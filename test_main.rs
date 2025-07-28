use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments received: {:?}", args);
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_pdf> <output_json>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    println!("Input: {}", input_path);
    println!("Output: {}", output_path);
    
    // Simple test - just write a test file
    match std::fs::write(output_path, "test") {
        Ok(_) => println!("Successfully wrote test file"),
        Err(e) => eprintln!("Error writing file: {}", e),
    }
}
