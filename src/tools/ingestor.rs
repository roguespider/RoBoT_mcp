// src/tools/ingestor.rs
// File ingestion tool for importing documents into short-term memory

use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tar::Archive;
use uuid::Uuid;
use zip::ZipArchive;

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::tools::ToolOutput;

/// Default folder name for files to import
pub const DEFAULT_IMPORT_FOLDER: &str = "files_to_import";

/// Default chunk size for text splitting
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Default overlap between chunks
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

/// Supported file extensions
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rst", "csv", "log", "xml", "html", "htm",  // Standard text
    "rs", "toml", "yaml", "yml", "env", "gitignore", "dockerfile",  // Code & config
    "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php",  // Code
    "sql", "sh", "bash", "zsh", "ps1", "bat", "cmd",  // Scripts
    "css", "scss", "sass", "less", "json", "jsonl",  // Web & data
    "properties", "conf", "cfg", "ini", "lock",  // Config
    "srt", "vtt", "ass",  // Subtitles
];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "tar", "gz", "tgz", "tar.gz", "bz2", "xz", "7z", "rar"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "flac", "ogg", "aac", "wma", "opus"];
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "ico", "tiff", "svg"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg"];
const DOC_EXTENSIONS: &[&str] = &["pdf", "doc", "docx", "odt", "rtf", "epub"];

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IngestFilesInput {
    /// Path to the folder containing files to import (defaults to 'files_to_import')
    pub folder: Option<String>,
    /// Ingest a specific file by path (ingests only this file)
    pub file_path: Option<String>,
    /// Maximum number of files to process (ignored if file_path is set)
    pub limit: Option<usize>,
    /// Chunk size for text splitting
    pub chunk_size: Option<usize>,
    /// Memory type to use for ingested content
    pub memory_type: Option<String>,
}

/// Tool: List files ready for import
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImportableInput {
    /// Path to the folder to check (defaults to 'files_to_import')
    pub folder: Option<String>,
    /// Maximum number of files to return (default: 5)
    pub limit: Option<usize>,
}

/// Tool: Transcribe an audio file
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TranscribeAudioInput {
    /// Path to the audio file
    pub path: String,
    /// Output path for the transcription JSON file
    pub output: Option<String>,
}

/// Tool: Delete successfully imported files (requires confirmation)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteIngestedFilesInput {
    /// List of file paths to delete (from successful ingestion response)
    pub files: Vec<String>,
    /// Must be "yes" or "confirm" to actually delete files
    pub confirmation: String,
}

/// Tool: List files that were successfully ingested and can be deleted
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListIngestedFilesInput {
    /// Only show files from this import folder
    pub folder: Option<String>,
    /// Maximum number of files to return
    pub limit: Option<usize>,
}

/// Result of ingesting a single file
#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub filename: String,
    pub file_path: String,
    pub success: bool,
    pub chunks_created: usize,
    pub memory_ids: Vec<String>,
    pub error: Option<String>,
}

/// Summary of ingestion operation
#[derive(Debug, Clone, Serialize)]
pub struct IngestSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_chunks: usize,
    pub results: Vec<IngestResult>,
}

/// File info for importable files
#[derive(Debug, Clone, Serialize)]
pub struct ImportableFile {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub file_type: String,
}

