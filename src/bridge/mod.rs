// src/bridge/mod.rs

pub mod app;
pub mod logging;

pub mod mcp;
pub mod rmcp;
pub mod acp;

#[cfg(target_os = "windows")]
pub mod windows_console;
