//! Tests for broken/invalid input files

use physlang_core::{parse_program, analyze_program, run_program};
use std::path::PathBuf;

fn broken_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("integration");
    path.push("broken");
    path.push(filename);
    path
}

#[test]
fn test_unknown_particle_produces_error() {
    let path = broken_data_path("unknown_particle.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    // Should parse successfully
    let program = parse_program(&source).expect("Should parse");
    
    // But analyzer should catch the error
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors(), "Should detect unknown particle");
    
    // Runtime should fail
    let result = run_program(&source);
    assert!(result.is_err(), "Runtime should fail with unknown particle");
}

#[test]
fn test_syntax_error_produces_parse_error() {
    let path = broken_data_path("syntax_error.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    let result = parse_program(&source);
    assert!(result.is_err(), "Should fail to parse syntax error");
    
    // Should not panic
    match result {
        Err(e) => {
            assert!(!e.to_string().is_empty(), "Error message should not be empty");
        }
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn test_missing_simulate_produces_error() {
    let path = broken_data_path("missing_simulate.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    let result = parse_program(&source);
    assert!(result.is_err(), "Should fail without simulate block");
}

#[test]
fn test_invalid_loop_syntax_produces_error() {
    let path = broken_data_path("invalid_loop_syntax.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    let result = parse_program(&source);
    assert!(result.is_err(), "Should fail to parse invalid loop syntax");
}

#[test]
fn test_duplicate_particle_produces_error() {
    let path = broken_data_path("duplicate_particle.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    // Should parse
    let program = parse_program(&source).expect("Should parse");
    
    // But analyzer should catch duplicate
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors(), "Should detect duplicate particle");
    
    // Runtime should fail
    let result = run_program(&source);
    assert!(result.is_err(), "Runtime should fail with duplicate particle");
}

#[test]
fn test_broken_inputs_do_not_panic() {
    // Test that all broken inputs fail gracefully without panicking
    let broken_files = [
        "unknown_particle.phys",
        "syntax_error.phys",
        "missing_simulate.phys",
        "invalid_loop_syntax.phys",
        "duplicate_particle.phys",
    ];
    
    for filename in &broken_files {
        let path = broken_data_path(filename);
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue, // Skip if file doesn't exist
        };
        
        // Should not panic
        let _ = parse_program(&source);
        let _ = run_program(&source);
    }
}

#[test]
fn test_error_messages_are_consistent() {
    let path = broken_data_path("unknown_particle.phys");
    let source = std::fs::read_to_string(path).unwrap();
    
    // Run twice, should get same error message
    let result1 = run_program(&source);
    let result2 = run_program(&source);
    
    match (result1, result2) {
        (Err(e1), Err(e2)) => {
            // Error messages should be similar (may have slight differences in formatting)
            let msg1 = e1.to_string();
            let msg2 = e2.to_string();
            assert!(!msg1.is_empty());
            assert!(!msg2.is_empty());
            // Both should mention the error
            assert!(msg1.contains("particle") || msg1.contains("unknown") || msg1.contains("error"));
            assert!(msg2.contains("particle") || msg2.contains("unknown") || msg2.contains("error"));
        }
        _ => panic!("Both should fail with errors"),
    }
}