/// Tool definitions
pub mod definitions {
    pub const INGEST_FILES: &str = "ingest_files";
    pub const LIST_IMPORTABLE: &str = "list_importable";
    pub const TRANSCRIBE_AUDIO: &str = "transcribe_audio";
    pub const LIST_INGESTED_FILES: &str = "list_ingested_files";
    pub const DELETE_INGESTED_FILES: &str = "delete_ingested_files";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: INGEST_FILES.to_string(),
                description: "Ingest files into short-term memory. Use file_path to ingest ONE specific file. Without file_path, use folder+limit to ingest multiple.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder containing files. Relative to exe location, or absolute path. Defaults to 'files_to_import'"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "INGEST ONE FILE - path to specific file to ingest (full path or relative to folder)"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Max files to ingest if no file_path specified (default: 1)"
                        },
                        "chunk_size": {
                            "type": "number",
                            "description": "Size of text chunks (default: 1000, use 500 for large files)"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Memory type: note, file, conversation, code",
                            "enum": ["note", "file", "conversation", "code"]
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_IMPORTABLE.to_string(),
                description: "List files ready for import. Returns one file at a time by default.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder. Relative to exe location, or absolute path. Defaults to 'files_to_import'"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Max files to return (default: 1)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: TRANSCRIBE_AUDIO.to_string(),
                description: "Transcribe an audio file to text (requires whisper or similar)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the audio file"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output path for transcription JSON file"
                        }
                    },
                    "required": ["path"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_INGESTED_FILES.to_string(),
                description: "List files that have been successfully ingested and are safe to delete".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Only show files from this import folder"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of files to return"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: DELETE_INGESTED_FILES.to_string(),
                description: "Delete files that were successfully ingested. REQUIRES confirmation - set confirmation to 'yes' to proceed with deletion.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of file paths to delete"
                        },
                        "confirmation": {
                            "type": "string",
                            "description": "Must be 'yes' or 'confirm' to delete files. Any other value will simulate deletion and show what would be deleted."
                        }
                    },
                    "required": ["files", "confirmation"]
                }),
            },
        ]
    }
}

/// Detect file type from extension
fn detect_file_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        "text".to_string()
    } else if ARCHIVE_EXTENSIONS.contains(&ext.as_str()) {
        "archive".to_string()
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        "audio".to_string()
    } else if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        "image".to_string()
    } else if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        "video".to_string()
    } else if DOC_EXTENSIONS.contains(&ext.as_str()) {
        "document".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Extract text content from a file
fn extract_text(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // All text/code extensions
    if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        let content = fs::read_to_string(path).context("Failed to read file")?;
        // Pretty print JSON/YAML for better searchability
        if ext == "json" || ext == "jsonl" {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(v) => return Ok(serde_json::to_string_pretty(&v).unwrap_or(content)),
                Err(_) => {}
            }
        }
        return Ok(content);
    }
    
    // Document formats
    if DOC_EXTENSIONS.contains(&ext.as_str()) {
        return match ext.as_str() {
            "pdf" => extract_pdf_text(path),
            "docx" => extract_docx_text(path),
            "epub" => extract_epub_text(path),
            _ => Err(anyhow::anyhow!("Document type '{}' requires external tool for extraction", ext)),
        };
    }
    
    // Image - needs OCR (note in output)
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(format!(
            "[IMAGE FILE: {}]\n\
            File: {}\n\
            Size: {} bytes\n\
            \n\
            NOTE: This is an image file. For full text extraction, use OCR tools like Tesseract.\n\
            The agent can analyze this image if vision capabilities are available.\n",
            ext,
            path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        ));
    }
    
    // Video - needs transcription
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(format!(
            "[VIDEO FILE: {}]\n\
            File: {}\n\
            Size: {} bytes\n\
            \n\
            NOTE: This is a video file. For full text extraction, use transcription tools.\n\
            Video content cannot be directly ingested without audio extraction.\n",
            ext,
            path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        ));
    }
    
    // Audio - needs transcription  
    if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(format!(
            "[AUDIO FILE: {}]\n\
            File: {}\n\
            Size: {} bytes\n\
            \n\
            NOTE: This is an audio file. Use whisper or similar for transcription.\n\
            Audio content cannot be directly ingested as text.\n",
            ext,
            path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        ));
    }
    
    Err(anyhow::anyhow!("Unsupported file type: {}", ext))
}

