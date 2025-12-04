//! Function execution for PhysLang v0.7
//!
//! This module executes user-defined functions before simulation,
//! generating world-building statements (particles, forces, etc.)

use crate::ast::{Expr, FunctionDecl, Program, Stmt};
use crate::diagnostics::Diagnostic;
use crate::eval::{eval_expr_with_function_ctx, EvalContext, FunctionEvalContext};
use std::collections::HashMap;

/// Execute all functions in the program, generating world-building statements
pub fn execute_functions(
    program: &mut Program,
    eval_ctx: &EvalContext<'_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Clone what we need before taking mutable borrows
    // Use a block to ensure the borrow of program.functions ends
    let (functions, top_level_calls) = {
        let functions = program.functions.clone();
        let calls = std::mem::take(&mut program.top_level_calls);
        (functions, calls)
    };
    
    // Build function map from cloned functions
    let function_map: HashMap<String, &FunctionDecl> = functions
        .iter()
        .map(|f| (f.name.clone(), f))
        .collect();

    // Execute top-level statements (function calls and control flow)
    for stmt in top_level_calls {
        match stmt {
            Stmt::ExprCall { name, args } => {
                match execute_function_call(&name, &args, &function_map, eval_ctx, program, None) {
                    Ok(()) => {}
                    Err(e) => {
                        diagnostics.push(Diagnostic::error(
                            format!("Error executing function '{}': {}", name, e),
                            None,
                        ));
                    }
                }
            }
            // v0.8: Execute top-level control flow statements
            _ => {
                // Create a minimal function context for top-level execution
                let mut top_ctx = FunctionEvalContext::new(eval_ctx);
                match execute_statements(&[stmt], &mut top_ctx, program, &function_map) {
                    Ok(_) => {}
                    Err(e) => {
                        diagnostics.push(Diagnostic::error(
                            format!("Error executing top-level statement: {}", e),
                            None,
                        ));
                    }
                }
            }
        }
    }

    diagnostics
}

/// Execute a single function call
fn execute_function_call(
    func_name: &str,
    args: &[Expr],
    function_map: &HashMap<String, &FunctionDecl>,
    global_ctx: &EvalContext<'_>,
    program: &mut Program,
    caller_ctx: Option<&FunctionEvalContext<'_>>,
) -> Result<(), String> {
    let func = function_map
        .get(func_name)
        .ok_or_else(|| format!("Unknown function '{}'", func_name))?;

    if args.len() != func.params.len() {
        return Err(format!(
            "Function '{}' expects {} argument(s), got {}",
            func_name,
            func.params.len(),
            args.len()
        ));
    }

    // Evaluate arguments in caller's context (if any)
    let mut arg_values = Vec::new();
    for arg in args {
        let value = eval_expr_with_function_ctx(arg, global_ctx, caller_ctx)
            .map_err(|e| format!("Error evaluating argument: {}", e))?;
        arg_values.push(value);
    }

    // Create new function execution context for the called function
    let mut func_ctx = FunctionEvalContext::new(global_ctx);
    for (param_name, arg_value) in func.params.iter().zip(arg_values.iter()) {
        func_ctx.params.insert(param_name.clone(), *arg_value);
    }

    // Execute function body
    execute_statements(&func.body, &mut func_ctx, program, function_map)?;

    Ok(())
}

