use lopdf::Document;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use clap::Parser;

#[derive(Debug, Serialize, Deserialize)]
struct Heading {
    level: String,
    text: String,
    page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Outline {
    title: String,
    outline: Vec<Heading>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let outline = extract_outline(&args.input)?;
    
    let json = serde_json::to_string_pretty(&outline)?;
    File::create(args.output)?.write_all(json.as_bytes())?;
    
    Ok(())
}

fn extract_outline(pdf_path: &str) -> Result<Outline, Box<dyn std::error::Error>> {
    let doc = Document::load(pdf_path)?;
    let mut title = String::new();
    let mut headings = Vec::new();
    let heading_re = Regex::new(r"^(\d+\.)+\s+")?;

    // Correctly handle page iteration
    for (page_num, (page_id, _)) in doc.page_iter().enumerate().take(50) {
        if let Ok(text) = doc.extract_text(&[page_id]) {
            for line in text.lines() {
                if title.is_empty() && line.len() > 10 {
                    title = line.trim().to_string();
                }
                
                if heading_re.is_match(line) {
                    let level = line.split('.').count();
                    if level <= 3 {
                        headings.push(Heading {
                            level: format!("H{}", level),
                            text: line.trim().to_string(),
                            page: page_num + 1,
                        });
                    }
                }
            }
        }
    }

    Ok(Outline {
        title: if title.is_empty() {
            Path::new(pdf_path).file_stem().unwrap().to_str().unwrap().to_string()
        } else {
            title
        },
        outline: headings,
    })
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,
}