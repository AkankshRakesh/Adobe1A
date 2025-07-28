use lopdf::{Document, Object, content::Content};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub size: f64,
    pub page: usize,
    pub font_name: String,
    pub is_bold: bool,
    pub is_italic: bool,
}

#[derive(Debug, Clone)]
pub struct HeadingCandidate {
    pub text: String,
    pub level: String,
    pub page: usize,
    pub confidence: f64,
}

// Extract text runs with their font size and style from a PDF
pub fn extract_runs(doc: &Document) -> Vec<TextRun> {
    let mut runs = Vec::new();

    for (page_idx, (&_page_no, &page_id)) in doc.get_pages().iter().enumerate() {
        let current_page = page_idx + 1;

        // Get the page content stream and decode operations
        if let Ok(content_data) = doc.get_page_content(page_id) {
            if let Ok(content) = Content::decode(&content_data) {
                let mut cur_font_size = 12.0_f64;
                let mut cur_font_name = String::new();

                for op in content.operations {
                    match op.operator.as_ref() {
                        "Tf" => {
                            // "Tf" has operands: font-name, font-size
                            if op.operands.len() == 2 {
                                // Extract font name
                                if let Object::Name(name) = &op.operands[0] {
                                    cur_font_name = String::from_utf8_lossy(name).to_string();
                                }
                                
                                // Extract font size
                                let sz_opt = match &op.operands[1] {
                                    Object::Real(r) => Some(*r as f64),
                                    Object::Integer(i) => Some(*i as f64),
                                    _ => None,
                                };
                                if let Some(sz) = sz_opt {
                                    cur_font_size = sz;
                                }
                            }
                        }
                        "Tj" => {
                            // Single string operand
                            if let Some(text_obj) = op.operands.get(0) {
                                if let Some(text) = try_decode_text(text_obj, doc) {
                                    if !text.trim().is_empty() {
                                        let (is_bold, is_italic) = analyze_font_style(&cur_font_name);
                                        runs.push(TextRun { 
                                            text, 
                                            size: cur_font_size, 
                                            page: current_page,
                                            font_name: cur_font_name.clone(),
                                            is_bold,
                                            is_italic,
                                        });
                                    }
                                }
                            }
                        }
                        "TJ" => {
                            // Array of strings and numbers
                            if let Some(text_obj) = op.operands.get(0) {
                                if let Object::Array(items) = text_obj {
                                    let mut combined = String::new();
                                    for item in items {
                                        if let Some(s) = try_decode_text(item, doc) {
                                            combined.push_str(&s);
                                        }
                                    }
                                    if !combined.trim().is_empty() {
                                        let (is_bold, is_italic) = analyze_font_style(&cur_font_name);
                                        runs.push(TextRun { 
                                            text: combined, 
                                            size: cur_font_size, 
                                            page: current_page,
                                            font_name: cur_font_name.clone(),
                                            is_bold,
                                            is_italic,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    runs
}

fn try_decode_text(obj: &Object, _doc: &Document) -> Option<String> {
    match obj {
        Object::String(bytes, _) => {
            Some(String::from_utf8_lossy(&bytes).to_string())
        }
        _ => None,
    }
}

// Analyze font style based on font name
fn analyze_font_style(font_name: &str) -> (bool, bool) {
    let font_lower = font_name.to_lowercase();
    
    let is_bold = font_lower.contains("bold") || 
                  font_lower.contains("black") || 
                  font_lower.contains("heavy") ||
                  font_lower.contains("extrabold") ||
                  font_lower.contains("semibold");
    
    let is_italic = font_lower.contains("italic") || 
                    font_lower.contains("oblique");
    
    (is_bold, is_italic)
}

// Classify heading level based on font size and style (similar to Python approach)
pub fn classify_heading(size: f64, is_bold: bool, is_italic: bool) -> (String, f64) {
    let mut confidence: f64 = 0.0;
    let level;

    if size > 15.0 {
        level = "H1".to_string();
        confidence = 0.9;
    } else if size > 12.0 && size <= 15.0 {
        level = "H2".to_string();
        confidence = 0.8;
    } else if size > 10.0 && size <= 12.0 {
        level = "H3".to_string();
        confidence = 0.6;
    } else {
        level = "Body Text".to_string();
        confidence = 0.1;
    }

    // Boost confidence if text is bold or italic
    if is_bold {
        confidence += 0.15;
    }
    if is_italic {
        confidence += 0.05;
    }

    (level, confidence.min(1.0))
}

// Extract heading candidates with confidence scores
pub fn extract_heading_candidates(doc: &Document) -> Vec<HeadingCandidate> {
    let runs = extract_runs(doc);
    let mut candidates = Vec::new();
    
    // Group runs by page and line (approximate)
    let mut page_lines: HashMap<usize, Vec<String>> = HashMap::new();
    let mut page_run_info: HashMap<(usize, String), (f64, bool, bool)> = HashMap::new();
    
    for run in runs {
        let text = run.text.trim();
        if text.len() <= 3 || text.len() > 150 { // Better length filtering like Python
            continue;
        }
        
        page_lines.entry(run.page).or_insert_with(Vec::new).push(text.to_string());
        page_run_info.insert((run.page, text.to_string()), (run.size, run.is_bold, run.is_italic));
    }
    
    for (page_num, lines) in page_lines {
        for line in lines {
            if let Some((size, is_bold, is_italic)) = page_run_info.get(&(page_num, line.clone())) {
                let (level, confidence) = classify_heading(*size, *is_bold, *is_italic);
                
                if confidence > 0.5 && is_good_heading_candidate(&line) {
                    candidates.push(HeadingCandidate {
                        text: line,
                        level,
                        page: page_num,
                        confidence,
                    });
                }
            }
        }
    }
    
    candidates
}

// Additional validation for heading candidates
fn is_good_heading_candidate(text: &str) -> bool {
    let text = text.trim();
    
    // Length constraints similar to Python approach
    if text.len() < 4 || text.len() > 100 {
        return false;
    }
    
    // Skip sentences (typically end with periods and have many words)
    let word_count = text.split_whitespace().count();
    if text.ends_with('.') && word_count > 8 {
        return false;
    }
    
    // Skip very long sentences or prose
    if word_count > 12 {
        return false;
    }
    
    // Skip common non-heading patterns
    let text_lower = text.to_lowercase();
    let prose_indicators = [
        "it is expected", "must be completed", "will be issued", "are expected",
        "the planning process", "specifically", "during", "suitable for",
        "include", "secure the full", "expected that", "approved by"
    ];
    
    for indicator in &prose_indicators {
        if text_lower.contains(indicator) {
            return false;
        }
    }
    
    // Skip text that starts with lowercase (usually continuation of sentences)
    if text.chars().next().map_or(false, |c| c.is_lowercase()) {
        return false;
    }
    
    // Skip numeric lists that are too detailed/long
    if text.starts_with(char::is_numeric) && word_count > 6 {
        return false;
    }
    
    true
}
