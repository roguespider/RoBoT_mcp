pub fn init_logging() {
    // For MCP stdio transport, we must NOT log to stdout
    // Use a null writer that discards all output but still allows manual stderr writes
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(|| std::io::sink())  // Discard tracing logs
        .with_ansi(false)
        .init();
    
    // Print startup message to stderr so users know the server started
    eprintln!("RoBoT Brain MCP server starting...");
}
