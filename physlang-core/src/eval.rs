//! Expression evaluation for PhysLang
//!
//! This module evaluates expressions to concrete f32 values before simulation.

use crate::ast::{BinaryOp, Expr, FuncName, LetDecl};
use crate::diagnostics::Diagnostic;
use std::collections::HashMap;

/// Evaluation context storing variable values
pub struct EvalContext<'a> {
    /// Values of let-bindings after evaluation
    pub values: HashMap<&'a str, f32>,
}

impl<'a> EvalContext<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl<'a> Default for EvalContext<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Function execution context with local variables and parameters
pub struct FunctionEvalContext<'a> {
    /// Global let bindings
    pub global: &'a EvalContext<'a>,
    /// Function parameters
    pub params: HashMap<String, f32>,
    /// Local let bindings within the function
    pub local_lets: HashMap<String, f32>,
}

impl<'a> FunctionEvalContext<'a> {
    pub fn new(global: &'a EvalContext<'a>) -> Self {
        Self {
            global,
            params: HashMap::new(),
            local_lets: HashMap::new(),
        }
    }
    
    /// Look up a variable: local lets -> params -> global lets
    pub fn lookup(&self, name: &str) -> Option<f32> {
        self.local_lets
            .get(name)
            .or_else(|| self.params.get(name))
            .or_else(|| self.global.values.get(name))
            .copied()
    }
    
    /// Clone this context to create a new scope (v0.8: for control flow)
    /// The new scope shares the same global context, params, and inherits local_lets
    pub fn clone_scope(&self) -> Self {
        Self {
            global: self.global,
            params: self.params.clone(),
            local_lets: self.local_lets.clone(),
        }
    }
}

/// Evaluation error
#[derive(Debug, Clone)]
pub enum EvalError {
    UnknownVar(String),
    DivByZero,
    InvalidArgs(String),
    FuncError(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnknownVar(name) => write!(f, "unknown variable '{}'", name),
            EvalError::DivByZero => write!(f, "division by zero"),
            EvalError::InvalidArgs(msg) => write!(f, "invalid arguments: {}", msg),
            EvalError::FuncError(msg) => write!(f, "function error: {}", msg),
        }
    }
}

impl std::error::Error for EvalError {}

/// Evaluate all let-bindings in order
/// Returns the evaluation context and any diagnostics
pub fn evaluate_lets<'a>(
    lets: &'a [LetDecl],
) -> (EvalContext<'a>, Vec<Diagnostic>) {
    let mut ctx = EvalContext::new();
    let mut diagnostics = Vec::new();

    for let_decl in lets {
        match eval_expr(&let_decl.expr, &ctx) {
            Ok(value) => {
                // Check for NaN or infinity
                if value.is_nan() {
                    diagnostics.push(Diagnostic::error(
                        format!("let binding '{}' evaluates to NaN", let_decl.name),
                        None,
                    ));
                    continue;
                }
                if value.is_infinite() {
                    diagnostics.push(Diagnostic::error(
                        format!("let binding '{}' evaluates to infinity", let_decl.name),
                        None,
                    ));
                    continue;
                }
                ctx.values.insert(&let_decl.name, value);
            }
            Err(e) => {
                diagnostics.push(Diagnostic::error(
                    format!("error evaluating let binding '{}': {}", let_decl.name, e),
                    None,
                ));
            }
        }
    }

    (ctx, diagnostics)
}

/// Evaluate an expression to a f32 value (global context)
pub fn eval_expr(expr: &Expr, ctx: &EvalContext<'_>) -> Result<f32, EvalError> {
    eval_expr_with_function_ctx(expr, ctx, None)
}

/// Evaluate an expression with optional function context
pub fn eval_expr_with_function_ctx(
    expr: &Expr,
    global_ctx: &EvalContext<'_>,
    func_ctx: Option<&FunctionEvalContext<'_>>,
) -> Result<f32, EvalError> {
    match expr {
        Expr::Literal(v) => Ok(*v),
        
        Expr::Var(name) => {
            // Look up in function context first, then global
            if let Some(func_ctx) = func_ctx {
                func_ctx
                    .lookup(name)
                    .ok_or_else(|| EvalError::UnknownVar(name.clone()))
            } else {
                global_ctx
                    .values
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| EvalError::UnknownVar(name.clone()))
            }
        }
        
        Expr::UnaryMinus(e) => {
            let v = eval_expr_with_function_ctx(e, global_ctx, func_ctx)?;
            Ok(-v)
        }
        
        Expr::Binary { op, left, right } => {
            let left_val = eval_expr_with_function_ctx(left, global_ctx, func_ctx)?;
            let right_val = eval_expr_with_function_ctx(right, global_ctx, func_ctx)?;
            
            match op {
                BinaryOp::Add => Ok(left_val + right_val),
                BinaryOp::Sub => Ok(left_val - right_val),
                BinaryOp::Mul => Ok(left_val * right_val),
                BinaryOp::Div => {
                    if right_val == 0.0 {
                        return Err(EvalError::DivByZero);
                    }
                    Ok(left_val / right_val)
                }
                // v0.8: Comparison operators (return 1.0 for true, 0.0 for false)
                BinaryOp::GreaterThan => Ok(if left_val > right_val { 1.0 } else { 0.0 }),
                BinaryOp::LessThan => Ok(if left_val < right_val { 1.0 } else { 0.0 }),
                BinaryOp::GreaterEqual => Ok(if left_val >= right_val { 1.0 } else { 0.0 }),
                BinaryOp::LessEqual => Ok(if left_val <= right_val { 1.0 } else { 0.0 }),
                BinaryOp::Equal => Ok(if left_val == right_val { 1.0 } else { 0.0 }),
                BinaryOp::NotEqual => Ok(if left_val != right_val { 1.0 } else { 0.0 }),
            }
        }
        
        Expr::Call { func, args } => {
            let arg_values: Result<Vec<f32>, EvalError> = args
                .iter()
                .map(|arg| eval_expr_with_function_ctx(arg, global_ctx, func_ctx))
                .collect();
            let arg_values = arg_values?;
            
            match func {
                FuncName::Sin => {
                    if arg_values.len() != 1 {
                        return Err(EvalError::InvalidArgs(
                            format!("sin expects 1 argument, got {}", arg_values.len())
                        ));
                    }
                    Ok(arg_values[0].sin())
                }
                FuncName::Cos => {
                    if arg_values.len() != 1 {
                        return Err(EvalError::InvalidArgs(
                            format!("cos expects 1 argument, got {}", arg_values.len())
                        ));
                    }
                    Ok(arg_values[0].cos())
                }
                FuncName::Sqrt => {
                    if arg_values.len() != 1 {
                        return Err(EvalError::InvalidArgs(
                            format!("sqrt expects 1 argument, got {}", arg_values.len())
                        ));
                    }
                    let x = arg_values[0];
                    if x < 0.0 {
                        return Err(EvalError::InvalidArgs(
                            format!("sqrt of negative number: {}", x)
                        ));
                    }
                    Ok(x.sqrt())
                }
                FuncName::Clamp => {
                    if arg_values.len() != 3 {
                        return Err(EvalError::InvalidArgs(
                            format!("clamp expects 3 arguments, got {}", arg_values.len())
                        ));
                    }
                    let x = arg_values[0];
                    let min = arg_values[1];
                    let max = arg_values[2];
                    // Allow min > max, just clamp in given order
                    Ok(x.max(min).min(max))
                }
            }
        }
    }
}

