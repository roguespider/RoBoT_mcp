// src/tools/ingestor/workflow.rs
// Workflow operations: list/delete imported files

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::tools::ToolOutput;
use crate::tools::ingestor::file_collector::{collect_importable_files, get_import_folder};

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
            "message": format!("Folder does not exist at: {}. Create it or check if robot_brain.exe is in the correct location.", folder_display),
            "hint": "The files_to_import folder should be in the same directory as robot_brain.exe"
        })));
    }
    
    let files = collect_importable_files(&folder)?;
    let files: Vec<_> = files.into_iter().take(limit).collect();
    
    let total = collect_importable_files(&folder)?.len();
    
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "import_folder": folder_display,
        "exe_directory": exe_dir_display,
        "relative_path": "files_to_import",
        "count": files.len(),
        "total": total,
        "instruction": "Use ingest_files with folder='files_to_import' (or omit folder parameter) and limit=1 to ingest one file at a time",
        "message": if files.is_empty() {
            format!("No importable files found in {}. Add files to this folder to ingest them.", folder_display)
        } else {
            format!("Found {} file(s) ready for ingestion at: {}", total, folder_display)
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
    // Strict confirmation verification - must be exactly "yes" or "confirm"
    let confirmation = input.confirmation.trim().to_lowercase();
    
    if confirmation != "yes" && confirmation != "confirm" {
        return Ok(ToolOutput::error(
            format!(
                "DELETION CANCELLED: Missing or invalid confirmation.\n\
                Files requested for deletion: {}\n\
                To delete, you MUST provide confirmation='yes' (exactly, case-insensitive).\n\
                No files were deleted.",
                input.files.len()
            )
        ));
    }
    
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
    
    // Track deleted and failed files
    let mut deleted = Vec::new();
    let mut failed = Vec::new();
    let mut parent_folders: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    // Log what we're about to delete for transparency
    tracing::info!("Delete operation starting for {} file(s)", input.files.len());
    
    for file_path in &input.files {
        let path = Path::new(file_path);
        
        // Track parent folder for potential cleanup
        if let Some(parent) = path.parent() {
            parent_folders.insert(parent.to_string_lossy().to_string());
        }
        
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
    
    // Note: Folder deletion is intentionally NOT done automatically
    // The folder (files_to_import) should remain for future use
    // Manual folder cleanup should be done by the user if desired
    
    let success = deleted.len();
    let failed_count = failed.len();
    
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
        "note": "The files_to_import folder was NOT deleted. It remains for future imports."
    })))
}
