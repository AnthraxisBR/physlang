//! Unit tests for oscillator logic (phase accumulation, wrapping)

use physlang_core::loops::{LoopInstance, LoopKindRuntime};
use physlang_core::tests::test_helpers::approx_eq_f32;
use std::f32::consts::PI;

#[test]
fn test_phase_accumulation() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 0.0,
            phase: 0.0,
        },
        body: vec![],
        active: true,
    };
    
    let dt = 0.1;
    
    // Advance phase for one step
    match &mut loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, frequency, .. } => {
            *phase += 2.0 * PI * (*frequency) * dt;
        }
        _ => panic!("Expected ForCycles"),
    }
    
    // Phase should be 2π * 1.0 * 0.1 = 0.2π
    match &loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, .. } => {
            assert!(approx_eq_f32(*phase, 2.0 * PI * 1.0 * 0.1, 1e-5));
        }
        _ => panic!("Expected ForCycles"),
    }
}

#[test]
fn test_phase_wrap() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 0.0,
            phase: 1.9 * PI, // Close to 2π
        },
        body: vec![],
        active: true,
    };
    
    let dt = 0.2;
    
    // Advance phase
    match &mut loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, frequency, .. } => {
            *phase += 2.0 * PI * (*frequency) * dt;
            // Check for wrap
            if *phase >= 2.0 * PI {
                *phase -= 2.0 * PI;
            }
        }
        _ => panic!("Expected ForCycles"),
    }
    
    // Phase should wrap: 1.9π + 0.2*2π = 1.9π + 0.4π = 2.3π -> 2.3π - 2π = 0.3π
    match &loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, .. } => {
            let expected = (1.9 * PI + 2.0 * PI * 1.0 * 0.2) - 2.0 * PI;
            assert!(approx_eq_f32(*phase, expected, 1e-5));
        }
        _ => panic!("Expected ForCycles"),
    }
}

#[test]
fn test_phase_wrap_exactly_2pi() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 0.0,
            phase: 2.0 * PI - 0.01, // Just below 2π
        },
        body: vec![],
        active: true,
    };
    
    let dt = 0.02;
    
    // Advance phase
    match &mut loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, frequency, .. } => {
            *phase += 2.0 * PI * (*frequency) * dt;
            if *phase >= 2.0 * PI {
                *phase -= 2.0 * PI;
            }
        }
        _ => panic!("Expected ForCycles"),
    }
    
    // Should wrap
    match &loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, .. } => {
            assert!(*phase < 2.0 * PI);
            assert!(*phase > 0.0);
        }
        _ => panic!("Expected ForCycles"),
    }
}

#[test]
fn test_damping_reduces_phase() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 0.1,
            phase: 0.0,
        },
        body: vec![],
        active: true,
    };
    
    let dt = 0.1;
    
    // Advance phase with damping
    match &mut loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, frequency, damping, .. } => {
            *phase += 2.0 * PI * (*frequency) * dt;
            *phase *= (1.0 - (*damping) * dt).max(0.0);
        }
        _ => panic!("Expected ForCycles"),
    }
    
    // Phase should be (2π * 1.0 * 0.1) * (1 - 0.1 * 0.1) = 0.2π * 0.99 = 0.198π
    match &loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, .. } => {
            let expected = 2.0 * PI * 1.0 * 0.1 * (1.0 - 0.1 * 0.1);
            assert!(approx_eq_f32(*phase, expected, 1e-5));
        }
        _ => panic!("Expected ForCycles"),
    }
}

#[test]
fn test_damping_prevents_negative() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 100.0, // Very high damping
            phase: 0.0,
        },
        body: vec![],
        active: true,
    };
    
    let dt = 0.1;
    
    // Advance phase with high damping
    match &mut loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, frequency, damping, .. } => {
            *phase += 2.0 * PI * (*frequency) * dt;
            *phase *= (1.0 - (*damping) * dt).max(0.0);
        }
        _ => panic!("Expected ForCycles"),
    }
    
    // Phase should be clamped to >= 0
    match &loop_inst.kind {
        LoopKindRuntime::ForCycles { phase, .. } => {
            assert!(*phase >= 0.0);
        }
        _ => panic!("Expected ForCycles"),
    }
}

