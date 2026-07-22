// src/tools/ingestor/mod.rs
// Ingestor module - file ingestion for short-term memory

pub mod archive_handler;
pub mod file_collector;
pub mod text_extractor;
pub mod ingestor;

// Re-export main types and functions
pub use ingestor::{
    definitions, execute_delete_ingested_files, execute_list_importable,
    execute_list_ingested_files, execute_transcribe_audio, ingest_file,
    IngestFilesInput, IngestResult, IngestSummary, ListImportableInput,
    DeleteIngestedFilesInput, ListIngestedFilesInput, TranscribeAudioInput,
    DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP,
};
