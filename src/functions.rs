use regex::Regex;
use once_cell::sync::Lazy;
use crate::{Heading, TITLE_PATTERN, NUMBERED_HEADING, APPENDIX_HEADING, SECTION_HEADING, COLON_HEADING};

pub fn extract_document_title(lines: &[&str], _first_page_text: &str) -> String {
    let mut candidate_titles = Vec::new();
    
    for (i, line) in lines.iter().take(20).enumerate() {
        let line = line.trim();
        
        if line.len() < 5 || line.len() > 200 {
            continue;
        }
        
        if line.starts_with("Page ") || 
           line.contains("http") ||
           line.contains("www.") ||
           line.contains("@") ||
           line.contains("©") ||
           line.to_lowercase().contains("table of contents") {
            continue;
        }
        
        let mut score = 0;
        
        score += (20 - i as i32) / 2;
        
        if line.len() >= 20 && line.len() <= 100 {
            score += 15;
        }
        
        let words: Vec<&str> = line.split_whitespace().collect();
        let capitalized_words = words.iter()
            .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
            .count();
        
        if capitalized_words > words.len() / 2 && words.len() >= 2 {
            score += 20;
        }
        
        if line == line.to_uppercase() && line.len() <= 80 {
            score += 10;
        }
        
        let line_lower = line.to_lowercase();
        let title_indicators = [
            "foundation", "guide", "manual", "handbook", "report", "study",
            "analysis", "overview", "introduction", "specification", "standard",
            "requirements", "proposal", "plan", "strategy", "framework",
            "methodology", "principles", "best practices", "guidelines"
        ];
        
        for indicator in &title_indicators {
            if line_lower.contains(indicator) {
                score += 10;
            }
        }
        
        let content_indicators = [
            "the following", "this document", "as described", "according to",
            "it is", "there are", "you will", "we recommend", "please note"
        ];
        
        let has_content_indicators = content_indicators.iter()
            .any(|&indicator| line_lower.contains(indicator));
        
        if has_content_indicators {
            score -= 20;
        }
        
        if line.ends_with('.') && words.len() > 8 {
            score -= 10;
        }
        
        if score > 0 {
            candidate_titles.push((line.to_string(), score));
        }
    }
    
    candidate_titles.sort_by(|a, b| b.1.cmp(&a.1));
    
    if let Some((title, _)) = candidate_titles.first() {
        return title.clone();
    }
    
    for line in lines.iter().take(15) {
        let line = line.trim();
        if line.len() > 10 && line.len() < 150 && 
           !line.starts_with("Page ") && 
           !line.contains("http") &&
           line.chars().next().map_or(false, |c| c.is_uppercase()) {
            return line.to_string();
        }
    }

    "Untitled Document".to_string()
}

pub fn analyze_potential_heading(
    line: &str,
    line_index: usize,
    all_lines: &[&str],
    page: usize,
) -> Option<Heading> {
    let line = line.trim();
    
    if line.len() < 3 || line.len() > 150 {
        return None;
    }
    
    if is_excluded_text(line) {
        return None;
    }

    if NUMBERED_HEADING.is_match(line) {
        return Some(Heading {
            level: determine_numbered_level(line),
            text: clean_heading_text(line),
            page,
        });
    }

    if SECTION_HEADING.is_match(line) {
        return Some(Heading {
            level: "H1".to_string(),
            text: clean_heading_text(line),
            page,
        });
    }

    if APPENDIX_HEADING.is_match(line) {
        return Some(Heading {
            level: "H1".to_string(),
            text: clean_heading_text(line),
            page,
        });
    }

    if line == line.to_uppercase() && line.len() > 5 {
        let word_count = line.split_whitespace().count();
        if word_count >= 2 && word_count <= 8 {
            let is_isolated = is_line_isolated(line_index, all_lines);
            if is_isolated {
                return Some(Heading {
                    level: "H1".to_string(),
                    text: clean_heading_text(line),
                    page,
                });
            }
        }
    }

    if line.ends_with(':') && !line.ends_with("::") {
        let word_count = line.split_whitespace().count();
        if word_count >= 2 && word_count <= 10 && line.len() >= 8 && line.len() <= 80 {
            let has_heading_context = is_line_isolated(line_index, all_lines) ||
                                    has_following_content(line_index, all_lines);
            if has_heading_context {
                return Some(Heading {
                    level: "H2".to_string(),
                    text: clean_heading_text(line),
                    page,
                });
            }
        }
    }

    let words: Vec<&str> = line.split_whitespace().collect();
    if words.len() >= 2 && words.len() <= 8 {
        let capitalized_words = words.iter()
            .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
            .count();
        
        if capitalized_words >= words.len() - 1 && capitalized_words >= 2 {
            let is_well_formed = line.len() >= 10 && line.len() <= 80 &&
                               is_line_isolated(line_index, all_lines) &&
                               has_meaningful_words(&words);
            
            if is_well_formed {
                return Some(Heading {
                    level: determine_heading_level_by_content(line),
                    text: clean_heading_text(line),
                    page,
                });
            }
        }
    }

    None
}

fn is_line_isolated(line_index: usize, all_lines: &[&str]) -> bool {
    let has_blank_before = line_index == 0 || 
                          all_lines.get(line_index.saturating_sub(1))
                          .map_or(true, |l| l.trim().is_empty());
    let has_blank_after = line_index >= all_lines.len().saturating_sub(1) || 
                         all_lines.get(line_index + 1)
                         .map_or(true, |l| l.trim().is_empty());
    
    has_blank_before && has_blank_after
}

fn has_following_content(line_index: usize, all_lines: &[&str]) -> bool {
    if let Some(next_line) = all_lines.get(line_index + 1) {
        let next_line = next_line.trim();
        return !next_line.is_empty() && 
               next_line.len() > 20 && 
               next_line.chars().next().map_or(false, |c| c.is_lowercase());
    }
    false
}

