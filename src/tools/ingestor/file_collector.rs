// src/tools/ingestor/file_collector.rs
// File collection and import folder management

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Default folder name for files to import
pub const DEFAULT_IMPORT_FOLDER: &str = "files_to_import";

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
fn is_supported_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| extensions.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
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

/// Collect all importable files from a folder
pub fn collect_importable_files(folder: &Path) -> Result<Vec<ImportableFile>> {
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
        
        return Ok(vec![ImportableFile {
            path: folder.to_string_lossy().to_string(),
            filename: file_name,
            size: fs::metadata(folder)?.len(),
            file_type: detect_file_type(folder),
        }]);
    }
    
    // Collect files from folder
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip directories
        if path.is_dir() {
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
        
        let size = fs::metadata(&path)?.len();
        
        files.push(ImportableFile {
            path: path.to_string_lossy().to_string(),
            filename: file_name,
            size,
            file_type,
        });
    }
    
    // Sort by filename
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    
    Ok(files)
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
