use crate::analyzer::analyze_program;
use crate::ast::{
    ConditionExpr, DetectorKind, ForceKind, LoopKind, ObservableExpr, Program,
};
use crate::engine::{Force, Particle, World};
use crate::eval::{eval_expr, evaluate_lets, EvalContext};
use crate::functions::execute_functions;
use crate::integrator::step;
use crate::loops::{
    apply_wells, evaluate_loop_conditions, update_and_apply_loops, ConditionRuntime,
    LoopBodyRuntime, LoopInstance, LoopKindRuntime, ObservableRuntime, WellInstance,
};
use crate::parser::parse_program;
use crate::diagnostics::Diagnostics;
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

/// Simulation context containing world, loops, and wells
pub struct SimulationContext {
    pub world: World,
    pub loops: Vec<LoopInstance>,
    pub wells: Vec<WellInstance>,
    pub dt: f32,
    pub max_steps: usize,
    pub current_step: usize,
}

/// Main entry point: parse and run a PhysLang program
pub fn run_program(source: &str) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let mut program = parse_program(source)?;
    
    // Perform static analysis
    let diagnostics = analyze_program(&program);
    if diagnostics.has_errors() {
        let error_messages: Vec<String> = diagnostics
            .errors()
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Static analysis errors:\n{}", error_messages.join("\n")).into());
    }
    
    // Evaluate let bindings (borrow ends here)
    let lets = program.lets.clone();
    let (eval_ctx, eval_diagnostics) = evaluate_lets(&lets);
    if eval_diagnostics.iter().any(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Error)) {
        let error_messages: Vec<String> = eval_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Error))
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Expression evaluation errors:\n{}", error_messages.join("\n")).into());
    }
    
    // Execute functions to generate world-building statements
    let func_diagnostics = execute_functions(&mut program, &eval_ctx);
    if func_diagnostics.iter().any(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Error)) {
        let error_messages: Vec<String> = func_diagnostics
            .iter()
            .filter(|d| matches!(d.severity, crate::diagnostics::DiagnosticSeverity::Error))
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Function execution errors:\n{}", error_messages.join("\n")).into());
    }
    
    // Re-analyze program after function execution to validate generated world
    let post_func_diagnostics = analyze_program(&program);
    if post_func_diagnostics.has_errors() {
        let error_messages: Vec<String> = post_func_diagnostics
            .errors()
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Post-function analysis errors:\n{}", error_messages.join("\n")).into());
    }
    
    let mut ctx = build_simulation_context(&program, &eval_ctx)?;

    // Evaluate steps expression
    let steps_value = eval_expr(&program.simulate.steps, &eval_ctx)
        .map_err(|e| format!("Error evaluating steps: {}", e))?;
    let steps_usize = steps_value as usize;
    if steps_value < 1.0 || steps_value != steps_usize as f32 {
        return Err(format!(
            "steps must be an integer >= 1, got {}",
            steps_value
        )
        .into());
    }
    
    // Evaluate dt expression
    let dt_value = eval_expr(&program.simulate.dt, &eval_ctx)
        .map_err(|e| format!("Error evaluating dt: {}", e))?;
    
    // Run the simulation
    for _ in 0..steps_usize {
        // 1. Update loops (advance oscillators, fire iterations)
        update_and_apply_loops(&mut ctx.loops, &mut ctx.world.particles, dt_value);

        // 2. Apply wells (convert wells into forces/accelerations)
        apply_wells(&ctx.wells, &mut ctx.world.particles, dt_value);

        // 3. Integrate physics
        step(&mut ctx.world, dt_value);

        // 4. Evaluate while-loop conditions to deactivate finished loops
        evaluate_loop_conditions(&mut ctx.loops, &ctx.world.particles);
    }

    // Evaluate detectors
    let detectors = evaluate_detectors(&program, &ctx.world)?;

    Ok(SimulationResult { detectors })
}

