//! Runtime structures and logic for loops and wells (v0.2+)

use crate::engine::Particle;
use glam::Vec2;
use std::f32::consts::PI;

/// Runtime loop instance (resolved indices instead of names)
#[derive(Debug)]
pub struct LoopInstance {
    pub kind: LoopKindRuntime,
    pub body: Vec<LoopBodyRuntime>,
    pub active: bool,
}

/// Runtime loop kind
#[derive(Debug)]
pub enum LoopKindRuntime {
    ForCycles {
        target_index: usize,
        cycles_remaining: u32,
        frequency: f32,
        damping: f32,
        phase: f32,
    },
    WhileCondition {
        target_index: usize,
        condition: ConditionRuntime,
        frequency: f32,
        damping: f32,
        phase: f32,
    },
}

/// Runtime condition
#[derive(Debug, Clone)]
pub enum ConditionRuntime {
    LessThan(ObservableRuntime, f32),
    GreaterThan(ObservableRuntime, f32),
}

/// Runtime observable
#[derive(Debug, Clone)]
pub enum ObservableRuntime {
    PositionX(usize),
    PositionY(usize),
    Distance(usize, usize),
}

/// Runtime loop body action
#[derive(Debug, Clone)]
pub enum LoopBodyRuntime {
    ForcePush {
        particle_index: usize,
        magnitude: f32,
        direction: Vec2,
    },
}

/// Potential well instance
#[derive(Debug)]
pub struct WellInstance {
    pub particle_index: usize,
    pub observable: ObservableRuntime,
    pub threshold: f32,
    pub depth: f32,
}

/// Update loops and apply loop body actions
pub fn update_and_apply_loops(
    loops: &mut [LoopInstance],
    particles: &mut [Particle],
    dt: f32,
) {
    for loop_inst in loops.iter_mut() {
        if !loop_inst.active {
            continue;
        }

        match &mut loop_inst.kind {
            LoopKindRuntime::ForCycles {
                cycles_remaining,
                frequency,
                damping,
                phase,
                ..
            } => {
                // Advance phase
                *phase += 2.0 * PI * (*frequency) * dt;
                *phase *= (1.0 - (*damping) * dt).max(0.0);

                // Check for phase wrap (one cycle completed)
                if *phase >= 2.0 * PI {
                    *phase -= 2.0 * PI;

                    // Apply loop body
                    apply_loop_body(&loop_inst.body, particles);

                    // Decrement cycles
                    if *cycles_remaining > 0 {
                        *cycles_remaining -= 1;
                    }

                    // Deactivate if cycles exhausted
                    if *cycles_remaining == 0 {
                        loop_inst.active = false;
                    }
                }
            }
            LoopKindRuntime::WhileCondition {
                condition,
                frequency,
                damping,
                phase,
                ..
            } => {
                // Advance phase
                *phase += 2.0 * PI * (*frequency) * dt;
                *phase *= (1.0 - (*damping) * dt).max(0.0);

                // Check for phase wrap
                if *phase >= 2.0 * PI {
                    *phase -= 2.0 * PI;

                    // Evaluate condition BEFORE applying body
                    let condition_met = evaluate_condition(condition, particles);

                    if condition_met {
                        // Apply loop body
                        apply_loop_body(&loop_inst.body, particles);
                    } else {
                        // Condition false, deactivate loop
                        loop_inst.active = false;
                    }
                }
            }
        }
    }
}

/// Apply loop body actions to particles
fn apply_loop_body(body: &[LoopBodyRuntime], particles: &mut [Particle]) {
    for action in body {
        match action {
            LoopBodyRuntime::ForcePush {
                particle_index,
                magnitude,
                direction,
            } => {
                let particle = &mut particles[*particle_index];
                let dir_normalized = direction.normalize_or_zero();
                // Apply impulse: directly modify velocity
                particle.vel += dir_normalized * (*magnitude);
            }
        }
    }
}

/// Evaluate a condition using current world state
fn evaluate_condition(condition: &ConditionRuntime, particles: &[Particle]) -> bool {
    match condition {
        ConditionRuntime::LessThan(obs, threshold) => {
            let value = evaluate_observable(obs, particles);
            value < *threshold
        }
        ConditionRuntime::GreaterThan(obs, threshold) => {
            let value = evaluate_observable(obs, particles);
            value > *threshold
        }
    }
}

/// Evaluate an observable expression
fn evaluate_observable(obs: &ObservableRuntime, particles: &[Particle]) -> f32 {
    match obs {
        ObservableRuntime::PositionX(idx) => particles[*idx].pos.x,
        ObservableRuntime::PositionY(idx) => particles[*idx].pos.y,
        ObservableRuntime::Distance(a_idx, b_idx) => {
            particles[*a_idx].pos.distance(particles[*b_idx].pos)
        }
    }
}

/// Apply potential wells as forces
pub fn apply_wells(wells: &[WellInstance], particles: &mut [Particle], dt: f32) {
    for well in wells {
        // Evaluate observable first (before mutable borrow)
        let value = match &well.observable {
            ObservableRuntime::PositionX(idx) => particles[*idx].pos.x,
            ObservableRuntime::PositionY(idx) => particles[*idx].pos.y,
            ObservableRuntime::Distance(a_idx, b_idx) => {
                particles[*a_idx].pos.distance(particles[*b_idx].pos)
            }
        };
        
        let particle = &mut particles[well.particle_index];

        // If value >= threshold, apply well force
        if value >= well.threshold {
            // Apply spring-like force pulling towards threshold
            // For PositionX: force = -depth * (x - threshold)
            match &well.observable {
                ObservableRuntime::PositionX(_) => {
                    let displacement = particle.pos.x - well.threshold;
                    let force = -well.depth * displacement;
                    // Apply as acceleration: a = F/m, then v += a*dt
                    let accel = force / particle.mass;
                    particle.vel.x += accel * dt;
                }
                ObservableRuntime::PositionY(_) => {
                    let displacement = particle.pos.y - well.threshold;
                    let force = -well.depth * displacement;
                    let accel = force / particle.mass;
                    particle.vel.y += accel * dt;
                }
                ObservableRuntime::Distance(_, _) => {
                    // For distance wells, we'd need to compute direction
                    // v0.2: skip distance wells for simplicity
                }
            }
        }
    }
}

/// Evaluate while-loop conditions to deactivate finished loops
pub fn evaluate_loop_conditions(loops: &mut [LoopInstance], particles: &[Particle]) {
    for loop_inst in loops.iter_mut() {
        if !loop_inst.active {
            continue;
        }

        // Only check while-loops (for-loops are handled in update_and_apply_loops)
        if let LoopKindRuntime::WhileCondition { condition, .. } = &loop_inst.kind {
            let condition_met = evaluate_condition(condition, particles);
            if !condition_met {
                loop_inst.active = false;
            }
        }
    }
}

