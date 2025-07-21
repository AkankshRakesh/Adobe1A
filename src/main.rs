use lopdf::Document;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Heading {
    pub level: String,
    pub text: String,
    pub page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outline {
    pub title: String,
    pub outline: Vec<Heading>,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let outline = extract_outline(&args.input)?;
    std::fs::write(args.output, serde_json::to_string_pretty(&outline)?)?;
    Ok(())
}

fn extract_outline(pdf_path: &PathBuf) -> Result<Outline, Box<dyn std::error::Error>> {
    let doc = Document::load(pdf_path)?;
    let mut title = String::new();
    let mut headings = Vec::new();

    for (page_num, &page_id) in doc.get_pages().iter().take(50) {
    let text = doc.extract_text(&[page_id.0])?;
    let lines = text.split('\n').collect::<Vec<_>>();

    if title.is_empty() {
        title = lines.first().unwrap_or(&"").to_string();
    }

    for line in lines {
        if line.len() > 5 && line.chars().next().unwrap().is_uppercase() {
            headings.push(Heading {
                level: "H1".to_string(),
                text: line.to_string(),
                page: (*page_num as usize) + 1,
            });
        }
    }
}


    Ok(Outline {
        title,
        outline: headings,
    })
}