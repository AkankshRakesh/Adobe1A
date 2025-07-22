use lopdf::Document;
use pdf_extract;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use clap::Parser;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use once_cell::sync::Lazy;

mod functions;

// Pre-compile regex patterns for better performance
pub static TITLE_PATTERN: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"(?i)^\s*(RFP|Request\s+for\s+Proposal|Proposal|Scope\s+of\s+Work)\s*:?\s*(.*)$").unwrap());
pub static NUMBERED_HEADING: Lazy<Regex> = Lazy::new(|| 
    Regex::new(r"^\s*((\d+\.)+\s*[A-Z].*|^[IVX]+\.?\s+[A-Z].*)").unwrap());
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
    // First try using pdf-extract which handles more PDF formats
    if let Ok(outline) = try_pdf_extract(pdf_path) {
        if !outline.outline.is_empty() {
            return Ok(outline);
        }
    }

    // Fall back to lopdf if pdf-extract fails
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
    let mut first_page_text = String::new();

    // Split text into pages (approximate)
    let pages: Vec<&str> = if text.contains('\x0C') {
        text.split('\x0C').collect()
    } else {
        // Fallback splitting by large whitespace sections
        text.split("\n\n\n").collect()
    };

    for (page_num, page_text) in pages.iter().enumerate() {
        let current_page = page_num + 1;
        let lines: Vec<&str> = page_text.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        // Extract title from first page
        if title.is_empty() && current_page == 1 {
            title = functions::extract_document_title(&lines, page_text);
        }

        // Process potential headings
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

fn extract_document_title(lines: &[&str], first_page_text: &str) -> String {
    // First try to find RFP-style title
    for line in lines.iter().take(10) {
        if let Some(caps) = TITLE_PATTERN.captures(line) {
            if let Some(title_part) = caps.get(2) {
                let candidate = title_part.as_str().trim();
                if !candidate.is_empty() {
                    return candidate.to_string();
                }
            }
        }
    }

    // Fallback to first significant line
    for line in lines.iter().take(10) {
        let line = line.trim();
        if line.len() > 10 && line.len() < 150 && 
           !line.starts_with("Page ") && 
           !line.contains("http") &&
           line.chars().next().map_or(false, |c| c.is_uppercase()) {
            return line.to_string();
        }
    }

    // Final fallback - first non-empty line
    first_page_text.lines()
        .find(|l| !l.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Untitled Document".to_string())
}

fn analyze_potential_heading(
    line: &str,
    line_index: usize,
    all_lines: &[&str],
    page: usize,
    numbered_heading: &Regex,
    chapter_heading: &Regex,
    all_caps_heading: &Regex,
    title_pattern: &Regex,
) -> Option<Heading> {
    let line = line.trim();
    
    // Skip very short or very long lines
    if line.len() < 3 || line.len() > 200 {
        return None;
    }
    
    // Skip lines that are clearly not headings
    if is_excluded_text(line) {
        return None;
    }
    
    let mut confidence_score = 0;
    let mut level = "H1".to_string();
    
    // Check for numbered headings (1.1.1 Format)
    if numbered_heading.is_match(line) {
        confidence_score += 50;
        level = determine_numbered_level(line);
    }
    
    // Check for chapter/section headings
    if chapter_heading.is_match(line) {
        confidence_score += 40;
        level = "H1".to_string();
    }
    
    // Check for all caps headings (but more selective)
    if all_caps_heading.is_match(line) && line.split_whitespace().count() >= 2 {
        confidence_score += 25;
    }
    
    // Check formatting context clues
    if line_index > 0 && line_index < all_lines.len() - 1 {
        let prev_line = all_lines[line_index - 1].trim();
        let next_line = all_lines[line_index + 1].trim();
        
        // Heading likely if surrounded by empty space or different formatting
        if prev_line.is_empty() || next_line.is_empty() {
            confidence_score += 10;
        }
        
        // Check if line stands out (different case pattern from surrounding)
        if is_different_formatting(line, prev_line, next_line) {
            confidence_score += 15;
        }
    }
    
    // Check word characteristics
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.len() >= 2 && words.len() <= 10 {
        confidence_score += 10;
        
        // Boost for common heading words
        if contains_heading_keywords(line) {
            confidence_score += 20;
        }
    }
    
    // Apply penalties for table-like content
    if line.contains('\t') || line.matches(' ').count() > 10 {
        confidence_score -= 20;
    }
    
    // Only consider as heading if confidence is high enough
    if confidence_score >= 35 {
        Some(Heading {
            level,
            text: clean_heading_text(line),
            page,
        })
    } else {
        None
    }
}

fn is_excluded_text(line: &str) -> bool {
    let exclusions = [
        "page", "www.", "http", "@", "©", "copyright",
        "table of contents", "index", "references", "bibliography",
        "appendix a", "appendix b", "appendix c"
    ];
    
    let line_lower = line.to_lowercase();
    exclusions.iter().any(|&exclusion| line_lower.contains(exclusion)) ||
    line.chars().all(|c| c.is_numeric() || c.is_whitespace() || ".-()".contains(c))
}

fn determine_numbered_level(line: &str) -> String {
    let dot_count = line.chars().take_while(|c| c.is_numeric() || *c == '.').filter(|&c| c == '.').count();
    match dot_count {
        1 => "H1".to_string(),
        2 => "H2".to_string(),
        3 => "H3".to_string(),
        _ => "H4".to_string(),
    }
}

fn is_different_formatting(line: &str, prev: &str, next: &str) -> bool {
    let line_caps = line.chars().filter(|c| c.is_uppercase()).count();
    let line_total = line.chars().filter(|c| c.is_alphabetic()).count();
    
    if line_total == 0 { return false; }
    
    let line_caps_ratio = line_caps as f32 / line_total as f32;
    
    // Check if this line has significantly different capitalization
    line_caps_ratio > 0.5 && (prev.len() < 3 || next.len() < 3 || 
        line.len() < prev.len() / 2 || line.len() < next.len() / 2)
}

fn contains_heading_keywords(line: &str) -> bool {
    let keywords = [
        "introduction", "overview", "summary", "conclusion", "abstract",
        "methodology", "results", "discussion", "background", "objective",
        "purpose", "scope", "definition", "implementation", "analysis",
        "requirements", "design", "architecture", "testing", "validation"
    ];
    
    let line_lower = line.to_lowercase();
    keywords.iter().any(|&keyword| line_lower.contains(keyword))
}

fn clean_heading_text(text: &str) -> String {
    // Remove excessive whitespace and clean up the text
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn establish_hierarchy(mut headings: Vec<Heading>) -> Vec<Heading> {
    // Remove very similar headings (potential duplicates)
    let mut filtered = Vec::new();
    let mut seen_texts = HashSet::new();
    
    for heading in headings {
        let normalized = heading.text.to_lowercase().replace(&[' ', '.', '-'][..], "");
        if !seen_texts.contains(&normalized) && normalized.len() > 2 {
            seen_texts.insert(normalized);
            filtered.push(heading);
        }
    }
    
    // Sort by page and then by the order they appear
    filtered.sort_by(|a, b| a.page.cmp(&b.page));
    
    filtered
}