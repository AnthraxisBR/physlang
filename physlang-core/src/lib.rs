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
pub use engine::{Force, Particle, World};
pub use parser::parse_program;
pub use runtime::{
    build_simulation_context, build_simulation_context_from_source, get_particle_states,
    run_program, step_simulation, DetectorResult, ParticleState, SimulationContext,
    SimulationResult,
};

// Test helpers module (public for integration tests)
// Always compiled - integration tests are separate crates and need access
pub mod tests;