/// Extract text from PDF file (basic implementation)
fn extract_pdf_text(path: &Path) -> Result<String> {
    let file = File::open(path).context("Failed to open PDF file")?;
    let mut reader = BufReader::new(file);
    
    // Simple PDF text extraction - reads raw PDF content
    // For production, use pdf-extract or lopdf crate
    let mut content = String::new();
    let mut buffer = [0u8; 8192];
    
    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        // Extract printable ASCII characters from PDF
        for &byte in &buffer[..n] {
            if (32..=126).contains(&byte) || byte == b'\n' || byte == b'\r' || byte == b'\t' {
                content.push(byte as char);
            }
        }
    }
    
    // Clean up the extracted content
    let cleaned: String = content
        .lines()
        .filter(|line| {
            !line.is_empty() && 
            !line.starts_with('%') && 
            !line.starts_with("<<") &&
            !line.starts_with(">>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    if cleaned.trim().is_empty() {
        Err(anyhow::anyhow!("No readable text found in PDF"))
    } else {
        Ok(cleaned)
    }
}

/// Extract text from DOCX file (ZIP-based extraction)
fn extract_docx_text(path: &Path) -> Result<String> {
    let file = File::open(path).context("Failed to open DOCX file")?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    
    let mut content = String::new();
    
    // DOCX files have document.xml in word/ folder
    if let Ok(mut xml_file) = archive.by_name("word/document.xml") {
        let mut xml_content = String::new();
        xml_file.read_to_string(&mut xml_content)?;
        
        // Simple XML text extraction - strip tags
        content = strip_xml_tags(&xml_content);
    }
    
    if content.trim().is_empty() {
        Err(anyhow::anyhow!("No readable text found in DOCX file"))
    } else {
        Ok(content)
    }
}

/// Extract text from EPUB file (ZIP-based extraction)
fn extract_epub_text(path: &Path) -> Result<String> {
    let file = File::open(path).context("Failed to open EPUB file")?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    
    let mut all_content = String::new();
    
    // EPUB files have XHTML content in OEBPS/content folder
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().to_string();
            // Look for .xhtml, .html, .htm files
            if name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm") {
                let mut html_content = String::new();
                file.read_to_string(&mut html_content)?;
                
                // Strip HTML tags
                let text = strip_html_tags(&html_content);
                if !text.trim().is_empty() {
                    all_content.push_str(&text);
                    all_content.push_str("\n\n");
                }
            }
        }
    }
    
    if all_content.trim().is_empty() {
        Err(anyhow::anyhow!("No readable text found in EPUB file"))
    } else {
        Ok(all_content)
    }
}

/// Strip XML/HTML tags from content
fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    // Simple XML parser to extract text content
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !result.ends_with(' ') && !result.is_empty() {
                    result.push(' ');
                }
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip HTML tags from content
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Chunk text into smaller pieces with overlap
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + chunk_size).min(text.len());
        let chunk = text[start..end].to_string();
        chunks.push(chunk);

        if end >= text.len() {
            break;
        }
        start = end - overlap.min(end);
    }

    chunks
}

/// Process an archive file and return contained files
fn process_archive(archive_path: &Path, temp_dir: &Path) -> Result<Vec<PathBuf>> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut extracted_files = Vec::new();

    match ext.as_str() {
        "zip" => {
            let file = File::open(archive_path).context("Failed to open zip file")?;
            let mut archive = ZipArchive::new(BufReader::new(file))?;
            
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = match file.enclosed_name() {
                    Some(path) => temp_dir.join(path),
                    None => continue,
                };

                if file.is_dir() {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut outfile = File::create(&outpath)?;
                    io::copy(&mut file, &mut outfile)?;
                    extracted_files.push(outpath);
                }
            }
        }
        "tar" => {
            let file = File::open(archive_path).context("Failed to open tar file")?;
            let mut archive = Archive::new(file);
            
            for entry in archive.entries()? {
                let mut entry = entry?;
                let outpath = temp_dir.join(entry.path()?);
                
                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    entry.unpack(&outpath)?;
                    extracted_files.push(outpath);
                }
            }
        }
        "gz" | "tgz" => {
            // Handle .gz and .tar.gz files
            if archive_path.to_string_lossy().ends_with(".tar.gz") || ext == "tgz" {
                let file = File::open(archive_path).context("Failed to open tar.gz file")?;
                let mut archive = Archive::new(GzDecoder::new(file));
                
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let outpath = temp_dir.join(entry.path()?);
                    
                    if entry.header().entry_type().is_dir() {
                        fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(parent) = outpath.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        entry.unpack(&outpath)?;
                        extracted_files.push(outpath);
                    }
                }
            } else {
                // Single gzipped file
                let file = File::open(archive_path)?;
                let mut decoder = GzDecoder::new(file);
                let mut content = String::new();
                decoder.read_to_string(&mut content)?;
                
                let outpath = temp_dir.join(
                    archive_path.file_stem().unwrap_or_default().to_string_lossy().as_ref()
                );
                fs::write(&outpath, content)?;
                extracted_files.push(outpath);
            }
        }
        _ => return Err(anyhow::anyhow!("Unsupported archive format: {}", ext)),
    }

    Ok(extracted_files)
}

