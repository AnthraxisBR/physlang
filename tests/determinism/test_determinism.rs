//! Determinism tests - ensure same program produces identical outputs

use physlang_core::tests::test_helpers::{run_phys_source, results_approx_equal};
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("integration");
    path.push("data");
    path.push(filename);
    path
}

#[test]
fn test_oscillator_determinism() {
    let path = test_data_path("spring_oscillator.phys");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    
    // Run simulation twice
    let result1 = run_phys_source(&source).expect("First run failed");
    let result2 = run_phys_source(&source).expect("Second run failed");
    
    // Results should be bit-equal or within very tight tolerance
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "Running same program twice should produce identical results");
}

#[test]
fn test_gravity_determinism() {
    let path = test_data_path("simple_gravity.phys");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    
    let result1 = run_phys_source(&source).expect("First run failed");
    let result2 = run_phys_source(&source).expect("Second run failed");
    
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "Gravity simulation should be deterministic");
}

#[test]
fn test_loop_determinism() {
    let path = test_data_path("loop_push.phys");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    
    let result1 = run_phys_source(&source).expect("First run failed");
    let result2 = run_phys_source(&source).expect("Second run failed");
    
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "Loop simulation should be deterministic");
}

#[test]
fn test_while_loop_determinism() {
    let path = test_data_path("while_loop_stop.phys");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    
    let result1 = run_phys_source(&source).expect("First run failed");
    let result2 = run_phys_source(&source).expect("Second run failed");
    
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "While-loop simulation should be deterministic");
}

#[test]
fn test_graph_layout_determinism() {
    let path = test_data_path("graph_layout.phys");
    let source = std::fs::read_to_string(path).expect("Failed to read file");
    
    let result1 = run_phys_source(&source).expect("First run failed");
    let result2 = run_phys_source(&source).expect("Second run failed");
    
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "Graph layout simulation should be deterministic");
}

#[test]
fn test_multiple_runs_determinism() {
    // Run same program multiple times and ensure all results match
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (3.0, 0.0) mass 1.0
force spring(a, b) k = 10.0 rest = 2.0
simulate dt = 0.01 steps = 1000
detect dist_ab = distance(a, b)
"#;
    
    let results: Vec<_> = (0..5)
        .map(|_| run_phys_source(source).expect("Run failed"))
        .collect();
    
    // All results should match
    for i in 1..results.len() {
        assert!(results_approx_equal(&results[0], &results[i], 1e-12),
            "Run {} should match run 0", i);
    }
}

#[test]
fn test_determinism_with_loops_and_wells() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well target on a if position(a).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 2000
loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
detect a_x = position(a)
"#;
    
    let result1 = run_phys_source(source).expect("First run failed");
    let result2 = run_phys_source(source).expect("Second run failed");
    
    assert!(results_approx_equal(&result1, &result2, 1e-12),
        "Simulation with loops and wells should be deterministic");
}

