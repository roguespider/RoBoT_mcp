// src/tools/ingestor/file_collector.rs
// File collection and import folder management

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Default folder name for files to import
pub const DEFAULT_IMPORT_FOLDER: &str = "files_to_import";

/// Maximum file size for text files (50MB) - larger files may cause timeouts
pub const MAX_TEXT_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Maximum file size for JSON files (10MB) - JSON with embeddings don't chunk well
pub const MAX_JSON_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Supported file extensions
pub const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rst", "csv", "log", "xml", "html", "htm",  // Standard text
    "rs", "toml", "yaml", "yml", "env", "gitignore", "dockerfile",  // Code & config
    "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php",  // Code
    "sql", "sh", "bash", "zsh", "ps1", "bat", "cmd",  // Scripts
    "css", "scss", "sass", "less", "json", "jsonl",  // Web & data
    "properties", "conf", "cfg", "ini", "lock",  // Config
    "srt", "vtt", "ass",  // Subtitles
];

/// JSON file extensions - these have special size limits due to embedding/metadata files
pub const JSON_EXTENSIONS: &[&str] = &["json", "jsonl"];

/// Files that should be skipped (typically embedding/metadata dumps)
pub const SKIP_PATTERNS: &[&str] = &[
    "embeddings",
    "embedding",
    "vector",
    "vectors",
    "chroma",
    "pinecone",
    "qdrant",
    "metadata",
    "index.json",
    "faiss",
    "ann",
    "hnsw",
];

pub const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "tar", "gz", "tgz", "tar.gz", "bz2", "xz", "7z", "rar"];
pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "flac", "ogg", "aac", "wma", "opus"];
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "ico", "tiff", "svg"];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg"];
pub const DOC_EXTENSIONS: &[&str] = &["pdf", "doc", "docx", "odt", "rtf", "epub"];

/// File info for importable files
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportableFile {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub file_type: String,
    pub skip_reason: Option<String>,
}

/// Get the import folder path
pub fn get_import_folder(folder: Option<&str>) -> PathBuf {
    match folder {
        Some(f) => {
            let path = Path::new(f);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                // Relative to executable
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(f)
            }
        }
        None => {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
                .join(DEFAULT_IMPORT_FOLDER)
        }
    }
}

/// Check if a file extension matches any of the supported extensions
pub fn is_supported_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| extensions.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if filename matches skip patterns (embedding/metadata files)
fn matches_skip_pattern(filename: &str) -> Option<String> {
    let filename_lower = filename.to_lowercase();
    for pattern in SKIP_PATTERNS {
        if filename_lower.contains(&pattern.to_lowercase()) {
            return Some(format!("matches skip pattern '{}'", pattern));
        }
    }
    None
}

/// Detect file type based on extension
fn detect_file_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    if is_supported_extension(path, TEXT_EXTENSIONS) {
        "text".to_string()
    } else if is_supported_extension(path, ARCHIVE_EXTENSIONS) {
        "archive".to_string()
    } else if is_supported_extension(path, AUDIO_EXTENSIONS) {
        "audio".to_string()
    } else if is_supported_extension(path, IMAGE_EXTENSIONS) {
        "image".to_string()
    } else if is_supported_extension(path, VIDEO_EXTENSIONS) {
        "video".to_string()
    } else if is_supported_extension(path, DOC_EXTENSIONS) {
        "document".to_string()
    } else {
        format!("unknown({})", ext)
    }
}

/// Check if file should be skipped due to size limits
fn check_file_size_limits(path: &Path, file_type: &str, size: u64) -> Option<String> {
    // Check skip patterns first
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(reason) = matches_skip_pattern(filename) {
            return Some(reason);
        }
    }
    
    // Check JSON-specific size limits
    if is_supported_extension(path, JSON_EXTENSIONS) {
        if size > MAX_JSON_FILE_SIZE {
            return Some(format!(
                "JSON file exceeds {}MB limit (found {}MB). JSON files with embeddings/metadata don't chunk well.",
                MAX_JSON_FILE_SIZE / (1024 * 1024),
                size / (1024 * 1024)
            ));
        }
        return None;
    }
    
    // Check general text file size limits
    if file_type == "text" && size > MAX_TEXT_FILE_SIZE {
        return Some(format!(
            "File exceeds {}MB limit (found {}MB). Try splitting the file.",
            MAX_TEXT_FILE_SIZE / (1024 * 1024),
            size / (1024 * 1024)
        ));
    }
    
    None
}

/// Collect all importable files from a folder (non-recursive)
pub fn collect_importable_files(folder: &Path) -> Result<Vec<ImportableFile>> {
    collect_importable_files_internal(folder, false)
}

/// Collect all importable files from a folder with recursive option
pub fn collect_importable_files_with_recursive(folder: &Path, recursive: bool) -> Result<Vec<ImportableFile>> {
    collect_importable_files_internal(folder, recursive)
}

fn collect_importable_files_internal(folder: &Path, recursive: bool) -> Result<Vec<ImportableFile>> {
    let mut files = Vec::new();
    
    if !folder.exists() {
        return Ok(files);
    }
    
    // Check if folder is actually a single file path
    if folder.is_file() {
        let file_name = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_type = detect_file_type(folder);
        let size = fs::metadata(folder)?.len();
        let skip_reason = check_file_size_limits(folder, &file_type, size);
        
        // Always use canonical/absolute path
        let absolute_path = folder.canonicalize().unwrap_or_else(|_| folder.to_path_buf());
        
        return Ok(vec![ImportableFile {
            path: absolute_path.to_string_lossy().to_string(),
            filename: file_name,
            size,
            file_type,
            skip_reason,
        }]);
    }
    
    // Collect files from folder
    collect_files_recursive(folder, recursive, &mut files)?;
    
    // Sort by filename
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    
    Ok(files)
}

fn collect_files_recursive(dir: &Path, recursive: bool, files: &mut Vec<ImportableFile>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if recursive {
                collect_files_recursive(&path, recursive, files)?;
            }
            continue;
        }
        
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_type = detect_file_type(&path);
        
        // Skip files with unknown type (unsupported)
        if file_type.starts_with("unknown(") {
            continue;
        }
        
        let size = match fs::metadata(&path) {
            Ok(m) => m.len(),
            Err(_) => continue,
        };
        
        let skip_reason = check_file_size_limits(&path, &file_type, size);
        
        // Always use canonical/absolute path to avoid confusion
        let absolute_path = path.canonicalize()
            .unwrap_or_else(|_| path.clone());
        
        files.push(ImportableFile {
            path: absolute_path.to_string_lossy().to_string(),
            filename: file_name,
            size,
            file_type,
            skip_reason,
        });
    }
    Ok(())
}

/// Collect all files recursively from a directory
pub fn collect_all_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !dir.exists() {
        return Ok(files);
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            files.extend(collect_all_files_recursive(&path)?);
        } else {
            files.push(path);
        }
    }
    
    Ok(files)
}