/// Execute a sequence of statements
fn execute_statements(
    stmts: &[Stmt],
    func_ctx: &mut FunctionEvalContext<'_>,
    program: &mut Program,
    function_map: &HashMap<String, &FunctionDecl>,
) -> Result<Option<f32>, String> {
    for stmt in stmts {
        match stmt {
            Stmt::Let { name, expr } => {
                let value = eval_expr_with_function_ctx(expr, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating let binding '{}': {}", name, e))?;
                func_ctx.local_lets.insert(name.clone(), value);
            }
            Stmt::ExprCall { name, args } => {
                // Evaluate arguments in current context
                let mut arg_values = Vec::new();
                for arg in args {
                    let value = eval_expr_with_function_ctx(arg, func_ctx.global, Some(func_ctx))
                        .map_err(|e| format!("Error evaluating function call argument: {}", e))?;
                    arg_values.push(value);
                }
                
                // Convert to Expr::Literal for passing to function
                let arg_exprs: Vec<Expr> = arg_values
                    .iter()
                    .map(|v| Expr::Literal(*v))
                    .collect();
                
                // Execute the called function (pass current context for argument evaluation)
                execute_function_call(
                    name,
                    &arg_exprs,
                    function_map,
                    func_ctx.global,
                    program,
                    Some(func_ctx),
                )?;
            }
            Stmt::Return(expr) => {
                let value = eval_expr_with_function_ctx(expr, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating return expression: {}", e))?;
                return Ok(Some(value));
            }
            Stmt::ParticleDecl(particle) => {
                // Evaluate expressions and create particle
                let x = eval_expr_with_function_ctx(&particle.position.0, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating particle x position: {}", e))?;
                let y = eval_expr_with_function_ctx(&particle.position.1, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating particle y position: {}", e))?;
                let mass = eval_expr_with_function_ctx(&particle.mass, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating particle mass: {}", e))?;
                
                // Create new particle with evaluated expressions
                let mut new_particle = particle.clone();
                new_particle.position = (Expr::Literal(x), Expr::Literal(y));
                new_particle.mass = Expr::Literal(mass);
                
                program.particles.push(new_particle);
            }
            Stmt::ForceDecl(force) => {
                // Evaluate force expressions
                let mut new_force = force.clone();
                match &mut new_force.kind {
                    crate::ast::ForceKind::Gravity { g } => {
                        let g_val = eval_expr_with_function_ctx(g, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating gravity G: {}", e))?;
                        *g = Expr::Literal(g_val);
                    }
                    crate::ast::ForceKind::Spring { k, rest } => {
                        let k_val = eval_expr_with_function_ctx(k, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating spring k: {}", e))?;
                        let rest_val = eval_expr_with_function_ctx(rest, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating spring rest: {}", e))?;
                        *k = Expr::Literal(k_val);
                        *rest = Expr::Literal(rest_val);
                    }
                }
                program.forces.push(new_force);
            }
            Stmt::LoopDecl(loop_decl) => {
                // Evaluate loop expressions
                let mut new_loop = loop_decl.clone();
                match &mut new_loop.kind {
                    crate::ast::LoopKind::ForCycles {
                        cycles,
                        frequency,
                        damping,
                        ..
                    } => {
                        let cycles_val = eval_expr_with_function_ctx(cycles, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating cycles: {}", e))?;
                        let freq_val = eval_expr_with_function_ctx(frequency, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating frequency: {}", e))?;
                        let damp_val = eval_expr_with_function_ctx(damping, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating damping: {}", e))?;
                        *cycles = Expr::Literal(cycles_val);
                        *frequency = Expr::Literal(freq_val);
                        *damping = Expr::Literal(damp_val);
                    }
                    crate::ast::LoopKind::WhileCondition {
                        condition,
                        frequency,
                        damping,
                        ..
                    } => {
                        let freq_val = eval_expr_with_function_ctx(frequency, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating frequency: {}", e))?;
                        let damp_val = eval_expr_with_function_ctx(damping, func_ctx.global, Some(func_ctx))
                            .map_err(|e| format!("Error evaluating damping: {}", e))?;
                        *frequency = Expr::Literal(freq_val);
                        *damping = Expr::Literal(damp_val);
                        
                        // Evaluate condition threshold
                        match condition {
                            crate::ast::ConditionExpr::LessThan(_, threshold)
                            | crate::ast::ConditionExpr::GreaterThan(_, threshold) => {
                                let threshold_val = eval_expr_with_function_ctx(threshold, func_ctx.global, Some(func_ctx))
                                    .map_err(|e| format!("Error evaluating condition threshold: {}", e))?;
                                *threshold = Expr::Literal(threshold_val);
                            }
                        }
                    }
                }
                program.loops.push(new_loop);
            }
            Stmt::WellDecl(well) => {
                // Evaluate well expressions
                let mut new_well = well.clone();
                let threshold_val = eval_expr_with_function_ctx(&new_well.threshold, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating well threshold: {}", e))?;
                let depth_val = eval_expr_with_function_ctx(&new_well.depth, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating well depth: {}", e))?;
                new_well.threshold = Expr::Literal(threshold_val);
                new_well.depth = Expr::Literal(depth_val);
                program.wells.push(new_well);
            }
            Stmt::DetectorDecl(detector) => {
                program.detectors.push(detector.clone());
            }
            // v0.8: Control flow statements
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Evaluate condition
                let cond_val = eval_expr_with_function_ctx(condition, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating if condition: {}", e))?;
                
                // Interpret as boolean: true if != 0.0
                let cond_true = cond_val != 0.0;
                
                if cond_true {
                    // Execute then branch in new scope
                    let mut then_ctx = func_ctx.clone_scope();
                    execute_statements(then_branch, &mut then_ctx, program, function_map)?;
                } else if !else_branch.is_empty() {
                    // Execute else branch in new scope
                    let mut else_ctx = func_ctx.clone_scope();
                    execute_statements(else_branch, &mut else_ctx, program, function_map)?;
                }
            }
            Stmt::For {
                var_name,
                start,
                end,
                body,
            } => {
                // Evaluate start and end
                let start_val = eval_expr_with_function_ctx(start, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating for loop start: {}", e))?;
                let end_val = eval_expr_with_function_ctx(end, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating for loop end: {}", e))?;
                
                // Convert to integer bounds
                let start_i = start_val.floor() as i64;
                let end_i = end_val.floor() as i64;
                
                // Execute loop body for each iteration
                for i in start_i..end_i {
                    // Create new scope for loop body
                    let mut loop_ctx = func_ctx.clone_scope();
                    // Bind loop variable
                    loop_ctx.local_lets.insert(var_name.clone(), i as f32);
                    
                    // Execute body
                    match execute_statements(body, &mut loop_ctx, program, function_map) {
                        Ok(Some(return_val)) => {
                            // Return from function
                            return Ok(Some(return_val));
                        }
                        Ok(None) => {
                            // Continue loop
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Stmt::Match { scrutinee, arms } => {
                // Evaluate scrutinee
                let scrutinee_val = eval_expr_with_function_ctx(scrutinee, func_ctx.global, Some(func_ctx))
                    .map_err(|e| format!("Error evaluating match scrutinee: {}", e))?;
                
                // Convert to integer (round to nearest)
                let scrutinee_i = scrutinee_val.round() as i64;
                
                // Find matching arm
                for arm in arms {
                    let matches = match &arm.pattern {
                        crate::ast::MatchPattern::Literal(lit) => scrutinee_i == *lit,
                        crate::ast::MatchPattern::Wildcard => true,
                    };
                    
                    if matches {
                        // Execute arm body in new scope
                        let mut arm_ctx = func_ctx.clone_scope();
                        match execute_statements(&arm.body, &mut arm_ctx, program, function_map) {
                            Ok(Some(return_val)) => {
                                // Return from function
                                return Ok(Some(return_val));
                            }
                            Ok(None) => {
                                // Continue after match
                            }
                            Err(e) => return Err(e),
                        }
                        break; // Only execute first matching arm
                    }
                }
                
                // If no arm matched and no wildcard, do nothing (silently continue)
            }
        }
    }

    Ok(None) // No return value
}

