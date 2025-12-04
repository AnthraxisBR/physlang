use crate::ast::{DetectorKind, ForceKind, Program};
use crate::engine::{Force, Particle, World};
use crate::integrator::step;
use crate::parser::parse_program;
use glam::Vec2;
use std::collections::HashMap;

/// Result of a detector evaluation
#[derive(Debug, Clone)]
pub struct DetectorResult {
    pub name: String,
    pub value: f32,
}

/// Final result of running a program
#[derive(Debug)]
pub struct SimulationResult {
    pub detectors: Vec<DetectorResult>,
}

/// Main entry point: parse and run a PhysLang program
pub fn run_program(source: &str) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let program = parse_program(source)?;
    let mut world = build_world(&program)?;

    // Run the simulation
    for _ in 0..program.simulate.steps {
        step(&mut world, program.simulate.dt);
    }

    // Evaluate detectors
    let detectors = evaluate_detectors(&program, &world)?;

    Ok(SimulationResult { detectors })
}

/// Build a World from a parsed Program
pub fn build_world(program: &Program) -> Result<World, Box<dyn std::error::Error>> {
    let mut world = World::new();
    let mut name_to_idx: HashMap<String, usize> = HashMap::new();

    // Add particles
    for particle_decl in &program.particles {
        let idx = world.particles.len();
        name_to_idx.insert(particle_decl.name.clone(), idx);
        world.particles.push(Particle {
            name: particle_decl.name.clone(),
            pos: particle_decl.position,
            vel: Vec2::ZERO,
            mass: particle_decl.mass,
        });
    }

    // Add forces
    for force_decl in &program.forces {
        let a_idx = name_to_idx
            .get(&force_decl.a)
            .ok_or_else(|| format!("Particle '{}' not found", force_decl.a))?;
        let b_idx = name_to_idx
            .get(&force_decl.b)
            .ok_or_else(|| format!("Particle '{}' not found", force_decl.b))?;

        let force = match &force_decl.kind {
            ForceKind::Gravity { g } => Force::Gravity {
                a: *a_idx,
                b: *b_idx,
                g: *g,
            },
            ForceKind::Spring { k, rest } => Force::Spring {
                a: *a_idx,
                b: *b_idx,
                k: *k,
                rest: *rest,
            },
        };

        world.forces.push(force);
    }

    Ok(world)
}

/// Evaluate all detectors on the final world state
pub fn evaluate_detectors(
    program: &Program,
    world: &World,
) -> Result<Vec<DetectorResult>, Box<dyn std::error::Error>> {
    let name_to_particle: HashMap<String, &Particle> = world
        .particles
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    let mut results = Vec::new();

    for detector in &program.detectors {
        let value = match &detector.kind {
            DetectorKind::Position(name) => {
                let particle = name_to_particle
                    .get(name)
                    .ok_or_else(|| format!("Particle '{}' not found for detector", name))?;
                // For position, we return the x coordinate
                // In the future, we might want to support x, y separately
                particle.pos.x
            }
            DetectorKind::Distance { a, b } => {
                let particle_a = name_to_particle
                    .get(a)
                    .ok_or_else(|| format!("Particle '{}' not found for detector", a))?;
                let particle_b = name_to_particle
                    .get(b)
                    .ok_or_else(|| format!("Particle '{}' not found for detector", b))?;
                particle_a.pos.distance(particle_b.pos)
            }
        };

        results.push(DetectorResult {
            name: detector.name.clone(),
            value,
        });
    }

    Ok(results)
}
