// src/tools/ingestor/core.rs
// Core file ingestion logic

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time;

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
use crate::tools::ingestor::file_collector::{collect_all_files_recursive, collect_importable_files, collect_importable_files_with_recursive, get_import_folder, is_supported_extension, JSON_EXTENSIONS};
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

/// Larger chunk size for JSON files (better for structured data)
pub const JSON_CHUNK_SIZE: usize = 16384;

/// Tracks recently ingested files for deletion verification
/// This prevents agents from deleting files without proper ingestion
pub struct IngestTracker {
    recently_ingested: HashSet<String>,
    last_ingest_time: Option<Instant>,
}

impl IngestTracker {
    pub fn new() -> Self {
        Self {
            recently_ingested: HashSet::new(),
            last_ingest_time: None,
        }
    }
    
    /// Record that files were ingested
    pub fn record_ingestion(&mut self, file_paths: Vec<String>) {
        for path in file_paths {
            self.recently_ingested.insert(path);
        }
        self.last_ingest_time = Some(Instant::now());
    }
    
    /// Check if a file was recently ingested
    pub fn was_recently_ingested(&self, file_path: &str) -> bool {
        // Normalize path for comparison
        let normalized = Path::new(file_path)
            .to_path_buf()
            .to_string_lossy()
            .to_lowercase();
        
        // Check exact match
        if self.recently_ingested.iter().any(|p| {
            Path::new(p).to_path_buf().to_string_lossy().to_lowercase() == normalized
        }) {
            return true;
        }
        
        // Check if it's in files_to_import (allow deletion of any file from import folder)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let import_folder = exe_dir.join("files_to_import");
                if let Ok(file_path_buf) = Path::new(file_path).canonicalize() {
                    if let Ok(import_canonical) = import_folder.canonicalize() {
                        return file_path_buf.starts_with(import_canonical);
                    }
                }
            }
        }
        
        false
    }
    
    /// Check if we can verify deletion (means a recent ingest happened)
    #[allow(dead_code)]
    pub fn can_verify_deletion(&self) -> bool {
        match self.last_ingest_time {
            Some(time) => time.elapsed() < Duration::from_secs(300), // 5 minute window
            None => false,
        }
    }
    
    /// Clear the tracker (after successful deletion or timeout)
    pub fn clear(&mut self) {
        self.recently_ingested.clear();
        self.last_ingest_time = None;
    }
}

impl Default for IngestTracker {
    fn default() -> Self {
        Self::new()
    }
}

// Global ingest tracker
static INGEST_TRACKER: std::sync::OnceLock<tokio::sync::Mutex<IngestTracker>> = std::sync::OnceLock::new();

fn get_ingest_tracker() -> &'static tokio::sync::Mutex<IngestTracker> {
    INGEST_TRACKER.get_or_init(|| tokio::sync::Mutex::new(IngestTracker::new()))
}

/// Record files as ingested (call after successful ingest)
pub async fn record_ingested_files(file_paths: Vec<String>) {
    if let Ok(mut tracker) = get_ingest_tracker().try_lock() {
        tracker.record_ingestion(file_paths);
    }
}

/// Check if files can be deleted
pub async fn can_delete_files(file_paths: &[String]) -> (bool, Vec<String>) {
    if let Ok(tracker) = get_ingest_tracker().try_lock() {
        let unverified: Vec<String> = file_paths
            .iter()
            .filter(|p| !tracker.was_recently_ingested(p))
            .cloned()
            .collect();
        
        let all_verified = unverified.is_empty();
        (all_verified, unverified)
    } else {
        (true, vec![]) // If can't lock, allow (fail open for now)
    }
}

/// Clear the ingest tracker
pub async fn clear_ingest_tracker() {
    if let Ok(mut tracker) = get_ingest_tracker().try_lock() {
        tracker.clear();
    }
}

// ============================================================================
// INPUT/OUTPUT TYPES
// ============================================================================

