// src/tools/ingestor/ingestor.rs
// Main file ingestion orchestration

pub mod archive_handler;
pub mod file_collector;
pub mod text_extractor;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::tools::ToolOutput;
use crate::tools::ingestor::archive_handler::{self, get_recent_archive_temp_folder, process_archive};
use crate::tools::ingestor::file_collector::{self, ImportableFile, collect_importable_files, get_import_folder};
use crate::tools::ingestor::text_extractor::{extract_text, chunk_text};

/// Default chunk size for text splitting
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Default overlap between chunks
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IngestFilesInput {
    pub folder: Option<String>,
    pub file_path: Option<String>,
    pub limit: Option<usize>,
    pub chunk_size: Option<usize>,
    pub memory_type: Option<String>,
}

/// Tool: List files ready for import
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImportableInput {
    pub folder: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Transcribe an audio file
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TranscribeAudioInput {
    pub path: String,
    pub output: Option<String>,
}

/// Tool: Delete successfully imported files
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteIngestedFilesInput {
    pub files: Vec<String>,
    pub confirmation: String,
}

/// Tool: List files that were successfully ingested
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListIngestedFilesInput {
    pub folder: Option<String>,
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
    pub remaining_count: usize,
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
                description: "Ingest files into short-term memory.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder containing files. Defaults to 'files_to_import'"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "INGEST ONE FILE - path to specific file"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max files to ingest (default: 1)"
                        },
                        "chunk_size": {
                            "type": "integer",
                            "description": "Chunk size for splitting text"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Memory type: file, conversation, code, note"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_IMPORTABLE.to_string(),
                description: "List files ready for import.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Path to folder to check"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max files to return (default: 5)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: TRANSCRIBE_AUDIO.to_string(),
                description: "Transcribe audio file to text.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to audio file"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output path for transcription JSON"
                        }
                    },
                    required: ["path"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_INGESTED_FILES.to_string(),
                description: "List successfully ingested files for deletion.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": {
                            "type": "string",
                            "description": "Import folder"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max files to return"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: DELETE_INGESTED_FILES.to_string(),
                description: "Delete ingested files (requires confirmation).".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "File paths to delete"
                        },
                        "confirmation": {
                            "type": "string",
                            "description": "Must be 'yes' or 'confirm'"
                        }
                    },
                    required: ["files", "confirmation"]
                }),
            },
        ]
    }
}

// ============================================================================
}

// ============================================================================
// MAIN INGESTION FUNCTIONS
// ============================================================================

pub async fn ingest_file(
    input: IngestFilesInput,
    db: Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let file_path = input.file_path.as_ref();
    let limit = input.limit.unwrap_or(1);
    let chunk_size = input.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    let memory_type = parse_memory_type(input.memory_type.as_deref().unwrap_or("file"));
    
    // Check if ingesting a specific file or from folder
    if let Some(file_path) = file_path {
        let path = Path::new(file_path);
        if path.exists() {
            return ingest_single_file(path, chunk_size, memory_type, db).await;
        }
        
        // Try relative to folder
        let path = folder.join(file_path);
        if path.exists() {
            return ingest_single_file(&path, chunk_size, memory_type, db).await;
        }
        
        return Ok(ToolOutput::error(format!("File not found: {}", file_path)));
    }
    
    // Ingest from folder
    if !folder.exists() {
        return Ok(ToolOutput::error(format!(
            "Import folder does not exist: {}",
            folder.display()
        )));
    }
    
    // If folder is a file, ingest it directly
    if folder.is_file() {
        return ingest_single_file(&folder, chunk_size, memory_type, db).await;
    }
    
    // Collect files from folder
    let files = collect_importable_files(&folder)?;
    let files_to_process: Vec<_> = files.into_iter().take(limit).collect();
    
    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    let mut total_chunks = 0;
    
    for file_info in files_to_process {
        let path = Path::new(&file_info.path);
        
        // Check if it's an archive
        if file_info.file_type == "archive" {
            match ingest_archive(path, chunk_size, memory_type, db.clone()).await {
                Ok(result) => {
                    results.push(result);
                    successful += 1;
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
                        remaining_count: 0,
                    });
                }
            }
        } else {
            match ingest_single_file(path, chunk_size, memory_type, db.clone()).await {
                Ok(result) => {
                    results.push(result);
                    if result.success {
                        successful += 1;
                        total_chunks += result.chunks_created;
                    } else {
                        failed += 1;
                    }
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
                        remaining_count: 0,
                    });
                }
            }
        }
    }
    
    let total_files = results.len();
    
    // Collect successfully ingested file paths
    let successfully_ingested: Vec<String> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.file_path.clone())
        .collect();
    
    // Get remaining count from archive ingestion
    let remaining_count: usize = results.iter().map(|r| r.remaining_count).sum();
    
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
        "temp_folder": get_recent_archive_temp_folder().map(|p| p.to_string_lossy().to_string()),
        "remaining_in_temp": remaining_count,
        "note": if remaining_count > 0 {
            format!("{} file(s) remaining in temp folder. Call ingest again with temp_folder path.", remaining_count)
        } else {
            "All files ingested.".to_string()
        },
        "workflow": "1. Ingest files\n2. Review remaining_in_temp\n3. Ingest more or delete originals"
    })))
}