/// Extract text from audio file using whisper CLI or return structured info
fn extract_audio_text(path: &Path, output_path: Option<&Path>) -> Result<String> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let path_str = path.to_string_lossy();
    
    // Check for whisper CLI in PATH
    let whisper_path = std::env::var("WHISPER_PATH").ok();
    let model_path = std::env::var("WHISPER_MODEL_PATH").ok();
    
    if let Some(whisper) = whisper_path {
        // Use whisper.cpp CLI for transcription
        let model = model_path.unwrap_or_else(|| "models/ggml-base.en.bin".to_string());
        let output_file = output_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{}.txt", path_str));
        
        let mut cmd = std::process::Command::new(&whisper);
        cmd.args(["-m", &model, "-f", &path_str, "-o", "txt", "--output-file"]);
        cmd.arg(&output_file);
        
        tracing::info!("Running whisper: {:?} -m {} -f {}", whisper, model, path_str);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let transcription = fs::read_to_string(&output_file)
                        .unwrap_or_else(|_| "[Transcription file not found]".to_string());
                    return Ok(transcription);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("Whisper transcription failed: {}", stderr);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to run whisper: {}", e);
            }
        }
    }
    
    // Fallback: Try to use ffprobe to extract audio info
    let audio_info = get_audio_file_info(path)?;
    
    if let Some(output) = output_path {
        let transcription = serde_json::json!({
            "file": path_str,
            "filename": filename,
            "status": "requires_transcription",
            "audio_info": audio_info,
            "configuration": {
                "WHISPER_PATH": "Set to whisper CLI path",
                "WHISPER_MODEL_PATH": "Set to model path"
            },
            "message": "Set WHISPER_PATH environment variable to enable transcription"
        });
        
        let json = serde_json::to_string_pretty(&transcription)?;
        fs::write(output, json)?;
    }
    
    Ok(format!(
        "[Audio file: {}]\n\
        Duration: {}s, Format: {}, Codec: {}\n\
        Set WHISPER_PATH env var to enable transcription.",
        filename,
        audio_info.get("duration").unwrap_or(&"unknown".to_string()),
        audio_info.get("format").unwrap_or(&"unknown".to_string()),
        audio_info.get("codec").unwrap_or(&"unknown".to_string())
    ))
}

/// Get basic audio file info using ffprobe
fn get_audio_file_info(path: &Path) -> Result<std::collections::HashMap<String, String>> {
    let mut info = std::collections::HashMap::new();
    
    if let Ok(output) = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path.to_string_lossy().as_ref())
        .output()
        && output.status.success()
            && let Ok(json_str) = String::from_utf8(output.stdout)
                && let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(format) = json.get("format") {
                        if let Some(duration) = format.get("duration").and_then(|v| v.as_str()) {
                            info.insert("duration".to_string(), format!("{:.1}", duration.parse::<f64>().unwrap_or(0.0)));
                        }
                        if let Some(format_name) = format.get("format_name").and_then(|v| v.as_str()) {
                            info.insert("format".to_string(), format_name.to_string());
                        }
                    }
                    if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
                        for stream in streams {
                            if stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio")
                                && let Some(codec) = stream.get("codec_name").and_then(|v| v.as_str()) {
                                    info.insert("codec".to_string(), codec.to_string());
                                }
                        }
                    }
                }
    
    info.entry("duration".to_string()).or_insert_with(|| "unknown".to_string());
    info.entry("format".to_string()).or_insert_with(|| "audio".to_string());
    info.entry("codec".to_string()).or_insert_with(|| "unknown".to_string());
    
    Ok(info)
}

/// Get system temp directory for archive extraction
fn get_archive_temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("robot_brain_{}", std::process::id()))
}

/// Public wrapper that handles all file types including archives
pub fn ingest_file(
    path: &Path,
    database: &Arc<SqliteDatabase>,
    chunk_size: usize,
    overlap: usize,
    memory_type: &str,
) -> Result<IngestResult> {
    let file_type = detect_file_type(path);

    // Handle archives by extracting and ingesting contents
    if file_type == "archive" {
        ingest_archive(path, database, chunk_size, overlap, memory_type)
    } else {
        ingest_single_file(path, database, chunk_size, overlap, memory_type)
    }
}