fn has_meaningful_words(words: &[&str]) -> bool {
    let meaningful_count = words.iter()
        .filter(|word| word.len() > 3 && 
                      !["The", "And", "For", "With", "From", "That", "This", "Into", "Upon"].contains(word))
        .count();
    
    meaningful_count >= words.len() / 2
}

fn determine_heading_level_by_content(line: &str) -> String {
    let line_lower = line.to_lowercase();
    
    let h1_indicators = [
        "introduction", "overview", "summary", "conclusion", "background",
        "methodology", "results", "discussion", "abstract", "executive summary"
    ];
    
    let h2_indicators = [
        "objectives", "requirements", "scope", "limitations", "assumptions",
        "definitions", "terminology", "approach", "process", "procedure"
    ];
    
    for indicator in &h1_indicators {
        if line_lower.contains(indicator) {
            return "H1".to_string();
        }
    }
    
    for indicator in &h2_indicators {
        if line_lower.contains(indicator) {
            return "H2".to_string();
        }
    }
    
    "H2".to_string()
}

pub fn establish_hierarchy(headings: Vec<Heading>) -> Vec<Heading> {
    let mut unique_headings = Vec::new();
    let mut seen_texts: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for heading in &headings {
        let normalized_text = heading.text.to_lowercase().trim().to_string();
        
        let text_without_numbers = heading.text.chars()
            .filter(|c| !c.is_ascii_digit() && *c != '.' && *c != ':')
            .collect::<String>()
            .trim()
            .to_lowercase();
            
        let is_duplicate = seen_texts.iter().any(|seen| {
            let seen_without_numbers = seen.chars()
                .filter(|c| !c.is_ascii_digit() && *c != '.' && *c != ':')
                .collect::<String>()
                .trim()
                .to_lowercase();
            seen_without_numbers == text_without_numbers && 
            !text_without_numbers.is_empty() &&
            text_without_numbers.len() > 5  
        });
        
        if !is_duplicate {
            seen_texts.insert(normalized_text);
            unique_headings.push(heading.clone());
        }
    }
    
    unique_headings.sort_by(|a, b| a.page.cmp(&b.page));
    unique_headings
}

pub fn is_excluded_text(line: &str) -> bool {
    let line_lower = line.to_lowercase();
    
    let generic_exclusions = [
        "www.", "http", "@", "©", "copyright", "page ",
        "table of contents", "index", "references", "bibliography",
        "acknowledgments", "acknowledgements", "preface", "foreword"
    ];
    
    if generic_exclusions.iter().any(|&exclusion| line_lower.contains(exclusion)) {
        return true;
    }
    
    let non_letter_count = line.chars().filter(|c| !c.is_alphabetic()).count();
    let total_chars = line.chars().count();
    
    if total_chars > 0 && non_letter_count as f64 / total_chars as f64 > 0.7 {
        return true;
    }
    
    if line.trim().len() < 3 {
        return true;
    }
    
    if line.trim().len() < 20 && (
        line_lower.starts_with("page ") ||
        line_lower.contains("chapter ") ||
        line_lower.matches(char::is_numeric).count() > line.len() / 3
    ) {
        return true;
    }
    
    if (line.contains("$") || line.contains("€") || line.contains("£")) &&
       line.matches(char::is_numeric).count() > 2 {
        return true;
    }
    
    let prose_patterns = [
        "the following", "as mentioned", "according to", "it should be noted",
        "please refer", "see section", "as shown in", "this chapter",
        "in this document", "the purpose of", "it is important"
    ];
    
    if prose_patterns.iter().any(|&pattern| line_lower.contains(pattern)) {
        return true;
    }
    
    if line.ends_with(',') || line.ends_with("and") || line.ends_with("or") || 
       line.ends_with("the") || line.ends_with("of") || line.ends_with("in") ||
       line.ends_with("to") || line.ends_with("for") || line.ends_with("with") {
        return true;
    }
    
    if line.chars().next().map_or(false, |c| c.is_lowercase()) &&
       !line.starts_with('(') && !line.starts_with('[') {
        return true;
    }
    
    false
}

pub fn determine_numbered_level(line: &str) -> String {
    let numbering_part = line.split_whitespace().next().unwrap_or("");
    
    // Count dots in the numbering part
    let dot_count = numbering_part.chars().filter(|&c| c == '.').count();
    
    match dot_count {
        0 => {
            // Single number like "9" -> H1
            if numbering_part.chars().all(|c| c.is_numeric()) {
                "H1".to_string()
            } else {
                "H1".to_string()  // Roman numerals or letters
            }
        },
        1 => {
            // Check if it's a decimal like "9.1" or just "9."
            if numbering_part.ends_with('.') {
                "H1".to_string()  // "9."
            } else {
                "H2".to_string()  // "9.1"
            }
        },
        2 => "H3".to_string(),  // "9.1.1"
        3 => "H4".to_string(),  // "9.1.1.1"
        _ => "H4".to_string(),  // Deeper nesting
    }
}

pub fn clean_heading_text(text: &str) -> String {
    let text = text.trim();
    
   
    let mut cleaned = if text.ends_with(':') {
        text[..text.len()-1].trim().to_string()
    } else {
        text.to_string()
    };

    let page_number_regex = Regex::new(r"\s+\d{1,3}$").unwrap();
    cleaned = page_number_regex.replace(&cleaned, "").to_string();
    
    let dotted_leaders_regex = Regex::new(r"\s*\.{3,}\s*\d*$").unwrap();
    cleaned = dotted_leaders_regex.replace(&cleaned, "").to_string();
    
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}
