//! Unit tests for loop iteration counts

use physlang_core::loops::{update_and_apply_loops, LoopBodyRuntime, LoopInstance, LoopKindRuntime};
use physlang_core::engine::Particle;
use glam::Vec2;

#[test]
fn test_for_loop_iteration_count() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 3,
            frequency: 10.0, // High frequency to complete cycles quickly
            damping: 0.0,
            phase: 0.0,
        },
        body: vec![LoopBodyRuntime::ForcePush {
            particle_index: 0,
            magnitude: 1.0,
            direction: Vec2::new(1.0, 0.0),
        }],
        active: true,
    };
    
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::ZERO,
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let dt = 0.01;
    let mut iterations = 0;
    
    // Run until loop deactivates
    while loop_inst.active {
        let initial_cycles = match &loop_inst.kind {
            LoopKindRuntime::ForCycles { cycles_remaining, .. } => *cycles_remaining,
            _ => panic!("Expected ForCycles"),
        };
        
        update_and_apply_loops(&mut [&mut loop_inst], &mut particles, dt);
        
        let new_cycles = match &loop_inst.kind {
            LoopKindRuntime::ForCycles { cycles_remaining, .. } => *cycles_remaining,
            _ => panic!("Expected ForCycles"),
        };
        
        if new_cycles < initial_cycles {
            iterations += 1;
        }
        
        // Safety limit
        if iterations > 100 {
            break;
        }
    }
    
    // Should have iterated 3 times
    assert_eq!(iterations, 3);
    assert!(!loop_inst.active);
}

#[test]
fn test_for_loop_body_triggers_once_per_cycle() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 2,
            frequency: 10.0,
            damping: 0.0,
            phase: 0.0,
        },
        body: vec![LoopBodyRuntime::ForcePush {
            particle_index: 0,
            magnitude: 1.0,
            direction: Vec2::new(1.0, 0.0),
        }],
        active: true,
    };
    
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::ZERO,
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let dt = 0.01;
    let initial_vel = particles[0].vel.x;
    
    // Run one complete cycle
    let mut steps = 0;
    while loop_inst.active && steps < 1000 {
        update_and_apply_loops(&mut [&mut loop_inst], &mut particles, dt);
        steps += 1;
    }
    
    // Velocity should have increased by magnitude * cycles
    // Each cycle applies push once, so 2 cycles = 2 pushes
    let expected_vel = initial_vel + 2.0; // 2 cycles * 1.0 magnitude
    assert!((particles[0].vel.x - expected_vel).abs() < 0.1, 
        "Expected velocity ~{}, got {}", expected_vel, particles[0].vel.x);
}

#[test]
fn test_while_loop_stops_when_condition_false() {
    use physlang_core::loops::{ConditionRuntime, ObservableRuntime};
    
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::WhileCondition {
            target_index: 0,
            condition: ConditionRuntime::LessThan(
                ObservableRuntime::PositionX(0),
                5.0,
            ),
            frequency: 10.0,
            damping: 0.0,
            phase: 0.0,
        },
        body: vec![LoopBodyRuntime::ForcePush {
            particle_index: 0,
            magnitude: 0.5,
            direction: Vec2::new(1.0, 0.0),
        }],
        active: true,
    };
    
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(0.0, 0.0),
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let dt = 0.01;
    
    // Run simulation steps
    for _ in 0..10000 {
        if !loop_inst.active {
            break;
        }
        update_and_apply_loops(&mut [&mut loop_inst], &mut particles, dt);
        
        // Also update position (simplified)
        particles[0].pos += particles[0].vel * dt;
        
        // Check condition manually
        use physlang_core::loops::evaluate_loop_conditions;
        evaluate_loop_conditions(&mut [&mut loop_inst], &particles);
    }
    
    // Loop should eventually deactivate when position.x >= 5.0
    assert!(!loop_inst.active || particles[0].pos.x >= 4.9);
}

#[test]
fn test_inactive_loop_does_not_iterate() {
    let mut loop_inst = LoopInstance {
        kind: LoopKindRuntime::ForCycles {
            target_index: 0,
            cycles_remaining: 5,
            frequency: 1.0,
            damping: 0.0,
            phase: 0.0,
        },
        body: vec![LoopBodyRuntime::ForcePush {
            particle_index: 0,
            magnitude: 1.0,
            direction: Vec2::new(1.0, 0.0),
        }],
        active: false, // Inactive
    };
    
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::ZERO,
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let initial_vel = particles[0].vel;
    
    // Update loop (should do nothing)
    update_and_apply_loops(&mut [&mut loop_inst], &mut particles, 0.1);
    
    // Velocity should be unchanged
    assert_eq!(particles[0].vel, initial_vel);
}

