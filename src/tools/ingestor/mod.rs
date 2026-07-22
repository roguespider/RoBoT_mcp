// src/tools/ingestor/mod.rs
// Ingestor module - file ingestion for short-term memory

pub mod archive_handler;
pub mod audio;
pub mod core;
pub mod definitions;
pub mod file_collector;
pub mod text_extractor;
pub mod workflow;

// Re-export main types and functions
pub use core::{
    execute_delete_ingested_files, execute_list_importable,
    execute_list_ingested_files, execute_transcribe_audio, ingest_file,
    IngestFilesInput, ListImportableInput,
    DeleteIngestedFilesInput, ListIngestedFilesInput, TranscribeAudioInput,
    record_ingested_files, can_delete_files, clear_ingest_tracker,
};
