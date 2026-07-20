// src/cli/output.rs
//! Output formatting utilities

use std::fmt;

/// ANSI color codes
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";

/// Color a string green
pub fn green(s: impl fmt::Display) -> String {
    format!("{}{}{}", GREEN, s, RESET)
}

/// Color a string red
pub fn red(s: impl fmt::Display) -> String {
    format!("{}{}{}", RED, s, RESET)
}

/// Color a string yellow
pub fn yellow(s: impl fmt::Display) -> String {
    format!("{}{}{}", YELLOW, s, RESET)
}

/// Color a string cyan
pub fn cyan(s: impl fmt::Display) -> String {
    format!("{}{}{}", CYAN, s, RESET)
}

/// Make a string bold
pub fn bold(s: impl fmt::Display) -> String {
    format!("{}{}{}", BOLD, s, RESET)
}

/// Separator types for tables
pub enum Separator {
    Line,
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Separator::Line => write!(f, "─────────────────────────────────────────"),
        }
    }
}

/// Print success message
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("\x1b[32m✓\x1b[0m {}", format!($($arg)*))
    };
}

/// Print error message
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("\x1b[31m✗\x1b[0m {}", format!($($arg)*))
    };
}

/// Print info message
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("\x1b[36mℹ\x1b[0m {}", format!($($arg)*))
    };
}

/// Print warning message
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("\x1b[33m⚠\x1b[0m {}", format!($($arg)*))
    };
}

/// Success function version
pub fn success_msg(s: impl fmt::Display) {
    println!("\x1b[32m✓\x1b[0m {}", s);
}

/// Error function version
pub fn error_msg(s: impl fmt::Display) {
    eprintln!("\x1b[31m✗\x1b[0m {}", s);
}

/// Info function version
pub fn info_msg(s: impl fmt::Display) {
    println!("\x1b[36mℹ\x1b[0m {}", s);
}

/// Warn function version
pub fn warn_msg(s: impl fmt::Display) {
    println!("\x1b[33m⚠\x1b[0m {}", s);
}

/// Print a table row with columns and widths
pub fn table_row(columns: &[&str], widths: &[usize]) {
    let mut row = String::new();
    for (col, &width) in columns.iter().zip(widths.iter()) {
        row.push_str(&format!("{:width$}  ", col, width = width));
    }
    println!("{}", row.trim_end());
}

/// Print table headers with separator
pub fn table_header(columns: &[&str], widths: &[usize]) {
    table_row(columns, widths);
    let sep: String = widths.iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", sep);
}

/// Print a formatted section header
pub fn section_header(title: &str) {
    println!();
    println!("{}", bold(title));
    println!("{}", Separator::Line);
}

/// Print a key-value pair with formatting
pub fn kv(key: &str, value: impl fmt::Display) {
    println!("  {}: {}", cyan(key), value);
}

/// Print a list item
pub fn list_item(item: &str) {
    println!("  • {}", item);
}

/// Print a numbered list item
pub fn numbered_item(n: usize, item: &str) {
    println!("  {}. {}", green(n.to_string()), item);
}
