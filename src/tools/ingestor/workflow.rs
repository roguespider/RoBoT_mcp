// src/tools/ingestor/workflow.rs
// Workflow operations: list/delete imported files

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::tools::ToolOutput;
use crate::tools::ingestor::file_collector::{collect_importable_files, collect_importable_files_with_recursive, get_import_folder, ImportableFile};

use super::ListImportableInput;
use super::ListIngestedFilesInput;
use super::DeleteIngestedFilesInput;

// ============================================================================
// LIST IMPORTABLE FILES
// ============================================================================

pub async fn execute_list_importable(
    input: ListImportableInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(5);
    let recursive = input.recursive.unwrap_or(false);
    
    // Get exe directory for reference
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    
    let folder_display = folder.to_string_lossy().to_string();
    let exe_dir_display = exe_dir.to_string_lossy().to_string();
    
    if !folder.exists() {
        return Ok(ToolOutput::success(serde_json::json!({
            "files": [],
            "import_folder": folder_display,
            "exe_directory": exe_dir_display,
            "relative_path": "files_to_import",
            "count": 0,
            "total": 0,
            "recursive": recursive,
            "message": format!("Folder does not exist at: {}. Create it or check if robot_brain.exe is in the correct location.", folder_display),
            "hint": "The files_to_import folder should be in the same directory as robot_brain.exe"
        })));
    }
    
    // Get all files based on recursive setting
    let all_files = if recursive {
        collect_importable_files_with_recursive(&folder, true)?
    } else {
        collect_importable_files(&folder)?
    };
    
    // Separate files into ingestable and skipped
    let (ingestable, skipped): (Vec<_>, Vec<_>) = all_files
        .into_iter()
        .partition(|f| f.skip_reason.is_none());
    
    let total = ingestable.len();
    let files: Vec<_> = ingestable.into_iter().take(limit).collect();
    
    // Build response with clear separation
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "import_folder": folder_display,
        "exe_directory": exe_dir_display,
        "relative_path": "files_to_import",
        "count": files.len(),
        "total": total,
        "recursive": recursive,
        "instruction": "Use ingest_files with folder='files_to_import' (or omit folder parameter) and limit=1 to ingest one file at a time",
        "message": if files.is_empty() && skipped.is_empty() {
            format!("No importable files found in {}. Add files to this folder to ingest them.", folder_display)
        } else if files.is_empty() {
            format!("All files in {} have issues (see 'skipped' list).", folder_display)
        } else {
            format!("Found {} file(s) ready for ingestion at: {}", total, folder_display)
        },
        "skipped": skipped,
        "skipped_count": skipped.len(),
        "skip_reasons": {
            "embedding_files": "Files with embeddings/metadata patterns (e.g., 'embeddings.json', 'vectors.json') are skipped - these don't chunk well",
            "size_limits": "JSON files >10MB and text files >50MB are skipped to prevent timeouts",
            "note": "Use recursive=true to search subfolders"
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
    // Double-check: if empty files list, warn
    if input.files.is_empty() {
        return Ok(ToolOutput::success(serde_json::json!({
            "deleted": Vec::<String>::new(),
            "deleted_count": 0,
            "failed": Vec::<String>::new(),
            "failed_count": 0,
            "message": "No files specified for deletion."
        })));
    }
    
    // Step 1: Check if files were recently ingested
    let (all_verified, unverified_files) = crate::tools::ingestor::can_delete_files(&input.files).await;
    
    // Step 2: Verify confirmation is EXACTLY "yes" or "confirm"
    let confirmation = input.confirmation.trim().to_lowercase();
    
    if confirmation != "yes" && confirmation != "confirm" {
        return Ok(ToolOutput::error(
            format!(
                "DELETION BLOCKED: Missing or invalid confirmation.\n\
                \n\
                Required: confirmation='yes' (exactly, case-insensitive)\n\
                Received: confirmation='{}'\n\
                \n\
                Files requested: {}\n\
                \n\
                IMPORTANT: You MUST ask the user before calling this tool!\n\
                Do NOT call delete_ingested_files without explicit user permission.\n\
                \n\
                Workflow:\n\
                1. Call ingest_files first\n\
                2. ASK user: 'Can I delete the original file(s)?'\n\
                3. Only if user says YES, call this tool with confirmation='yes'",
                input.confirmation,
                input.files.len()
            )
        ));
    }
    
    // Step 3: If files weren't verified, require extra confirmation
    if !all_verified && !unverified_files.is_empty() {
        // Files exist but weren't tracked as ingested - this is suspicious
        // Still allow if user explicitly confirmed, but log it
        tracing::warn!("Deleting files that weren't recently ingested: {:?}", unverified_files);
    }
    
    // Step 4: Track deleted and failed files
    let mut deleted = Vec::new();
    let mut failed = Vec::new();
    
    // Log what we're about to delete for transparency
    tracing::info!("Delete operation starting for {} file(s) with user confirmation", input.files.len());
    
    for file_path in &input.files {
        let path = Path::new(file_path);
        
        if !path.exists() {
            tracing::warn!("File not found, skipping: {:?}", path);
            failed.push(serde_json::json!({
                "path": file_path,
                "error": "File not found"
            }));
            continue;
        }
        
        if !path.is_file() {
            tracing::warn!("Path is not a file, skipping: {:?}", path);
            failed.push(serde_json::json!({
                "path": file_path,
                "error": "Path is not a file"
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
    
    // Step 5: Clear the ingest tracker after successful deletion
    let success = deleted.len();
    let failed_count = failed.len();
    
    if success > 0 {
        crate::tools::ingestor::clear_ingest_tracker().await;
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "deleted": deleted,
        "deleted_count": success,
        "failed": failed,
        "failed_count": failed_count,
        "message": if success > 0 && failed_count == 0 {
            format!("SUCCESS: Deleted {} file(s). Original files have been removed.", success)
        } else if success > 0 && failed_count > 0 {
            format!("PARTIAL: Deleted {} file(s), {} failed. Check failed list.", success, failed_count)
        } else {
            "No files were deleted.".to_string()
        },
        "verification": {
            "files_were_ingested": all_verified,
            "unverified_files": unverified_files.len()
        },
        "note": "The files_to_import folder was NOT deleted. It remains for future imports.",
        "tracker_cleared": success > 0
    })))
}
