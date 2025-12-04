//! Diagnostics and error reporting for PhysLang
//!
//! This module provides utilities for reporting parse errors,
//! runtime errors, and other diagnostics to users.

use crate::parser::ParseError;

/// Format a parse error with context from the source code
pub fn format_parse_error(error: &ParseError, source: &str, line_num: Option<usize>) -> String {
    let mut msg = format!("Parse error: {}", error);
    
    if let Some(line) = line_num {
        let lines: Vec<&str> = source.lines().collect();
        if line < lines.len() {
            msg.push_str(&format!("\n  at line {}: {}", line + 1, lines[line]));
        }
    }
    
    msg
}

/// Format a runtime error with context
pub fn format_runtime_error(error: &dyn std::error::Error) -> String {
    format!("Runtime error: {}", error)
}

