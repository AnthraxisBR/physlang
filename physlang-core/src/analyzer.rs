//! Static analysis and type checking for PhysLang programs
//!
//! This module performs static checks on parsed programs to catch
//! errors before execution.

use crate::ast::{
    ConditionExpr, DetectorKind, Expr, FuncName, FunctionDecl, LetDecl, LoopKind,
    ObservableExpr, Program, Stmt,
};
use crate::diagnostics::{Diagnostic, Diagnostics, Span};
use std::collections::HashMap;

/// Analyze a program and return diagnostics
pub fn analyze_program(program: &Program) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();

    // Check let bindings: duplicate names
    let mut let_names = HashMap::new();
    for (idx, let_decl) in program.lets.iter().enumerate() {
        if let_names.insert(let_decl.name.clone(), idx).is_some() {
            diagnostics.push(Diagnostic::error(
                format!("duplicate let binding '{}'", let_decl.name),
                None, // TODO: Add spans to AST
            ));
        }
    }

    // Check functions: duplicate names and name collisions
    let mut function_names = HashMap::new();
    for (idx, func_decl) in program.functions.iter().enumerate() {
        if function_names.insert(func_decl.name.clone(), idx).is_some() {
            diagnostics.push(Diagnostic::error(
                format!("duplicate function name '{}'", func_decl.name),
                None,
            ));
        }
        
        // Check for collision with let bindings
        if let_names.contains_key(&func_decl.name) {
            diagnostics.push(Diagnostic::error(
                format!("function name '{}' conflicts with let binding", func_decl.name),
                None,
            ));
        }
        
        // Check parameter names within function
        let mut param_names = HashMap::new();
        for (param_idx, param) in func_decl.params.iter().enumerate() {
            if param_names.insert(param.clone(), param_idx).is_some() {
                diagnostics.push(Diagnostic::error(
                    format!("duplicate parameter name '{}' in function '{}'", param, func_decl.name),
                    None,
                ));
            }
        }
        
        // Check function body statements
        let func_diagnostics = check_function_body(&func_decl, &program.functions);
        diagnostics.extend(func_diagnostics.into());
    }

    // Build environment for expression checking
    let env_lets: HashMap<String, &LetDecl> = program
        .lets
        .iter()
        .map(|let_decl| (let_decl.name.clone(), let_decl))
        .collect();
    
    // Build function map for call validation
    let function_map: HashMap<String, &FunctionDecl> = program
        .functions
        .iter()
        .map(|func| (func.name.clone(), func))
        .collect();
    
    // Check top-level function calls
    for call in &program.top_level_calls {
        if let Stmt::ExprCall { name, args } = call {
            if let Some(func) = function_map.get(name) {
                if args.len() != func.params.len() {
                    diagnostics.push(Diagnostic::error(
                        format!(
                            "function '{}' expects {} argument(s), got {}",
                            name,
                            func.params.len(),
                            args.len()
                        ),
                        None,
                    ));
                }
                // Check argument expressions
                for arg in args {
                    let expr_diagnostics = check_expr(arg, &env_lets);
                    diagnostics.extend(expr_diagnostics.into());
                }
            } else {
                diagnostics.push(Diagnostic::error(
                    format!("unknown function '{}'", name),
                    None,
                ));
            }
        }
    }

    // Check all let expressions
    for let_decl in &program.lets {
        let expr_diagnostics = check_expr(&let_decl.expr, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
    }

    // Build particle name map for validation
    let mut particle_names = std::collections::HashMap::new();
    let mut particle_spans = std::collections::HashMap::<String, Option<Span>>::new();

    // Check 1: Unique particle names and validate expressions
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

        // Check particle expressions
        let expr_diagnostics = check_expr(&particle.position.0, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
        let expr_diagnostics = check_expr(&particle.position.1, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
        let expr_diagnostics = check_expr(&particle.mass, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
    }

    // Check 2: Forces reference existing particles and validate expressions
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

        // Check force expressions
        match &force.kind {
            crate::ast::ForceKind::Gravity { g } => {
                let expr_diagnostics = check_expr(g, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
            }
            crate::ast::ForceKind::Spring { k, rest } => {
                let expr_diagnostics = check_expr(k, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(rest, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
            }
        }
    }

    // Check 3: Loops reference existing particles and validate expressions
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

        // Check loop expressions
        match &loop_decl.kind {
            LoopKind::ForCycles {
                cycles,
                frequency,
                damping,
                ..
            } => {
                let expr_diagnostics = check_expr(cycles, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(frequency, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(damping, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
            }
            LoopKind::WhileCondition {
                condition,
                frequency,
                damping,
                ..
            } => {
                let expr_diagnostics = check_expr(frequency, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(damping, &env_lets);
                diagnostics.extend(expr_diagnostics.into());
                check_observable_in_condition(condition, &particle_names, &mut diagnostics);
                // Check condition threshold expressions
                match condition {
                    ConditionExpr::LessThan(_, threshold) | ConditionExpr::GreaterThan(_, threshold) => {
                        let expr_diagnostics = check_expr(threshold, &env_lets);
                        diagnostics.extend(expr_diagnostics.into());
                    }
                }
            }
        }

        // Check loop body push targets and expressions
        for stmt in &loop_decl.body {
            match stmt {
                crate::ast::LoopBodyStmt::ForcePush {
                    particle,
                    magnitude,
                    direction,
                } => {
                    if !particle_names.contains_key(particle) {
                        diagnostics.push(Diagnostic::error(
                            format!("unknown particle '{}' in loop body push", particle),
                            None, // TODO: Add spans to AST
                        ));
                    }
                    let expr_diagnostics = check_expr(magnitude, &env_lets);
                    diagnostics.extend(expr_diagnostics.into());
                    let expr_diagnostics = check_expr(&direction.0, &env_lets);
                    diagnostics.extend(expr_diagnostics.into());
                    let expr_diagnostics = check_expr(&direction.1, &env_lets);
                    diagnostics.extend(expr_diagnostics.into());
                }
            }
        }
    }

    // Check 4: Wells reference existing particles and validate expressions
    for well in &program.wells {
        if !particle_names.contains_key(&well.particle) {
            diagnostics.push(Diagnostic::error(
                format!("unknown particle '{}' in well", well.particle),
                None, // TODO: Add spans to AST
            ));
        }

        // Check observable in well
        check_observable(&well.observable, &particle_names, &mut diagnostics);

        // Check well expressions
        let expr_diagnostics = check_expr(&well.threshold, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
        let expr_diagnostics = check_expr(&well.depth, &env_lets);
        diagnostics.extend(expr_diagnostics.into());
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

    // Check 6: Simulate block exists and is unique, and validate expressions
    // This is already checked in the parser, but we verify here too
    // (The parser ensures exactly one simulate block exists)
    let expr_diagnostics = check_expr(&program.simulate.dt, &env_lets);
    diagnostics.extend(expr_diagnostics.into());
    let expr_diagnostics = check_expr(&program.simulate.steps, &env_lets);
    diagnostics.extend(expr_diagnostics.into());

    diagnostics
}

/// Check an expression for validity (unknown variables, function arity, etc.)
pub fn check_expr(expr: &Expr, env_lets: &HashMap<String, &LetDecl>) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();

    match expr {
        Expr::Literal(_) => {
            // Literals are always valid
        }
        Expr::Var(name) => {
            if !env_lets.contains_key(name) {
                diagnostics.push(Diagnostic::error(
                    format!("unknown variable '{}'", name),
                    None, // TODO: Add spans to AST
                ));
            }
        }
        Expr::UnaryMinus(e) => {
            let expr_diagnostics = check_expr(e, env_lets);
            diagnostics.extend(expr_diagnostics.into());
        }
        Expr::Binary { left, right, .. } => {
            let left_diagnostics = check_expr(left, env_lets);
            diagnostics.extend(left_diagnostics.into());
            let right_diagnostics = check_expr(right, env_lets);
            diagnostics.extend(right_diagnostics.into());
        }
        Expr::Call { func, args } => {
            // Check function arity
            let expected_arity = match func {
                FuncName::Sin | FuncName::Cos | FuncName::Sqrt => 1,
                FuncName::Clamp => 3,
            };

            if args.len() != expected_arity {
                let func_name = match func {
                    FuncName::Sin => "sin",
                    FuncName::Cos => "cos",
                    FuncName::Sqrt => "sqrt",
                    FuncName::Clamp => "clamp",
                };
                diagnostics.push(Diagnostic::error(
                    format!(
                        "function '{}' expects {} argument(s), got {}",
                        func_name, expected_arity, args.len()
                    ),
                    None, // TODO: Add spans to AST
                ));
            }

            // Check all arguments
            for arg in args {
                let arg_diagnostics = check_expr(arg, env_lets);
                diagnostics.extend(arg_diagnostics.into());
            }
        }
    }

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

/// Check a function body for validity
fn check_function_body(
    func: &FunctionDecl,
    all_functions: &[FunctionDecl],
) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();
    
    // Build function map for call validation
    let function_map: HashMap<String, &FunctionDecl> = all_functions
        .iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    
    // Track local let bindings
    let mut local_lets = HashMap::new();
    
    for stmt in &func.body {
        match stmt {
            Stmt::Let { name, expr } => {
                // Check expression (can reference params and previous local lets)
                // For now, just check the expression structure
                let expr_diagnostics = check_expr(expr, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
                
                local_lets.insert(name.clone(), ());
            }
            Stmt::ExprCall { name, args } => {
                if let Some(called_func) = function_map.get(name) {
                    if args.len() != called_func.params.len() {
                        diagnostics.push(Diagnostic::error(
                            format!(
                                "function '{}' expects {} argument(s), got {}",
                                name,
                                called_func.params.len(),
                                args.len()
                            ),
                            None,
                        ));
                    }
                    // Check argument expressions
                    for arg in args {
                        let expr_diagnostics = check_expr(arg, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                    }
                } else {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown function '{}'", name),
                        None,
                    ));
                }
            }
            Stmt::Return(expr) => {
                // Check return expression
                let expr_diagnostics = check_expr(expr, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
            }
            Stmt::ParticleDecl(particle) => {
                // Check particle expressions
                let expr_diagnostics = check_expr(&particle.position.0, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(&particle.position.1, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(&particle.mass, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
            }
            Stmt::ForceDecl(force) => {
                match &force.kind {
                    crate::ast::ForceKind::Gravity { g } => {
                        let expr_diagnostics = check_expr(g, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                    }
                    crate::ast::ForceKind::Spring { k, rest } => {
                        let expr_diagnostics = check_expr(k, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                        let expr_diagnostics = check_expr(rest, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                    }
                }
            }
            Stmt::LoopDecl(loop_decl) => {
                match &loop_decl.kind {
                    crate::ast::LoopKind::ForCycles {
                        cycles,
                        frequency,
                        damping,
                        ..
                    } => {
                        let expr_diagnostics = check_expr(cycles, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                        let expr_diagnostics = check_expr(frequency, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                        let expr_diagnostics = check_expr(damping, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                    }
                    crate::ast::LoopKind::WhileCondition {
                        condition,
                        frequency,
                        damping,
                        ..
                    } => {
                        let expr_diagnostics = check_expr(frequency, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                        let expr_diagnostics = check_expr(damping, &HashMap::new());
                        diagnostics.extend(expr_diagnostics.into());
                        match condition {
                            ConditionExpr::LessThan(_, threshold)
                            | ConditionExpr::GreaterThan(_, threshold) => {
                                let expr_diagnostics = check_expr(threshold, &HashMap::new());
                                diagnostics.extend(expr_diagnostics.into());
                            }
                        }
                    }
                }
            }
            Stmt::WellDecl(well) => {
                let expr_diagnostics = check_expr(&well.threshold, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
                let expr_diagnostics = check_expr(&well.depth, &HashMap::new());
                diagnostics.extend(expr_diagnostics.into());
            }
            Stmt::DetectorDecl(_) => {
                // Detectors don't have expressions to check
            }
        }
    }
    
    diagnostics
}