/// Default timeout for ingestion operations (60 seconds)
pub const DEFAULT_INGEST_TIMEOUT_SECS: u64 = 60;

/// Tool: Ingest files from import folder
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IngestFilesInput {
    pub folder: Option<String>,
    pub file_path: Option<String>,
    pub limit: Option<usize>,
    pub chunk_size: Option<usize>,
    pub memory_type: Option<String>,
    /// Timeout in seconds for the entire ingestion operation (default: 60)
    /// Increase this value for large files or slow storage
    pub timeout_seconds: Option<u64>,
    /// Search subfolders recursively (default: false)
    pub recursive: Option<bool>,
    /// Force re-ingestion of already-ingested files (default: false)
    /// Use this when user confirms they want to add a file again
    pub force: Option<bool>,
}

/// Tool: List files ready for import
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImportableInput {
    pub folder: Option<String>,
    pub limit: Option<usize>,
    /// Search subfolders recursively (default: false)
    pub recursive: Option<bool>,
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
    pub chunk_size_used: usize,
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
    let timeout_secs = input.timeout_seconds.unwrap_or(DEFAULT_INGEST_TIMEOUT_SECS);
    let recursive = input.recursive.unwrap_or(false);
    let force = input.force.unwrap_or(false);

    tracing::info!("Starting file ingestion: limit={}, chunk_size={}, timeout={}s, recursive={}, force={}", 
                   limit, chunk_size, timeout_secs, recursive, force);

    // Check if ingesting a specific file or from folder
    if let Some(file_path) = file_path {
        let path = Path::new(file_path);
        if path.exists() {
            let result = time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_single_file(path, chunk_size, memory_type, db)
            ).await;
            
            match result {
                Ok(Ok(ingest_result)) => {
                    tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                    Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to ingest file: {}", e);
                    Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                }
                Err(_) => {
                    tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                    Ok(ToolOutput::error(format!(
                        "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                        timeout_secs
                    )))
                }
            }
        } else {
            // Try relative to folder
            let path = folder.join(file_path);
            if path.exists() {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_single_file(&path, chunk_size, memory_type, db)
                ).await;
                
                match result {
                    Ok(Ok(ingest_result)) => {
                        tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                        Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Failed to ingest file: {}", e);
                        Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                    }
                    Err(_) => {
                        tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                        Ok(ToolOutput::error(format!(
                            "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                            timeout_secs
                        )))
                    }
                }
            } else {
                Ok(ToolOutput::error(format!("File not found: {}", file_path)))
            }
        }
    } else {
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
            let result = time::timeout(
                Duration::from_secs(timeout_secs),
                ingest_single_file(&folder, chunk_size, memory_type, db)
            ).await;
            
            return match result {
                Ok(Ok(ingest_result)) => {
                    tracing::info!("Ingested file successfully: {} chunks", ingest_result.chunks_created);
                    Ok(ToolOutput::success(serde_json::to_value(ingest_result)?))
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to ingest file: {}", e);
                    Ok(ToolOutput::error(format!("Failed to ingest file: {}", e)))
                }
                Err(_) => {
                    tracing::error!("Ingestion timed out after {} seconds", timeout_secs);
                    Ok(ToolOutput::error(format!(
                        "Ingestion timed out after {} seconds. Try increasing timeout_seconds for large files.",
                        timeout_secs
                    )))
                }
            };
        }

        // Collect files from folder (with optional recursive search)
        let all_files = if recursive {
            collect_importable_files_with_recursive(&folder, true)?
        } else {
            collect_importable_files(&folder)?
        };
        
        // Filter out files with skip reasons
        let (skipped_files, files_to_check): (Vec<_>, Vec<_>) = all_files
            .into_iter()
            .partition(|f| f.skip_reason.is_some());
        
        // Check for already-ingested files and separate them
        let ingest_tracker = get_ingest_tracker().try_lock();
        let (already_ingested, files_to_process): (Vec<_>, Vec<_>) = files_to_check
            .into_iter()
            .partition(|f| {
                if !force {
                    // Only check tracker if not forcing
                    if let Some(ref tracker) = ingest_tracker {
                        return tracker.was_recently_ingested(&f.path);
                    }
                }
                false
            });
        
        let files_to_process: Vec<_> = files_to_process.into_iter().take(limit).collect();

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut total_chunks = 0;
        let mut timeout_occurred = false;

        for file_info in files_to_process {
            let path = Path::new(&file_info.path);
            let filename = file_info.filename.clone();

            // Check if it's an archive
            if file_info.file_type == "archive" {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_archive(path, chunk_size, memory_type.clone(), db.clone())
                ).await;
                
                match result {
                    Ok(Ok(result)) => {
                        results.push(result);
                        successful += 1;
                    }
                    Ok(Err(e)) => {
                        failed += 1;
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            memory_ids: vec![],
                            error: Some(e.to_string()),
                            remaining_count: 0,
                        });
                    }
                    Err(_) => {
                        timeout_occurred = true;
                        failed += 1;
                        tracing::error!("Archive ingestion timed out for: {}", file_info.filename);
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            memory_ids: vec![],
                            error: Some(format!("Ingestion timed out after {} seconds. Try increasing timeout_seconds.", timeout_secs)),
                            remaining_count: 0,
                        });
                        break; // Stop processing on timeout
                    }
                }
            } else {
                let result = time::timeout(
                    Duration::from_secs(timeout_secs),
                    ingest_single_file(path, chunk_size, memory_type.clone(), db.clone())
                ).await;
                
                match result {
                    Ok(Ok(result)) => {
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
                    Ok(Err(e)) => {
                        failed += 1;
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            memory_ids: vec![],
                            error: Some(e.to_string()),
                            remaining_count: 0,
                        });
                    }
                    Err(_) => {
                        timeout_occurred = true;
                        failed += 1;
                        tracing::error!("File ingestion timed out for: {}", file_info.filename);
                        results.push(IngestResult {
                            filename,
                            file_path: file_info.path.clone(),
                            success: false,
                            chunks_created: 0,
                            memory_ids: vec![],
                            error: Some(format!("Ingestion timed out after {} seconds. Try increasing timeout_seconds.", timeout_secs)),
                            remaining_count: 0,
                        });
                        break; // Stop processing on timeout
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

        // RECORD INGESTED FILES for deletion tracking
        // This enables the delete_ingested_files tool to verify files were actually ingested
        if !successfully_ingested.is_empty() {
            record_ingested_files(successfully_ingested.clone()).await;
        }

        // CLEANUP WAL FILES after batch operations
        // This checkpoints the WAL and cleans up the -wal and -shm files
        if let Err(e) = db.cleanup_wal_files() {
            tracing::warn!("Failed to cleanup WAL files: {}", e);
        }

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

        // Format already ingested files for display
        let already_ingested_filenames: Vec<String> = already_ingested.iter().map(|f| f.filename.clone()).collect();
        
        Ok(ToolOutput::success(serde_json::json!({
            "summary": summary,
            "successfully_ingested": successfully_ingested,
            "import_folder": folder_display,
            "exe_directory": exe_dir,
            "temp_folder": get_recent_archive_temp_folder().map(|p| p.to_string_lossy().to_string()),
            "remaining_in_temp": remaining_count,
            "files_stored_in": format!("robot_brain.db in {}", exe_dir),
            "files_ready_for_deletion": successfully_ingested.clone(),
            "already_ingested": already_ingested_filenames,
            "already_ingested_count": already_ingested.len(),
            "skipped_count": skipped_files.len(),
            "IMPORTANT_SCOPING": {
                "scope": "ONLY look in import_folder for files",
                "do_not_look": ["current project folder", "source code directories", "anywhere outside import_folder"],
                "this_folder": folder_display,
                "reason": "robot_brain.exe, robot_brain.db, and files_to_import are all in the robot_brain directory"
            },
            "note": if remaining_count > 0 {
                format!("{} file(s) remaining in temp folder. Call ingest again with temp_folder path.", remaining_count)
            } else if successfully_ingested.is_empty() && already_ingested.is_empty() {
                "No files were ingested. Files may have been skipped (size limits, embedding patterns).".to_string()
            } else if successfully_ingested.is_empty() {
                "No new files were ingested.".to_string()
            } else {
                "All files ingested.".to_string()
            },
            "NEXT_ACTION": if !already_ingested.is_empty() && successfully_ingested.is_empty() {
                // All files were already ingested - ask user if they want to re-ingest
                serde_json::json!("ASK USER: 'The following files have already been added to memory: {:?}. Do you want to add them again?'. If YES, use force=true parameter.".replace("{:?}", &format!("{:?}", already_ingested_filenames)))
            } else if successfully_ingested.is_empty() {
                "No action needed - no files were successfully ingested."
            } else {
                "ASK USER: 'I successfully ingested the file(s). Can I delete the original file(s) to save space?'"
            },
            "deletion_workflow": {
                "if_user_says_yes": {
                    "step_1": "Call delete_ingested_files",
                    "step_2": "Use files from 'files_ready_for_deletion' list",
                    "step_3": "Set confirmation='yes'",
                    "step_4": "Check response.empty_folders - if not empty, ASK USER about folder cleanup",
                    "example": {
                        "files": successfully_ingested,
                        "confirmation": "yes"
                    }
                },
                "if_user_says_no": {
                    "action": "Keep files - no deletion needed",
                    "note": "Files will NOT be offered again on next ingest_files call"
                },
                "files_pending_deletion": successfully_ingested.len()
            },
            "folder_cleanup": {
                "note": "After file deletion, check delete_ingested_files response for empty_folders",
                "scenario": "If empty_folders is not empty, folders are now empty and can be deleted",
                "ask_user": "ASK USER: 'Do you want to delete the empty folder(s)?'",
                "warning": "Only delete folders INSIDE files_to_import, never the files_to_import folder itself"
            },
            "timeout_occurred": timeout_occurred,
            "timeout_help": if timeout_occurred {
                serde_json::json!("A timeout occurred. To fix this, call ingest_files again with a higher timeout_seconds value (e.g., 300 for 5 minutes).")
            } else {
                serde_json::Value::Null
            },
            "recursive_used": recursive,
            "warning": "You MUST ask the user before calling delete_ingested_files. Do NOT delete without asking."
        })))
    }
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
            chunk_size_used: chunk_size,
            memory_ids: vec![],
            error: Some("File contains no text".to_string()),
            remaining_count: 0,
        });
    }

    // Use larger chunk size for JSON files (better for structured data)
    let actual_chunk_size = if is_supported_extension(path, JSON_EXTENSIONS) {
        JSON_CHUNK_SIZE
    } else {
        chunk_size
    };

    // Chunk the text
    let chunks = chunk_text(&text, actual_chunk_size, DEFAULT_CHUNK_OVERLAP);

    // Store each chunk as a memory card using batch inserts for performance
    let mut memory_ids = Vec::new();
    let batch_size = 100; // Insert 100 chunks per transaction

    for batch in chunks.chunks(batch_size) {
        let memories: Vec<MemoryCard> = batch
            .iter()
            .map(|chunk| MemoryCard::new(chunk.clone(), memory_type.clone()))
            .collect();

        let conn = db.connection()?;
        queries::insert_memories_batch(&conn, &memories)?;
        
        // Collect the IDs from this batch
        for memory in &memories {
            memory_ids.push(memory.id.to_string());
        }
    }

    Ok(IngestResult {
        filename,
        file_path: path.to_string_lossy().to_string(),
        success: true,
        chunks_created: chunks.len(),
        chunk_size_used: actual_chunk_size,
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