/// Ingest an archive file - extracts and ingests contents one by one
fn ingest_archive(
    path: &Path,
    database: &Arc<SqliteDatabase>,
    chunk_size: usize,
    overlap: usize,
    memory_type: &str,
) -> Result<IngestResult> {
    let archive_path = path.to_path_buf();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Create temp directory for extraction
    let temp_dir = get_archive_temp_dir();
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return Err(anyhow::anyhow!("Failed to create temp directory: {}", e));
    }

    // Extract archive
    let extracted_files = process_archive(&archive_path, &temp_dir)?;
    
    if extracted_files.is_empty() {
        return Err(anyhow::anyhow!("Archive is empty or contains no readable files"));
    }

    // Ingest only the first file from archive (to avoid token overflow)
    let first_file = &extracted_files[0];
    
    // Recursively ingest the extracted file
    let result = ingest_single_file(first_file, database, chunk_size, overlap, memory_type)?;
    
    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    // Delete the ZIP file after successful ingestion
    if result.success {
        if let Err(e) = fs::remove_file(&archive_path) {
            tracing::warn!("Failed to delete archive {:?}: {}", archive_path, e);
        }
    }

    Ok(IngestResult {
        filename: format!("{}/{}", filename, result.filename),
        file_path: path.to_string_lossy().to_string(),
        success: result.success,
        chunks_created: result.chunks_created,
        memory_ids: result.memory_ids,
        error: result.error,
    })
}

/// Ingest a single file into memory (no archive handling)
fn ingest_single_file(
    path: &Path,
    database: &Arc<SqliteDatabase>,
    chunk_size: usize,
    overlap: usize,
    memory_type: &str,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_type = detect_file_type(path);

    // Extract text content
    let text = match file_type.as_str() {
        "audio" => extract_audio_text(path, None)?,
        _ => extract_text(path)?,
    };

    // Chunk the text
    let chunks = chunk_text(&text, chunk_size, overlap);
    
    // Store chunks in memory
    let mut memory_ids = Vec::new();
    let conn = database.connection()?;
    
    for (i, chunk) in chunks.iter().enumerate() {
        let memory = MemoryCard {
            id: Uuid::new_v4(),
            content: format!(
                "[File: {}] [Chunk {}/{}]\n\n{}",
                filename,
                i + 1,
                chunks.len(),
                chunk
            ),
            memory_type: parse_memory_type(memory_type),
            confidence: 0.9, // Ingested files get high confidence
            importance: 0.7,  // Default importance
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        queries::insert_memory(&conn, &memory)?;
        memory_ids.push(memory.id.to_string());
    }

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: chunks.len(),
        memory_ids,
        error: None,
    })
}

fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "note" => MemoryType::Note,
        _ => MemoryType::File,
    }
}

/// Get the import folder path
fn get_import_folder(folder: Option<&str>) -> PathBuf {
    match folder {
        Some(f) => {
            let path = PathBuf::from(f);
            // If path is absolute or exe_dir exists, use it
            if path.is_absolute() {
                path
            } else if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
                let absolute_path = exe_dir.join(&path);
                if absolute_path.exists() {
                    absolute_path
                } else {
                    // Fall back to current dir
                    path
                }
            } else {
                path
            }
        }
        None => {
            // Default to files_to_import in exe directory
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join(DEFAULT_IMPORT_FOLDER)))
                .unwrap_or_else(|| PathBuf::from(DEFAULT_IMPORT_FOLDER))
        }
    }
}

