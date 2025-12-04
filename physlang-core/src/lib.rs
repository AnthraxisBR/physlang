pub mod analyzer;
pub mod ast;
pub mod diagnostics;
pub mod engine;
pub mod integrator;
pub mod loops;
pub mod parser;
pub mod runtime;

pub use analyzer::analyze_program;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics, SourceLocation, Span};
pub use parser::parse_program;
pub use runtime::{
    build_simulation_context, build_simulation_context_from_source, get_particle_states,
    run_program, step_simulation, DetectorResult, ParticleState, SimulationContext,
    SimulationResult,
};

// Test helpers module (only compiled in test mode)
#[cfg(test)]
pub mod tests;
