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
        McpTool {
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
        McpTool {
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
                "required": ["files", "confirmation"]
            }),
        },
    ]
}
