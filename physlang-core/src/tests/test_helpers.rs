//! Test helper utilities for PhysLang tests

use std::fs;
use std::path::Path;

/// Check if two floating point values are approximately equal within tolerance
pub fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// Check if two f32 values are approximately equal within tolerance
pub fn approx_eq_f32(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

/// Load expected output from a file
pub fn load_expected(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(path)?)
}

/// Write expected output to a file (for initial generation)
pub fn write_expected(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, content)?)
}

/// Run a PhysLang program from a file path
pub fn run_phys_file(file: &str) -> Result<crate::runtime::SimulationResult, Box<dyn std::error::Error>> {
    let src = fs::read_to_string(file)?;
    crate::run_program(&src)
}

/// Run a PhysLang program from source string
pub fn run_phys_source(source: &str) -> Result<crate::runtime::SimulationResult, Box<dyn std::error::Error>> {
    crate::run_program(source)
}

/// Convert simulation result to JSON string for golden tests
pub fn result_to_json(result: &crate::runtime::SimulationResult) -> String {
    use std::fmt::Write;
    
    let mut json = String::from("{\n  \"detectors\": [\n");
    for (i, detector) in result.detectors.iter().enumerate() {
        if i > 0 {
            json.push_str(",\n");
        }
        write!(json, "    {{\"name\": \"{}\", \"value\": {:.12}}}", detector.name, detector.value).unwrap();
    }
    json.push_str("\n  ]\n}");
    json
}

/// Compare two simulation results with tolerance
pub fn results_approx_equal(
    a: &crate::runtime::SimulationResult,
    b: &crate::runtime::SimulationResult,
    tol: f32,
) -> bool {
    if a.detectors.len() != b.detectors.len() {
        return false;
    }
    
    // Sort by name for comparison
    let mut a_sorted: Vec<_> = a.detectors.iter().collect();
    let mut b_sorted: Vec<_> = b.detectors.iter().collect();
    a_sorted.sort_by_key(|d| &d.name);
    b_sorted.sort_by_key(|d| &d.name);
    
    for (a_det, b_det) in a_sorted.iter().zip(b_sorted.iter()) {
        if a_det.name != b_det.name {
            return false;
        }
        if !approx_eq_f32(a_det.value, b_det.value, tol) {
            return false;
        }
    }
    
    true
}