/// Collect all files from import folder (including from archives)
fn collect_importable_files(folder: &Path) -> Result<Vec<ImportableFile>> {
    let mut files = Vec::new();

    if !folder.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();

        // Skip temp directories
        if path.to_string_lossy().contains("robot_brain_") {
            continue;
        }

        if path.is_file() {
            let file_type = detect_file_type(&path);

            // For archives, just list the archive itself - user ingests the archive directly
            // The ingest process will extract it
            let metadata = fs::metadata(&path)?;
            files.push(ImportableFile {
                path: path.to_string_lossy().to_string(),
                filename: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                size: metadata.len(),
                file_type,
            });
        }
    }

    Ok(files)
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Execute ingest files tool
pub async fn execute_ingest_files(
    input: IngestFilesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let chunk_size = input.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    let overlap = DEFAULT_CHUNK_OVERLAP;
    let memory_type = input.memory_type.unwrap_or_else(|| "file".to_string());

    tracing::info!("Starting file ingestion from {:?}", folder);

    let mut results = Vec::new();
    let mut total_chunks = 0;
    let mut successful = 0;
    let mut failed = 0;

    // If file_path is specified, ingest ONLY that one file
    if let Some(ref file_path) = input.file_path {
        let path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            folder.join(file_path)
        };

        if !path.exists() {
            return Ok(ToolOutput::success(serde_json::json!({
                "success": false,
                "error": format!("File not found: {:?}", path),
                "summary": {
                    "total_files": 1,
                    "successful": 0,
                    "failed": 1,
                    "total_chunks": 0
                }
            })));
        }

        match ingest_file(&path, database, chunk_size, overlap, &memory_type) {
            Ok(result) => {
                total_chunks += result.chunks_created;
                successful += 1;
                results.push(result);
            }
            Err(e) => {
                failed += 1;
                results.push(IngestResult {
                    filename: path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string(),
                    file_path: path.to_string_lossy().to_string(),
                    success: false,
                    chunks_created: 0,
                    memory_ids: vec![],
                    error: Some(e.to_string()),
                });
            }
        }
    } else {
        // Collect files from folder with limit (default 1 for safety)
        let files = collect_importable_files(&folder)?;
        let limit = input.limit.unwrap_or(1);
        let files_to_process: Vec<_> = files.into_iter().take(limit).collect();

        for file_info in files_to_process {
            let path = Path::new(&file_info.path);

            match ingest_file(path, database, chunk_size, overlap, &memory_type) {
                Ok(result) => {
                    total_chunks += result.chunks_created;
                    successful += 1;
                    results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    results.push(IngestResult {
                        filename: file_info.filename.clone(),
                        file_path: file_info.path.clone(),
                        success: false,
                        chunks_created: 0,
                        memory_ids: vec![],
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    let total_files = results.len();

    // Collect successfully ingested file paths for deletion
    let successfully_ingested: Vec<String> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.file_path.clone())
        .collect();

    let summary = IngestSummary {
        total_files,
        successful,
        failed,
        total_chunks,
        results,
    };

    Ok(ToolOutput::success(serde_json::json!({
        "summary": summary,
        "successfully_ingested": successfully_ingested,
        "note": "ZIP/archive files are automatically deleted after successful ingestion",
        "files_to_delete": successfully_ingested.iter().filter(|f| !f.contains(".zip") && !f.ends_with(".tar") && !f.ends_with(".gz")).collect::<Vec<_>>(),
        "next_action": "Use delete_ingested_files for remaining regular files"
    })))
}

/// Execute list importable tool
pub async fn execute_list_importable(
    input: ListImportableInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    
    let files = collect_importable_files(&folder)?;
    let total_count = files.len();
    
    // Limit files returned to avoid token overflow (default 1)
    let limit = input.limit.unwrap_or(1);
    let limited_files: Vec<_> = files.into_iter().take(limit).collect();
    let returned_count = limited_files.len();
    
    // Get first file path for easy ingestion
    let first_file = limited_files.first().map(|f| &f.path);
    
    Ok(ToolOutput::success(serde_json::json!({
        "folder": folder.to_string_lossy(),
        "total_files": total_count,
        "returned_files": returned_count,
        "has_more": total_count > returned_count,
        "files": limited_files,
        "next_file_path": first_file,
        "workflow_tip": if total_count > returned_count {
            format!("Showing {} of {} files. Use ingest_files with file_path parameter to ingest a specific file, or use limit to see more.", returned_count, total_count)
        } else {
            format!("All {} files listed. Use ingest_files with file_path to ingest one at a time.", total_count)
        }
    })))
}

/// Execute transcribe audio tool
pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
) -> Result<ToolOutput> {
    let audio_path = Path::new(&input.path);
    
    if !audio_path.exists() {
        return Err(anyhow::anyhow!("Audio file not found: {}", input.path));
    }
    
    let file_type = detect_file_type(audio_path);
    if file_type != "audio" {
        return Err(anyhow::anyhow!("Not an audio file: {}", input.path));
    }
    
    let output_path = input.output
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("transcription.json"));
    
    let text = extract_audio_text(audio_path, Some(&output_path))?;
    
    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "audio_file": input.path,
        "transcription_file": output_path.to_string_lossy(),
        "transcription": text,
        "note": "Whisper CLI integration enabled. Set WHISPER_PATH env var or local model for actual transcription."
    })))
}

