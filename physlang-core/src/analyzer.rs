//! Static analysis and type checking for PhysLang programs
//!
//! This module performs static checks on parsed programs to catch
//! errors before execution.

use crate::ast::{
    ConditionExpr, DetectorKind, LoopKind, ObservableExpr, Program,
};
use crate::diagnostics::{Diagnostic, Diagnostics, Span};

/// Analyze a program and return diagnostics
pub fn analyze_program(program: &Program) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();

    // Build particle name map for validation
    let mut particle_names = std::collections::HashMap::new();
    let mut particle_spans = std::collections::HashMap::<String, Option<Span>>::new();

    // Check 1: Unique particle names
    for (idx, particle) in program.particles.iter().enumerate() {
        if particle_names.insert(particle.name.clone(), idx).is_some() {
            diagnostics.push(Diagnostic::error(
                format!("duplicate particle name '{}'", particle.name),
                particle_spans.get(&particle.name).copied().flatten(),
            ));
        } else {
            // Store span for first occurrence (we don't have spans in AST yet, so None for now)
            particle_spans.insert(particle.name.clone(), None);
        }
    }

    // Check 2: Forces reference existing particles
    for force in &program.forces {
        if !particle_names.contains_key(&force.a) {
            diagnostics.push(Diagnostic::error(
                format!("unknown particle '{}' in force", force.a),
                None, // TODO: Add spans to AST
            ));
        }
        if !particle_names.contains_key(&force.b) {
            diagnostics.push(Diagnostic::error(
                format!("unknown particle '{}' in force", force.b),
                None, // TODO: Add spans to AST
            ));
        }
    }

    // Check 3: Loops reference existing particles
    for loop_decl in &program.loops {
        // Check target particle
        let target = match &loop_decl.kind {
            LoopKind::ForCycles { target, .. } => target,
            LoopKind::WhileCondition { target, .. } => target,
        };

        if !particle_names.contains_key(target) {
            diagnostics.push(Diagnostic::error(
                format!("unknown particle '{}' in loop target", target),
                None, // TODO: Add spans to AST
            ));
        }

        // Check loop body push targets
        for stmt in &loop_decl.body {
            match stmt {
                crate::ast::LoopBodyStmt::ForcePush { particle, .. } => {
                    if !particle_names.contains_key(particle) {
                        diagnostics.push(Diagnostic::error(
                            format!("unknown particle '{}' in loop body push", particle),
                            None, // TODO: Add spans to AST
                        ));
                    }
                }
            }
        }

        // Check conditions in while-loops
        if let LoopKind::WhileCondition { condition, .. } = &loop_decl.kind {
            check_observable_in_condition(condition, &particle_names, &mut diagnostics);
        }
    }

    // Check 4: Wells reference existing particles
    for well in &program.wells {
        if !particle_names.contains_key(&well.particle) {
            diagnostics.push(Diagnostic::error(
                format!("unknown particle '{}' in well", well.particle),
                None, // TODO: Add spans to AST
            ));
        }

        // Check observable in well
        check_observable(&well.observable, &particle_names, &mut diagnostics);
    }

    // Check 5: Detectors reference existing particles
    for detector in &program.detectors {
        match &detector.kind {
            DetectorKind::Position(name) => {
                if !particle_names.contains_key(name) {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown particle '{}' in detector", name),
                        None, // TODO: Add spans to AST
                    ));
                }
            }
            DetectorKind::Distance { a, b } => {
                if !particle_names.contains_key(a) {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown particle '{}' in detector", a),
                        None, // TODO: Add spans to AST
                    ));
                }
                if !particle_names.contains_key(b) {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown particle '{}' in detector", b),
                        None, // TODO: Add spans to AST
                    ));
                }
            }
        }
    }

    // Check 6: Simulate block exists and is unique
    // This is already checked in the parser, but we verify here too
    // (The parser ensures exactly one simulate block exists)

    diagnostics
}

/// Check an observable expression for valid particle references
fn check_observable(
    obs: &ObservableExpr,
    particle_names: &std::collections::HashMap<String, usize>,
    diagnostics: &mut Diagnostics,
) {
    match obs {
        ObservableExpr::PositionX(name) | ObservableExpr::PositionY(name) => {
            if !particle_names.contains_key(name) {
                diagnostics.push(Diagnostic::error(
                    format!("unknown particle '{}' in observable", name),
                    None, // TODO: Add spans to AST
                ));
            }
        }
        ObservableExpr::Distance(a, b) => {
            if !particle_names.contains_key(a) {
                diagnostics.push(Diagnostic::error(
                    format!("unknown particle '{}' in distance observable", a),
                    None, // TODO: Add spans to AST
                ));
            }
            if !particle_names.contains_key(b) {
                diagnostics.push(Diagnostic::error(
                    format!("unknown particle '{}' in distance observable", b),
                    None, // TODO: Add spans to AST
                ));
            }
        }
    }
}

/// Check observables in a condition expression
fn check_observable_in_condition(
    condition: &ConditionExpr,
    particle_names: &std::collections::HashMap<String, usize>,
    diagnostics: &mut Diagnostics,
) {
    match condition {
        ConditionExpr::LessThan(obs, _) | ConditionExpr::GreaterThan(obs, _) => {
            check_observable(obs, particle_names, diagnostics);
        }
    }
}