/// Ingest an archive file
async fn ingest_archive(
    path: &Path,
    chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive")
        .to_string();
    
    // Create temp directory for extraction
    let temp_dir = archive_handler::create_archive_temp_dir(&filename);
    std::fs::create_dir_all(&temp_dir)?;
    
    // Process archive
    let files = process_archive(path, &temp_dir)?;
    
    if files.is_empty() {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            memory_ids: vec![],
            error: Some("Archive is empty".to_string()),
            remaining_count: 0,
        });
    }
    
    // Ingest the first file
    let first_file = &files[0];
    let result = ingest_single_file(first_file, chunk_size, memory_type, db).await?;
    
    // Clean up empty subfolders
    archive_handler::delete_empty_folders(&temp_dir);
    
    // Count remaining files
    let remaining_files = archive_handler::collect_all_files_recursive(&temp_dir)?;
    let remaining_count = remaining_files.len();
    
    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: result.success,
        chunks_created: result.chunks_created,
        memory_ids: result.memory_ids,
        error: result.error,
        remaining_count,
    })
}

/// Ingest a single file into memory
async fn ingest_single_file(
    path: &Path,
    chunk_size: usize,
    memory_type: MemoryType,
    db: Arc<SqliteDatabase>,
) -> Result<IngestResult> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract text content
    let text = extract_text(path)
        .with_context(|| format!("Failed to extract text from {}", filename))?;
    
    if text.trim().is_empty() {
        return Ok(IngestResult {
            filename,
            file_path: path.to_string_lossy().to_string(),
            success: false,
            chunks_created: 0,
            memory_ids: vec![],
            error: Some("File contains no text".to_string()),
            remaining_count: 0,
        });
    }
    
    // Chunk the text
    let chunks = chunk_text(&text, chunk_size, DEFAULT_CHUNK_OVERLAP);
    
    // Store each chunk as a memory card
    let mut memory_ids = Vec::new();
    
    for chunk in &chunks {
        let memory = MemoryCard {
            id: Uuid::new_v4(),
            content: chunk.clone(),
            memory_type: memory_type.clone(),
            source: Some(path.to_string_lossy().to_string()),
            created_at: chrono::Utc::now(),
            access_count: 0,
            last_accessed: None,
            importance_score: Some(0.5),
            summary: None,
            metadata: None,
            recall_count: 0,
            recall_interval: None,
        };
        
        queries::create_memory(&db, &memory)?;
        memory_ids.push(memory.id.to_string());
    }
    
    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: chunks.len(),
        memory_ids,
        error: None,
        remaining_count: 0,
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

// ============================================================================
// LIST IMPORTABLE
// ============================================================================

pub async fn execute_list_importable(
    input: ListImportableInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(5);
    
    if !folder.exists() {
        return Ok(ToolOutput::success(serde_json::json!({
            "files": [],
            "folder": folder.to_string_lossy(),
            "message": "Folder does not exist"
        })));
    }
    
    let files = collect_importable_files(&folder)?;
    let files: Vec<_> = files.into_iter().take(limit).collect();
    
    let total = collect_importable_files(&folder)?.len();
    
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "folder": folder.to_string_lossy(),
        "count": files.len(),
        "total": total,
        "message": if files.is_empty() {
            "No importable files found".to_string()
        } else {
            format!("Found {} files (showing {})", total, files.len())
        }
    })))
}

// ============================================================================
// LIST/DELETE INGESTED FILES
// ============================================================================

pub async fn execute_list_ingested_files(
    input: ListIngestedFilesInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(50);
    
    let files = collect_importable_files(&folder)?;
    let files: Vec<_> = files.into_iter().take(limit).collect();
    
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "count": files.len(),
        "warning": "These files have been ingested into memory. Delete originals if no longer needed."
    })))
}

pub async fn execute_delete_ingested_files(
    input: DeleteIngestedFilesInput,
) -> Result<ToolOutput> {
    // Verify confirmation
    if input.confirmation.to_lowercase() != "yes" && input.confirmation.to_lowercase() != "confirm" {
        return Ok(ToolOutput::error("Deletion cancelled. Must confirm with 'yes' or 'confirm'.".to_string()));
    }
    
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
        "deleted": deleted,
        "deleted_count": deleted.len(),
        "failed": failed,
        "failed_count": failed.len(),
        "message": if deleted.is_empty() {
            "No files were deleted".to_string()
        } else {
            format!("Successfully deleted {} file(s)", deleted.len())
        }
    })))
}

// ============================================================================
// AUDIO TRANSCRIPTION
// ============================================================================

pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
) -> Result<ToolOutput> {
    let path = Path::new(&input.path);
    
    if !path.exists() {
        return Ok(ToolOutput::error(format!("File not found: {}", input.path)));
    }
    
    // Check if whisper-cli is available
    let output = std::process::Command::new("which")
        .arg("whisper")
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            // Use whisper-cli for transcription
            let output_path = input.output.unwrap_or_else(|| {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                format!("{}_transcription.json", stem)
            });
            
            let result = std::process::Command::new("whisper")
                .arg(&input.path)
                .arg("--output_format")
                .arg("json")
                .arg("--output_dir")
                .arg(std::path::Path::new(&output_path).parent().unwrap_or(std::path::Path::new(".")))
                .output()?;
            
            if result.status.success() {
                Ok(ToolOutput::success(serde_json::json!({
                    "transcription": format!("{}.json", path.file_stem().unwrap_or_default().to_string_lossy()),
                    "path": input.path
                })))
            } else {
                Ok(ToolOutput::error(format!(
                    "Whisper failed: {}",
                    String::from_utf8_lossy(&result.stderr)
                )))
            }
        }
        _ => {
            // No whisper available
            Ok(ToolOutput::error(
                "Audio transcription requires whisper-cli to be installed.\n\
                 Install with: pip install whisper\n\
                 Or: brew install whisper".to_string()
            ))
        }
    }
}
