// src/tools/ingestor/text_extractor.rs
// Text extraction from various file formats

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::Result;

/// Extract text from a file based on its extension
pub fn extract_text(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "pdf" => extract_pdf_text(path),
        "docx" => extract_docx_text(path),
        "epub" => extract_epub_text(path),
        _ => {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
    }
}

/// Extract text from PDF
pub fn extract_pdf_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    
    let text = String::from_utf8_lossy(&bytes);
    let mut result = String::new();
    
    for line in text.lines() {
        if line.contains("BT") || line.contains(" Tj") || line.contains(" TJ") {
            let cleaned = line
                .replace("BT", "")
                .replace("ET", "")
                .replace("(", "")
                .replace(")", "")
                .replace("\\n", "\n")
                .replace("\\t", "\t");
            if !cleaned.trim().is_empty() {
                result.push_str(&cleaned);
                result.push('\n');
            }
        }
    }
    
    if result.trim().is_empty() {
        anyhow::bail!("Could not extract text from PDF - file may be scanned/image-based");
    }
    
    Ok(result)
}

/// Extract text from DOCX
pub fn extract_docx_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut content = String::new();
    
    if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
        let mut xml = String::new();
        doc_file.read_to_string(&mut xml)?;
        content = strip_xml_tags(&xml);
    }
    
    if content.trim().is_empty() {
        anyhow::bail!("Could not extract text from DOCX");
    }
    
    Ok(content)
}

/// Extract text from EPUB
pub fn extract_epub_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = String::new();
    
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm") || name == "content.opf" {
                let mut html = String::new();
                let mut f = file;
                f.read_to_string(&mut html)?;
                content.push_str(&strip_html_tags(&html));
                content.push_str("\n\n");
            }
        }
    }
    
    if content.trim().is_empty() {
        anyhow::bail!("Could not extract text from EPUB");
    }
    
    Ok(content)
}

/// Strip XML/HTML tags and extract text content
pub fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::new();
    let mut in_content = true;
    
    let mut chars = xml.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            in_content = false;
        } else if c == '>' {
            in_content = true;
        } else if in_content {
            result.push(c);
        }
    }
    
    // Clean up whitespace
    let lines: Vec<&str> = result.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    lines.join("\n")
}

/// Strip HTML tags
pub fn strip_html_tags(html: &str) -> String {
    strip_xml_tags(html) // Same logic
}

/// Chunk text into smaller pieces with overlap
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }
    
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;
    
    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        
        if end >= chars.len() {
            break;
        }
        
        start = end - overlap;
    }
    
    chunks
}