/// Execute list ingested files tool
pub async fn execute_list_ingested_files(
    input: ListIngestedFilesInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(100);
    
    // Collect files from the import folder
    let files = collect_importable_files(&folder)?;
    
    // Filter and limit
    let files_to_show: Vec<_> = files
        .into_iter()
        .take(limit)
        .collect();
    
    let total_count = files_to_show.len();
    
    // Format for display
    let file_paths: Vec<String> = files_to_show
        .iter()
        .map(|f| f.path.clone())
        .collect();
    
    Ok(ToolOutput::success(serde_json::json!({
        "folder": folder.to_string_lossy(),
        "count": total_count,
        "files": file_paths,
        "deletion_command": {
            "tool": "delete_ingested_files",
            "example": {
                "files": file_paths,
                "confirmation": "yes"
            }
        }
    })))
}

/// Execute delete ingested files tool
pub async fn execute_delete_ingested_files(
    input: DeleteIngestedFilesInput,
) -> Result<ToolOutput> {
    let confirmed = input.confirmation.to_lowercase() == "yes" 
        || input.confirmation.to_lowercase() == "confirm";
    
    if input.files.is_empty() {
        return Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "message": "No files specified for deletion",
            "deleted": Vec::<()>::new(),
            "failed": Vec::<()>::new()
        })));
    }
    
    if !confirmed {
        // Simulation mode - show what would be deleted
        let mut would_delete = Vec::new();
        for file_path in &input.files {
            let path = Path::new(file_path);
            if path.exists() {
                would_delete.push(serde_json::json!({
                    "path": file_path,
                    "status": "would_delete",
                    "exists": true
                }));
            } else {
                would_delete.push(serde_json::json!({
                    "path": file_path,
                    "status": "not_found",
                    "exists": false
                }));
            }
        }
        
        return Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "simulation": true,
            "message": "This is a SIMULATION. Files were NOT deleted.",
            "confirmation_required": "Set 'confirmation' to 'yes' or 'confirm' to actually delete",
            "would_delete": would_delete,
            "deleted": Vec::<String>::new(),
            "failed": Vec::<()>::new()
        })));
    }
    
    // Actually delete the files
    let mut deleted = Vec::new();
    let mut failed = Vec::new();
    
    for file_path in &input.files {
        let path = Path::new(file_path);
        
        if !path.exists() {
            failed.push(serde_json::json!({
                "path": file_path,
                "error": "File not found"
            }));
            continue;
        }
        
        match fs::remove_file(path) {
            Ok(()) => {
                tracing::info!("Deleted file: {:?}", path);
                deleted.push(file_path.clone());
            }
            Err(e) => {
                tracing::warn!("Failed to delete {:?}: {}", path, e);
                failed.push(serde_json::json!({
                    "path": file_path,
                    "error": e.to_string()
                }));
            }
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "success": failed.is_empty(),
        "simulation": false,
        "deleted": deleted,
        "failed": failed,
        "summary": {
            "total_requested": input.files.len(),
            "deleted_count": deleted.len(),
            "failed_count": failed.len()
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type(Path::new("test.txt")), "text");
        assert_eq!(detect_file_type(Path::new("test.json")), "json");
        assert_eq!(detect_file_type(Path::new("test.pdf")), "pdf");
        assert_eq!(detect_file_type(Path::new("test.zip")), "archive");
        assert_eq!(detect_file_type(Path::new("test.mp3")), "audio");
        assert_eq!(detect_file_type(Path::new("test.unknown")), "unknown");
    }

    #[test]
    fn test_chunk_text() {
        let text = "Hello world! This is a test. ".repeat(100);
        let chunks = chunk_text(&text, 50, 10);
        
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= 50 || c.len() < 50));
    }

    #[test]
    fn test_chunk_small_text() {
        let text = "Short text";
        let chunks = chunk_text(text, 1000, 100);
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
