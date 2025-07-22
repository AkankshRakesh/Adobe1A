use lopdf::Document;
use pdf_extract;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use clap::Parser;
use anyhow::{Context, Result};
use regex::Regex;
use once_cell::sync::Lazy;

mod functions;
mod font_utils;

pub static TITLE_PATTERN: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"(?i)^\s*(RFP|Request\s+for\s+Proposal|Proposal|Scope\s+of\s+Work)\s*:?\s*(.*)$").unwrap());
pub static NUMBERED_HEADING: Lazy<Regex> = Lazy::new(||
    // Matches headings that begin with multi-level decimals like "1.", "1.2.", etc.,
    // single decimals with text ("1 Introduction"), roman numerals ("IV. Scope"),
    // or alpha enumerations such as "A. Background" or "b) Goals".
    Regex::new(r"^\s*(?:((?:\d+\.)+\d*|\d+)[\.)]?\s+.+|[A-Za-z]{1,2}[\.)]\s+.+|[IVXLCDM]+[\.)]?\s+.+)").unwrap());
pub static SECTION_HEADING: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"^\s*(Chapter|Section|Part)\s+([A-Z0-9]+)").unwrap());
pub static APPENDIX_HEADING: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"^\s*Appendix\s+([A-Z0-9]+)").unwrap());
pub static COLON_HEADING: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"^[A-Z][A-Za-z\s]+:$").unwrap());

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

fn main() -> Result<()> {
    let args = Args::parse();
    let outline = extract_outline(&args.input)
        .with_context(|| format!("Failed to process {}", args.input.display()))?;
    
    std::fs::write(&args.output, serde_json::to_string_pretty(&outline)?)?;
    println!("Successfully processed {}", args.input.display());
    Ok(())
}

fn extract_outline(pdf_path: &PathBuf) -> Result<Outline> {
    if let Ok(outline) = try_pdf_extract(pdf_path) {
        if !outline.outline.is_empty() {
            return Ok(outline);
        }
    }

    extract_with_lopdf(pdf_path)
}

fn try_pdf_extract(pdf_path: &PathBuf) -> Result<Outline> {
    let bytes = std::fs::read(pdf_path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)?;
    
    if text.trim().is_empty() {
        return Err(anyhow::anyhow!("No text extracted"));
    }

    let mut title = String::new();
    let mut headings = Vec::new();

    let pages: Vec<&str> = if text.contains('\x0C') {
        text.split('\x0C').collect()
    } else {
        text.split("\n\n\n").collect()
    };

    for (page_num, page_text) in pages.iter().enumerate() {
        let current_page = page_num + 1;
        let lines: Vec<&str> = page_text.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if title.is_empty() && current_page == 1 {
            title = functions::extract_document_title(&lines, page_text);
        }

        for (i, line) in lines.iter().enumerate() {
            if let Some(heading) = functions::analyze_potential_heading(
                line,
                i,
                &lines,
                current_page,
            ) {
                if !headings.iter().any(|h: &Heading| h.text == heading.text && h.page == heading.page) {
                    headings.push(heading);
                }
            }
        }
    }

    Ok(Outline {
        title: if title.is_empty() {
            pdf_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            title
        },
        outline: functions::establish_hierarchy(headings),
    })
}

fn extract_with_lopdf(pdf_path: &PathBuf) -> Result<Outline> {
    let doc = Document::load(pdf_path)?;
    let mut title = String::new();
    let mut headings = Vec::new();
    let mut first_page_text = String::new();

    // First, compute heading size threshold once.
    let runs = font_utils::extract_runs(&doc);
    use std::collections::HashSet;
    let mut sizes: Vec<f64> = runs.iter().map(|r| r.size).collect();
    sizes.sort_by(|a, b| b.partial_cmp(a).unwrap());
    sizes.dedup();
    let heading_threshold = sizes.get(1).copied().unwrap_or(14.0);
    // Build quick lookup of large-font lines by (page,text)
    let mut large_font_set: HashSet<(usize, String)> = HashSet::new();
    for r in &runs {
        if r.size >= heading_threshold {
            large_font_set.insert((r.page, r.text.trim().to_string()));
        }
    }

    // Iterate by page, but we'll also check runs to find candidate lines.
    for (page_index, (page_id, _)) in doc.page_iter().enumerate() {
        let current_page = page_index + 1;
        
        match doc.extract_text(&[page_id]) {
            Ok(text) => {
                if current_page == 1 {
                    first_page_text = text.clone();
                }

                let lines: Vec<&str> = text.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();

                if title.is_empty() {
                    title = functions::extract_document_title(&lines, &first_page_text);
                }

                for (i, line) in lines.iter().enumerate() {
                    if !large_font_set.contains(&(current_page, line.to_string())) {
                        continue;
                    }
                    if let Some(heading) = functions::analyze_potential_heading(
                        line,
                        i,
                        &lines,
                        current_page,
                    ) {
                        if !headings.iter().any(|h: &Heading| h.text == heading.text && h.page == heading.page) {
                            headings.push(heading);
                        }
                    }
                }
            },
            Err(e) => eprintln!("Warning: Could not extract text from page {}: {}", current_page, e),
        }
    }

    Ok(Outline {
        title: if title.is_empty() {
            pdf_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            title
        },
        outline: functions::establish_hierarchy(headings),
    })
}
