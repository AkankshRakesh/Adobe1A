use lopdf::{Document, Object, content::Content};

#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub size: f64,
    pub page: usize,
}

// Extract text runs with their font size from a PDF. Works with simple encodings; if decoding
// fails we fall back to raw bytes.
pub fn extract_runs(doc: &Document) -> Vec<TextRun> {
    let mut runs = Vec::new();

    for (page_idx, (&_page_no, &page_id)) in doc.get_pages().iter().enumerate() {
        let current_page = page_idx + 1;

        // Get the page content stream and decode operations
        if let Ok(content_data) = doc.get_page_content(page_id) {
            if let Ok(content) = Content::decode(&content_data) {
                let mut cur_font_size = 12.0_f64;

                for op in content.operations {
                    match op.operator.as_ref() {
                        "Tf" => {
                            // "Tf" has operands: font-name, font-size
                            if op.operands.len() == 2 {
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
                                        runs.push(TextRun { text, size: cur_font_size, page: current_page });
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
                                        runs.push(TextRun { text: combined, size: cur_font_size, page: current_page });
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
