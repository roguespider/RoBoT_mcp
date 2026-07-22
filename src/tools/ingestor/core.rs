// src/tools/ingestor/core.rs
// Core file ingestion logic

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::database::models::{MemoryCard, MemoryType};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::tools::ToolOutput;
use crate::tools::ingestor::archive_handler::{
    create_archive_temp_dir, delete_empty_folders,
    get_recent_archive_temp_folder, process_archive,
};
use crate::tools::ingestor::file_collector::{collect_all_files_recursive, collect_importable_files, get_import_folder};
use crate::tools::ingestor::text_extractor::{chunk_text, extract_text};

// Re-export for convenience (via parent module)
pub use super::workflow::{
    execute_delete_ingested_files, execute_list_importable, execute_list_ingested_files,
};
pub use super::audio::execute_transcribe_audio;

/// Default chunk size for text splitting
pub const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Default overlap between chunks
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;

// ============================================================================
// INPUT/OUTPUT TYPES
// ============================================================================

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
            return Ok(ToolOutput::success(serde_json::to_value(ingest_single_file(path, chunk_size, memory_type, db).await?)?));
        }

        // Try relative to folder
        let path = folder.join(file_path);
        if path.exists() {
            return Ok(ToolOutput::success(serde_json::to_value(ingest_single_file(&path, chunk_size, memory_type, db).await?)?));
        }

        return Ok(ToolOutput::error(format!("File not found: {}", file_path)));
    }

    // Ingest from folder
    if !folder.exists() {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "robot_brain.exe directory".to_string());
        
        return Ok(ToolOutput::error(format!(
            "Import folder does not exist: {}\n\
            \n\
            The 'files_to_import' folder should be in: {}\n\
            \n\
            Create the folder and add files there, or put files_to_import next to robot_brain.exe",
            folder.display(),
            exe_dir
        )));
    }

    // If folder is a file, ingest it directly
    if folder.is_file() {
        return Ok(ToolOutput::success(serde_json::to_value(ingest_single_file(&folder, chunk_size, memory_type, db).await?)?));
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
            match ingest_archive(path, chunk_size, memory_type.clone(), db.clone()).await {
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
            match ingest_single_file(path, chunk_size, memory_type.clone(), db.clone()).await {
                Ok(result) => {
                    let chunks = result.chunks_created;
                    let success = result.success;
                    results.push(result);
                    if success {
                        successful += 1;
                        total_chunks += chunks;
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
    let successfully_ingested: Vec<String> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.file_path.clone())
        .collect();

    let remaining_count: usize = results.iter().map(|r| r.remaining_count).sum();

    // Get folder path for reference
    let folder_display = folder.to_string_lossy().to_string();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

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
        "import_folder": folder_display,
        "exe_directory": exe_dir,
        "temp_folder": get_recent_archive_temp_folder().map(|p| p.to_string_lossy().to_string()),
        "remaining_in_temp": remaining_count,
        "files_stored_in": format!("robot_brain.db in {}", exe_dir),
        "note": if remaining_count > 0 {
            format!("{} file(s) remaining in temp folder. Call ingest again with temp_folder path.", remaining_count)
        } else {
            "All files ingested.".to_string()
        },
        "workflow": "1. Ingest files\n2. Review remaining_in_temp\n3. ASK USER for confirmation before deleting originals\n4. Use delete_ingested_files with confirmation='yes' to delete originals"
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
    let temp_dir = create_archive_temp_dir(&filename);
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
    delete_empty_folders(&temp_dir);

    // Count remaining files
    let remaining_files = collect_all_files_recursive(&temp_dir)?;
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
        let memory = MemoryCard::new(chunk.clone(), memory_type.clone());

        let conn = db.connection()?;
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
