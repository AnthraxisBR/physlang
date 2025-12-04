//! Diagnostics and error reporting for PhysLang
//!
//! This module provides utilities for reporting parse errors,
//! runtime errors, and static analysis diagnostics.

/// A span in the source code (byte offsets)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize, // byte offset
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Convert a span to a source location (line, column)
    pub fn to_location(&self, source: &str) -> SourceLocation {
        let mut line = 1;
        let mut column = 1;
        let mut offset = 0;

        for ch in source.chars() {
            if offset >= self.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            offset += ch.len_utf8();
        }

        SourceLocation { line, column }
    }
}

/// A source location (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A diagnostic message with location information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    /// Create an error diagnostic
    pub fn error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            span,
        }
    }

    /// Create a warning diagnostic
    pub fn warning(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            span,
        }
    }

    /// Convert span to source location if available
    pub fn location(&self, source: &str) -> Option<SourceLocation> {
        self.span.map(|s| s.to_location(source))
    }
}

/// Collection of diagnostics
#[derive(Debug, Clone)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
    }

    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Diagnostic>> for Diagnostics {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

/// Format a parse error with context from the source code
pub fn format_parse_error(error: &crate::parser::ParseError, source: &str) -> String {
    let mut msg = format!("Parse error: {}", error);
    
    if let Some(span) = error.span() {
        let location = span.to_location(source);
        msg.push_str(&format!("\n  at line {}, column {}", location.line, location.column));
        
        // Try to show the line
        let lines: Vec<&str> = source.lines().collect();
        if location.line > 0 && location.line <= lines.len() {
            msg.push_str(&format!("\n  {}", lines[location.line - 1]));
            if location.column > 0 {
                let caret = " ".repeat(location.column.saturating_sub(1)) + "^";
                msg.push_str(&format!("\n  {}", caret));
            }
        }
    }
    
    msg
}

/// Format a runtime error with context
pub fn format_runtime_error(error: &dyn std::error::Error) -> String {
    format!("Runtime error: {}", error)
}