/// Build simulation context from a parsed Program
pub fn build_simulation_context(
    program: &Program,
    eval_ctx: &EvalContext<'_>,
) -> Result<SimulationContext, Box<dyn std::error::Error>> {
    let mut world = World::new();
    let mut name_to_idx: HashMap<String, usize> = HashMap::new();

    // Add particles
    for particle_decl in &program.particles {
        let idx = world.particles.len();
        name_to_idx.insert(particle_decl.name.clone(), idx);
        
        // Evaluate position expressions
        let x = eval_expr(&particle_decl.position.0, eval_ctx)
            .map_err(|e| format!("Error evaluating particle {} x position: {}", particle_decl.name, e))?;
        let y = eval_expr(&particle_decl.position.1, eval_ctx)
            .map_err(|e| format!("Error evaluating particle {} y position: {}", particle_decl.name, e))?;
        
        // Evaluate mass expression
        let mass = eval_expr(&particle_decl.mass, eval_ctx)
            .map_err(|e| format!("Error evaluating particle {} mass: {}", particle_decl.name, e))?;
        
        world.particles.push(Particle {
            name: particle_decl.name.clone(),
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            mass,
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
            ForceKind::Gravity { g } => {
                let g_value = eval_expr(g, eval_ctx)
                    .map_err(|e| format!("Error evaluating gravity G: {}", e))?;
                Force::Gravity {
                    a: *a_idx,
                    b: *b_idx,
                    g: g_value,
                }
            }
            ForceKind::Spring { k, rest } => {
                let k_value = eval_expr(k, eval_ctx)
                    .map_err(|e| format!("Error evaluating spring k: {}", e))?;
                let rest_value = eval_expr(rest, eval_ctx)
                    .map_err(|e| format!("Error evaluating spring rest: {}", e))?;
                Force::Spring {
                    a: *a_idx,
                    b: *b_idx,
                    k: k_value,
                    rest: rest_value,
                }
            }
        };

        world.forces.push(force);
    }

    // Build loops
    let loops = build_loops(&program.loops, &name_to_idx, eval_ctx)?;

    // Build wells
    let wells = build_wells(&program.wells, &name_to_idx, eval_ctx)?;

    // Evaluate dt and steps
    let dt_value = eval_expr(&program.simulate.dt, eval_ctx)
        .map_err(|e| format!("Error evaluating dt: {}", e))?;
    let steps_value = eval_expr(&program.simulate.steps, eval_ctx)
        .map_err(|e| format!("Error evaluating steps: {}", e))?;
    let steps_usize = steps_value as usize;
    if steps_value < 1.0 || steps_value != steps_usize as f32 {
        return Err(format!(
            "steps must be an integer >= 1, got {}",
            steps_value
        )
        .into());
    }

    Ok(SimulationContext {
        world,
        loops,
        wells,
        dt: dt_value,
        max_steps: steps_usize,
        current_step: 0,
    })
}

/// Build runtime loops from AST loops
fn build_loops(
    loop_decls: &[crate::ast::LoopDecl],
    name_to_idx: &HashMap<String, usize>,
    eval_ctx: &EvalContext<'_>,
) -> Result<Vec<LoopInstance>, Box<dyn std::error::Error>> {
    let mut loops = Vec::new();

    for loop_decl in loop_decls {
        let kind = match &loop_decl.kind {
            LoopKind::ForCycles {
                cycles,
                frequency,
                damping,
                target,
            } => {
                let target_idx = name_to_idx
                    .get(target)
                    .ok_or_else(|| format!("Particle '{}' not found for loop", target))?;
                
                // Evaluate expressions
                let cycles_value = eval_expr(cycles, eval_ctx)
                    .map_err(|e| format!("Error evaluating cycles: {}", e))?;
                let cycles_u32 = cycles_value as u32;
                if cycles_value < 0.0 || cycles_value != cycles_u32 as f32 {
                    return Err(format!(
                        "cycles must be an integer >= 0, got {}",
                        cycles_value
                    )
                    .into());
                }
                
                let frequency_value = eval_expr(frequency, eval_ctx)
                    .map_err(|e| format!("Error evaluating frequency: {}", e))?;
                let damping_value = eval_expr(damping, eval_ctx)
                    .map_err(|e| format!("Error evaluating damping: {}", e))?;
                
                LoopKindRuntime::ForCycles {
                    target_index: *target_idx,
                    cycles_remaining: cycles_u32,
                    frequency: frequency_value,
                    damping: damping_value,
                    phase: 0.0,
                }
            }
            LoopKind::WhileCondition {
                condition,
                frequency,
                damping,
                target,
            } => {
                let target_idx = name_to_idx
                    .get(target)
                    .ok_or_else(|| format!("Particle '{}' not found for loop", target))?;
                
                // Evaluate expressions
                let frequency_value = eval_expr(frequency, eval_ctx)
                    .map_err(|e| format!("Error evaluating frequency: {}", e))?;
                let damping_value = eval_expr(damping, eval_ctx)
                    .map_err(|e| format!("Error evaluating damping: {}", e))?;
                
                LoopKindRuntime::WhileCondition {
                    target_index: *target_idx,
                    condition: convert_condition(condition, name_to_idx, eval_ctx)?,
                    frequency: frequency_value,
                    damping: damping_value,
                    phase: 0.0,
                }
            }
        };

        let body = loop_decl
            .body
            .iter()
            .map(|stmt| convert_loop_body_stmt(stmt, name_to_idx, eval_ctx))
            .collect::<Result<Vec<_>, _>>()?;

        loops.push(LoopInstance {
            kind,
            body,
            active: true,
        });
    }

    Ok(loops)
}

/// Convert AST condition to runtime condition
fn convert_condition(
    condition: &ConditionExpr,
    name_to_idx: &HashMap<String, usize>,
    eval_ctx: &EvalContext<'_>,
) -> Result<ConditionRuntime, Box<dyn std::error::Error>> {
    match condition {
        ConditionExpr::LessThan(obs, threshold) => {
            let threshold_value = eval_expr(threshold, eval_ctx)
                .map_err(|e| format!("Error evaluating condition threshold: {}", e))?;
            Ok(ConditionRuntime::LessThan(
                convert_observable(obs, name_to_idx)?,
                threshold_value,
            ))
        }
        ConditionExpr::GreaterThan(obs, threshold) => {
            let threshold_value = eval_expr(threshold, eval_ctx)
                .map_err(|e| format!("Error evaluating condition threshold: {}", e))?;
            Ok(ConditionRuntime::GreaterThan(
                convert_observable(obs, name_to_idx)?,
                threshold_value,
            ))
        }
    }
}

/// Convert AST observable to runtime observable
fn convert_observable(
    obs: &ObservableExpr,
    name_to_idx: &HashMap<String, usize>,
) -> Result<ObservableRuntime, Box<dyn std::error::Error>> {
    match obs {
        ObservableExpr::PositionX(name) => {
            let idx = name_to_idx
                .get(name)
                .ok_or_else(|| format!("Particle '{}' not found", name))?;
            Ok(ObservableRuntime::PositionX(*idx))
        }
        ObservableExpr::PositionY(name) => {
            let idx = name_to_idx
                .get(name)
                .ok_or_else(|| format!("Particle '{}' not found", name))?;
            Ok(ObservableRuntime::PositionY(*idx))
        }
        ObservableExpr::Distance(a, b) => {
            let a_idx = name_to_idx
                .get(a)
                .ok_or_else(|| format!("Particle '{}' not found", a))?;
            let b_idx = name_to_idx
                .get(b)
                .ok_or_else(|| format!("Particle '{}' not found", b))?;
            Ok(ObservableRuntime::Distance(*a_idx, *b_idx))
        }
    }
}

/// Convert AST loop body statement to runtime
fn convert_loop_body_stmt(
    stmt: &crate::ast::LoopBodyStmt,
    name_to_idx: &HashMap<String, usize>,
    eval_ctx: &EvalContext<'_>,
) -> Result<LoopBodyRuntime, Box<dyn std::error::Error>> {
    match stmt {
        crate::ast::LoopBodyStmt::ForcePush {
            particle,
            magnitude,
            direction,
        } => {
            let particle_idx = name_to_idx
                .get(particle)
                .ok_or_else(|| format!("Particle '{}' not found", particle))?;
            
            // Evaluate expressions
            let magnitude_value = eval_expr(magnitude, eval_ctx)
                .map_err(|e| format!("Error evaluating push magnitude: {}", e))?;
            let x_value = eval_expr(&direction.0, eval_ctx)
                .map_err(|e| format!("Error evaluating push direction x: {}", e))?;
            let y_value = eval_expr(&direction.1, eval_ctx)
                .map_err(|e| format!("Error evaluating push direction y: {}", e))?;
            
            Ok(LoopBodyRuntime::ForcePush {
                particle_index: *particle_idx,
                magnitude: magnitude_value,
                direction: Vec2::new(x_value, y_value),
            })
        }
    }
}

/// Build runtime wells from AST wells
fn build_wells(
    well_decls: &[crate::ast::WellDecl],
    name_to_idx: &HashMap<String, usize>,
    eval_ctx: &EvalContext<'_>,
) -> Result<Vec<WellInstance>, Box<dyn std::error::Error>> {
    let mut wells = Vec::new();

    for well_decl in well_decls {
        let particle_idx = name_to_idx
            .get(&well_decl.particle)
            .ok_or_else(|| format!("Particle '{}' not found for well", well_decl.particle))?;

        let observable = convert_observable(&well_decl.observable, name_to_idx)?;

        // Evaluate expressions
        let threshold_value = eval_expr(&well_decl.threshold, eval_ctx)
            .map_err(|e| format!("Error evaluating well threshold: {}", e))?;
        let depth_value = eval_expr(&well_decl.depth, eval_ctx)
            .map_err(|e| format!("Error evaluating well depth: {}", e))?;

        wells.push(WellInstance {
            particle_index: *particle_idx,
            observable,
            threshold: threshold_value,
            depth: depth_value,
        });
    }

    Ok(wells)
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

// ============================================================================
// VEL (Visual Evaluation Loop) API - v0.5+
// ============================================================================

/// Particle state for visualization
#[derive(Debug, Clone)]
pub struct ParticleState {
    pub name: String,
    pub pos: Vec2,
    pub mass: f32,
}

/// Build simulation context from source code, returning diagnostics
pub fn build_simulation_context_from_source(
    source: &str,
) -> Result<(SimulationContext, Diagnostics), Box<dyn std::error::Error>> {
    let mut program = parse_program(source)?;
    
    // Perform static analysis
    let mut diagnostics = analyze_program(&program);
    
    // If there are errors, return them
    if diagnostics.has_errors() {
        return Err(format!(
            "Static analysis errors:\n{}",
            diagnostics
                .errors()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        ).into());
    }
    
    // Evaluate let bindings (clone to avoid borrow conflict)
    let lets = program.lets.clone();
    let (eval_ctx, eval_diagnostics) = evaluate_lets(&lets);
    diagnostics.extend(eval_diagnostics.into());
    
    // If there are evaluation errors, return them
    if diagnostics.has_errors() {
        return Err(format!(
            "Expression evaluation errors:\n{}",
            diagnostics
                .errors()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        ).into());
    }
    
    // Execute functions to generate world-building statements
    let func_diagnostics = execute_functions(&mut program, &eval_ctx);
    diagnostics.extend(func_diagnostics.into());
    
    // If there are function execution errors, return them
    if diagnostics.has_errors() {
        return Err(format!(
            "Function execution errors:\n{}",
            diagnostics
                .errors()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        ).into());
    }
    
    // Re-analyze program after function execution
    let post_func_diagnostics = analyze_program(&program);
    diagnostics.extend(post_func_diagnostics.into());
    
    // If there are post-function analysis errors, return them
    if diagnostics.has_errors() {
        return Err(format!(
            "Post-function analysis errors:\n{}",
            diagnostics
                .errors()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        ).into());
    }
    
    let ctx = build_simulation_context(&program, &eval_ctx)?;
    Ok((ctx, diagnostics))
}

/// Step the simulation forward by one step
/// Returns true if the simulation is finished (current_step >= max_steps)
pub fn step_simulation(ctx: &mut SimulationContext) -> bool {
    if ctx.current_step >= ctx.max_steps {
        return true;
    }

    // 1. Update loops (advance oscillators, fire iterations)
    update_and_apply_loops(&mut ctx.loops, &mut ctx.world.particles, ctx.dt);

    // 2. Apply wells (convert wells into forces/accelerations)
    apply_wells(&ctx.wells, &mut ctx.world.particles, ctx.dt);

    // 3. Integrate physics
    step(&mut ctx.world, ctx.dt);

    // 4. Evaluate while-loop conditions to deactivate finished loops
    evaluate_loop_conditions(&mut ctx.loops, &ctx.world.particles);

    // 5. Increment step counter
    ctx.current_step += 1;

    // Return true if finished
    ctx.current_step >= ctx.max_steps
}

/// Get particle states for visualization
pub fn get_particle_states(ctx: &SimulationContext) -> Vec<ParticleState> {
    ctx.world
        .particles
        .iter()
        .map(|p| ParticleState {
            name: p.name.clone(),
            pos: p.pos,
            mass: p.mass,
        })
        .collect()
}
