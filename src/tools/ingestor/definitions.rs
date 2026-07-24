// src/tools/ingestor/definitions.rs
// MCP tool definitions with JSON schemas

use crate::bridge::mcp::McpTool;

pub const INGEST_FILES: &str = "ingest_files";
pub const LIST_IMPORTABLE: &str = "list_importable";
pub const TRANSCRIBE_AUDIO: &str = "transcribe_audio";
pub const LIST_INGESTED_FILES: &str = "list_ingested_files";
pub const DELETE_INGESTED_FILES: &str = "delete_ingested_files";

pub fn all() -> Vec<McpTool> {
    vec![
        McpTool {
            name: INGEST_FILES.to_string(),
            description: "INGEST FILES INTO MEMORY. Default: folder='files_to_import' (in robot_brain.exe directory). IMPORTANT: Always use limit=1 to ingest ONE file at a time. Files are stored in robot_brain.db (same directory as exe). DO NOT batch ingest - always one file at a time.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Defaults to 'files_to_import' - it's ALREADY next to robot_brain.exe. You don't need to specify this unless using a different folder. Example: 'files_to_import'"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "SINGLE FILE MODE - Ingest one specific file by full path. Example: 'C:\\robot_brain\\files_to_import\\notes.txt'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "ALWAYS use limit=1 for single file ingestion. Default is 1. Example: limit=1"
                    },
                    "chunk_size": {
                        "type": "integer",
                        "description": "Chunk size for splitting text (default: 1000)"
                    },
                    "memory_type": {
                        "type": "string",
                        "description": "Memory type: file, conversation, code, note (default: file)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: false). Set to true to include files in subdirectories."
                    }
                }
            }),
        },
        McpTool {
            name: LIST_IMPORTABLE.to_string(),
            description: "LIST FILES READY FOR IMPORT. Automatically looks in 'files_to_import' folder (same directory as robot_brain.exe). Returns list of files with full paths. No need to search - just call this tool.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Leave empty - defaults to 'files_to_import' which is already next to robot_brain.exe"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return (default: 5)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: false). Set to true to include files in subdirectories."
                    }
                }
            }),
        },
        McpTool {
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
                "required": ["path"]
            }),
        },
        McpTool {
            name: LIST_INGESTED_FILES.to_string(),
            description: "List files that have been successfully ingested and can now be deleted.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Import folder path (default: files_to_import)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return"
                    }
                }
            }),
        },
        McpTool {
            name: DELETE_INGESTED_FILES.to_string(),
            description: "Delete original files after they have been ingested. ALWAYS ask user for confirmation before calling this tool!".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File paths to delete. MUST be files that were already ingested."
                    },
                    "confirmation": {
                        "type": "string",
                        "description": "VERIFICATION REQUIRED: Must be EXACTLY 'yes' to confirm deletion. Without this, deletion will NOT proceed."
                    }
                },
                "required": ["files", "confirmation"]
            }),
        },
    ]
}
