use std::io::Read;
use std::path::Path;
use tracing::debug;

/// Extract text from a .docx file.
///
/// A .docx is a ZIP archive containing XML. We extract text from
/// `word/document.xml` by stripping XML tags and collecting text nodes.
pub fn extract_text(file_path: &Path) -> Result<ExtractedWord, anyhow::Error> {
    debug!("Extracting text from Word: {}", file_path.display());

    let file = std::fs::File::open(file_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| anyhow::anyhow!("Not a valid DOCX/ZIP: {e}"))?;

    let mut xml_content = String::new();

    // Read the main document XML
    let mut doc_entry = archive
        .by_name("word/document.xml")
        .map_err(|_| anyhow::anyhow!("No word/document.xml found — not a valid DOCX"))?;

    doc_entry.read_to_string(&mut xml_content)?;

    let text = extract_text_from_xml(&xml_content);

    Ok(ExtractedWord {
        text: text.trim().to_string(),
    })
}

pub struct ExtractedWord {
    pub text: String,
}

/// Simple XML text extractor for Word document XML.
///
/// Extracts text content from `<w:t>` tags and converts `<w:p>` paragraph
/// boundaries into newlines. This is deliberately simple — handles the
/// common case without pulling in a full XML parser.
fn extract_text_from_xml(xml: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut collecting_text = false;
    let mut chars = xml.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
            }
            '>' if in_tag => {
                in_tag = false;

                // Check for paragraph end — add newline
                if tag_name.starts_with("/w:p") {
                    result.push('\n');
                }

                // <w:t> or <w:t ...> starts text collection
                if tag_name == "w:t" || tag_name.starts_with("w:t ") {
                    collecting_text = true;
                }
                // </w:t> ends text collection
                if tag_name == "/w:t" {
                    collecting_text = false;
                }

                tag_name.clear();
            }
            _ if in_tag => {
                tag_name.push(ch);
            }
            _ if collecting_text => {
                result.push(ch);
            }
            _ => {}
        }
    }

    // Clean up: collapse multiple blank lines
    let lines: Vec<&str> = result.lines().collect();
    let mut cleaned = String::new();
    let mut prev_empty = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_empty {
                cleaned.push('\n');
            }
            prev_empty = true;
        } else {
            cleaned.push_str(trimmed);
            cleaned.push('\n');
            prev_empty = false;
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_from_simple_xml() {
        let xml = r#"<w:body><w:p><w:r><w:t>Hello</w:t></w:r><w:r><w:t xml:space="preserve"> World</w:t></w:r></w:p><w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p></w:body>"#;
        let text = extract_text_from_xml(xml);
        assert!(text.contains("Hello World"));
        assert!(text.contains("Second paragraph"));
    }
}
