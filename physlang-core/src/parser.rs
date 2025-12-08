use crate::ast::{
    BinaryOp, ConditionExpr, DetectorDecl, DetectorKind, Expr, ForceDecl, ForceKind, FuncName,
    FunctionDecl, LetDecl, LoopBodyStmt, LoopDecl, LoopKind, MatchArm, MatchPattern,
    ObservableExpr, ParticleDecl, Program, SimulateDecl, Stmt, WellDecl,
};
use crate::diagnostics::Span;
use thiserror::Error;

/// Parse error with detailed location information
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{message}")]
    SyntaxError {
        message: String,
        span: Option<Span>,
        line_number: Option<usize>,
        line_content: Option<String>,
        context: Option<String>,
    },
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span,
            line_number: None,
            line_content: None,
            context: None,
        }
    }
    
    pub fn with_line_info(
        message: impl Into<String>,
        span: Option<Span>,
        line_number: usize,
        line_content: impl Into<String>,
    ) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span,
            line_number: Some(line_number + 1), // Convert 0-indexed to 1-indexed
            line_content: Some(line_content.into()),
            context: None,
        }
    }
    
    pub fn with_context(
        message: impl Into<String>,
        span: Option<Span>,
        line_number: usize,
        line_content: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span,
            line_number: Some(line_number + 1), // Convert 0-indexed to 1-indexed
            line_content: Some(line_content.into()),
            context: Some(context.into()),
        }
    }

    pub fn message(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span: None,
            line_number: None,
            line_content: None,
            context: None,
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::SyntaxError { span, .. } => *span,
        }
    }
    
    pub fn line_number(&self) -> Option<usize> {
        match self {
            Self::SyntaxError { line_number, .. } => *line_number,
        }
    }
    
    pub fn line_content(&self) -> Option<&str> {
        match self {
            Self::SyntaxError { line_content, .. } => line_content.as_deref(),
        }
    }
    
    pub fn context(&self) -> Option<&str> {
        match self {
            Self::SyntaxError { context, .. } => context.as_deref(),
        }
    }
    
    /// Format a detailed error message with source location
    pub fn format_detailed(&self) -> String {
        let mut result = String::new();
        
        // Add context if available
        if let Some(ctx) = self.context() {
            result.push_str(&format!("[{}] ", ctx));
        }
        
        // Add main message
        match self {
            Self::SyntaxError { message, .. } => {
                result.push_str(message);
            }
        }
        
        // Add line info
        if let Some(line_num) = self.line_number() {
            result.push_str(&format!("\n  --> line {}", line_num));
        }
        
        // Add line content
        if let Some(line_content) = self.line_content() {
            result.push_str(&format!("\n  | {}", line_content));
        }
        
        result
    }
}

/// Check if verbose parsing is enabled via PHYSLANG_PARSE_TRACE env var
fn is_trace_enabled() -> bool {
    std::env::var("PHYSLANG_PARSE_TRACE").is_ok()
}

/// Log a trace message if tracing is enabled
macro_rules! trace_parse {
    ($($arg:tt)*) => {
        if is_trace_enabled() {
            eprintln!("[PARSE] {}", format!($($arg)*));
        }
    };
}

/// Helper to track byte offsets while parsing
struct ParseContext {
    #[allow(dead_code)]
    source: String,
    lines: Vec<String>,
    line_offsets: Vec<usize>, // Byte offset of start of each line
}

impl ParseContext {
    fn new(source: &str) -> Self {
        let mut line_offsets = vec![0];
        let mut offset = 0;
        for ch in source.chars() {
            offset += ch.len_utf8();
            if ch == '\n' {
                line_offsets.push(offset);
            }
        }
        Self {
            source: source.to_string(),
            lines: source.lines().map(|s| s.to_string()).collect(),
            line_offsets,
        }
    }

    /// Get byte offset for start of line (0-indexed)
    fn line_start(&self, line: usize) -> usize {
        self.line_offsets.get(line).copied().unwrap_or(0)
    }

    /// Create a span for a line
    #[allow(dead_code)]
    fn line_span(&self, line: usize, start_col: usize, end_col: usize) -> Span {
        let line_start = self.line_start(line);
        Span::new(
            line_start + start_col,
            line_start + end_col,
        )
    }

    /// Create a span for the entire line
    fn full_line_span(&self, line: usize) -> Span {
        let start = self.line_start(line);
        let end = self.line_start(line + 1);
        Span::new(start, end)
    }
    
    /// Get line content by index (0-indexed)
    fn get_line(&self, line: usize) -> &str {
        self.lines.get(line).map(|s| s.as_str()).unwrap_or("")
    }
    
    /// Create a detailed parse error with line information
    fn error(&self, message: impl Into<String>, line: usize, context: impl Into<String>) -> ParseError {
        ParseError::with_context(
            message,
            Some(self.full_line_span(line)),
            line,
            self.get_line(line),
            context,
        )
    }
    
    /// Create a simple parse error with line information
    fn error_simple(&self, message: impl Into<String>, line: usize) -> ParseError {
        ParseError::with_line_info(
            message,
            Some(self.full_line_span(line)),
            line,
            self.get_line(line),
        )
    }
}

/// Parse a PhysLang program from source code
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    trace_parse!("Starting parse_program");
    let ctx = ParseContext::new(source);
    let mut lets = Vec::new();
    let mut functions = Vec::new();
    let mut top_level_calls = Vec::new();
    let mut particles = Vec::new();
    let mut forces = Vec::new();
    let mut simulate = None;
    let mut detectors = Vec::new();
    let mut loops = Vec::new();
    let mut wells = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        let line_span = ctx.full_line_span(i);
        
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        trace_parse!("Line {}: parsing '{}'", i + 1, line);

        if line.starts_with("let ") {
            trace_parse!("  -> let declaration");
            lets.push(parse_let(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("fn ") {
            trace_parse!("  -> function declaration");
            let (func_decl, next_line) = parse_function(&lines, i, &ctx)?;
            trace_parse!("  -> function '{}' parsed, next line: {}", func_decl.name, next_line + 1);
            functions.push(func_decl);
            i = next_line;
        } else if line.starts_with("particle ") {
            trace_parse!("  -> particle declaration");
            particles.push(parse_particle(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("force ") && !line.contains("push") {
            trace_parse!("  -> force declaration");
            forces.push(parse_force(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("simulate ") {
            trace_parse!("  -> simulate declaration");
            simulate = Some(parse_simulate(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("detect ") {
            trace_parse!("  -> detect declaration");
            detectors.push(parse_detector(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("loop ") {
            trace_parse!("  -> loop declaration");
            let (loop_decl, next_line) = parse_loop(&lines, i, &ctx)?;
            trace_parse!("  -> loop parsed, next line: {}", next_line + 1);
            loops.push(loop_decl);
            i = next_line;
        } else if line.starts_with("well ") {
            trace_parse!("  -> well declaration");
            wells.push(parse_well(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("if ") {
            // v0.8: Top-level if statement
            trace_parse!("  -> if statement");
            let (stmt, next_line) = parse_if_stmt(&lines, i, &ctx)?;
            trace_parse!("  -> if parsed, next line: {}", next_line + 1);
            top_level_calls.push(stmt);
            i = next_line;
        } else if line.starts_with("for ") {
            // v0.8: Top-level for loop
            trace_parse!("  -> for loop");
            let (stmt, next_line) = parse_for_stmt(&lines, i, &ctx)?;
            trace_parse!("  -> for parsed, next line: {}", next_line + 1);
            top_level_calls.push(stmt);
            i = next_line;
        } else if line.starts_with("match ") {
            // v0.8: Top-level match statement
            trace_parse!("  -> match statement");
            let (stmt, next_line) = parse_match_stmt(&lines, i, &ctx)?;
            trace_parse!("  -> match parsed, next line: {}", next_line + 1);
            top_level_calls.push(stmt);
            i = next_line;
        } else {
            // Try parsing as function call statement
            if let Some(paren_pos) = line.find('(') {
                let func_name = line[..paren_pos].trim();
                if is_valid_identifier(func_name) {
                    trace_parse!("  -> function call: {}", func_name);
                    // This looks like a function call
                    let rest = &line[paren_pos..];
                    if let Some(paren_end) = rest.find(')') {
                        let args_str = &rest[1..paren_end];
                        let args = if args_str.trim().is_empty() {
                            Vec::new()
                        } else {
                            args_str
                                .split(',')
                                .map(|arg| parse_expr(arg.trim(), Some(line_span)))
                                .collect::<Result<Vec<_>, _>>()?
                        };
                        // Store top-level function calls for execution
                        top_level_calls.push(Stmt::ExprCall {
                            name: func_name.to_string(),
                            args,
                        });
                        i += 1;
                        continue;
                    }
                }
            }
            
            return Err(ctx.error(
                format!("Unexpected token: '{}'", line.split_whitespace().next().unwrap_or("")),
                i,
                "top-level parsing",
            ));
        }
    }

    let simulate = simulate.ok_or_else(|| {
        ParseError::message("Missing 'simulate' declaration".to_string())
    })?;

    Ok(Program {
        lets,
        functions,
        top_level_calls,
        particles,
        forces,
        simulate,
        detectors,
        loops,
        wells,
    })
}

/// Parse a particle declaration: `particle name at (x, y) mass m`
fn parse_particle(line: &str, span: Option<Span>) -> Result<ParticleDecl, ParseError> {
    // Remove "particle " prefix
    let rest = line.strip_prefix("particle ").ok_or_else(|| {
        ParseError::new("Expected 'particle' keyword", span)
    })?;
    
    // Find " at "
    let at_pos = rest.find(" at ").ok_or_else(|| {
        ParseError::new(format!("Expected 'at' in particle declaration: {}", line), span)
    })?;
    
    let name = rest[..at_pos].trim().to_string();
    let rest = &rest[at_pos + 4..];
    
    // Parse position: (x, y)
    let pos_start = rest.find('(').ok_or_else(|| {
        ParseError::new(format!("Expected '(' in position: {}", line), span)
    })?;
    let pos_end = rest.find(')').ok_or_else(|| {
        ParseError::new(format!("Expected ')' in position: {}", line), span)
    })?;
    
    let pos_str = &rest[pos_start + 1..pos_end];
    let coords: Vec<&str> = pos_str.split(',').map(|s| s.trim()).collect();
    if coords.len() != 2 {
        return Err(ParseError::new(
            format!("Expected two coordinates in position: {}", line),
            span,
        ));
    }
    
    let x_expr = parse_expr(coords[0], span)?;
    let y_expr = parse_expr(coords[1], span)?;
    
    let rest = &rest[pos_end + 1..];
    
    // Parse mass
    let mass_start = rest.find("mass ").ok_or_else(|| {
        ParseError::new(format!("Expected 'mass' in particle declaration: {}", line), span)
    })?;
    
    let mass_str = &rest[mass_start + 5..].trim();
    let mass_expr = parse_expr(mass_str, span)?;
    
    Ok(ParticleDecl {
        name,
        position: (x_expr, y_expr),
        mass: mass_expr,
    })
}

/// Parse a force declaration: `force gravity(a, b) G = x` or `force spring(a, b) k = x rest = y`
fn parse_force(line: &str, span: Option<Span>) -> Result<ForceDecl, ParseError> {
    // Remove "force " prefix
    let rest = line.strip_prefix("force ").ok_or_else(|| {
        ParseError::new("Expected 'force' keyword", span)
    })?;
    
    // Find the opening parenthesis
    let paren_start = rest.find('(').ok_or_else(|| {
        ParseError::new(format!("Expected '(' in force declaration: {}", line), span)
    })?;
    
    let force_type = rest[..paren_start].trim();
    let rest = &rest[paren_start + 1..];
    
    // Find closing parenthesis
    let paren_end = rest.find(')').ok_or_else(|| {
        ParseError::new(format!("Expected ')' in force declaration: {}", line), span)
    })?;
    
    let args_str = &rest[..paren_end];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    if args.len() != 2 {
        return Err(ParseError::new(
            format!("Expected two particle names in force: {}", line),
            span,
        ));
    }
    
    // Strip quotes from particle names if they're string literals
    let a = strip_quotes(args[0]);
    let b = strip_quotes(args[1]);
    
    let rest = &rest[paren_end + 1..].trim();
    
    let kind = match force_type {
        "gravity" => {
            // Parse: G = value
            let g_str = rest.strip_prefix("G = ").ok_or_else(|| {
                ParseError::new(format!("Expected 'G =' in gravity force: {}", line), span)
            })?;
            let g_expr = parse_expr(g_str.trim(), span)?;
            ForceKind::Gravity { g: g_expr }
        }
        "spring" => {
            // Parse: k = value rest = value
            let k_start = rest.find("k = ").ok_or_else(|| {
                ParseError::new(format!("Expected 'k =' in spring force: {}", line), span)
            })?;
            let after_k = &rest[k_start + 4..];
            let k_end = after_k.find(" rest = ").ok_or_else(|| {
                ParseError::new(format!("Expected 'rest =' in spring force: {}", line), span)
            })?;
            
            let k_str = &after_k[..k_end].trim();
            let k_expr = parse_expr(k_str, span)?;
            
            let rest_str = &after_k[k_end + 8..].trim();
            let rest_expr = parse_expr(rest_str, span)?;
            
            ForceKind::Spring { k: k_expr, rest: rest_expr }
        }
        _ => {
            return Err(ParseError::new(
                format!("Unknown force type: {}", force_type),
                span,
            ));
        }
    };
    
    Ok(ForceDecl { a, b, kind })
}

/// Parse a simulate declaration: `simulate dt = x steps = n`
fn parse_simulate(line: &str, span: Option<Span>) -> Result<SimulateDecl, ParseError> {
    // Remove "simulate " prefix
    let rest = line.strip_prefix("simulate ").ok_or_else(|| {
        ParseError::new("Expected 'simulate' keyword", span)
    })?;
    
    // Parse dt = value
    let dt_start = rest.find("dt = ").ok_or_else(|| {
        ParseError::new(format!("Expected 'dt =' in simulate: {}", line), span)
    })?;
    let after_dt = &rest[dt_start + 5..];
    let dt_end = after_dt.find(" steps = ").ok_or_else(|| {
        ParseError::new(format!("Expected 'steps =' in simulate: {}", line), span)
    })?;
    
    let dt_str = &after_dt[..dt_end].trim();
    let dt_expr = parse_expr(dt_str, span)?;
    
    let steps_str = &after_dt[dt_end + 9..].trim();
    let steps_expr = parse_expr(steps_str, span)?;
    
    Ok(SimulateDecl { dt: dt_expr, steps: steps_expr })
}

/// Parse a detector declaration: `detect name = position(a)` or `detect name = distance(a, b)`
fn parse_detector(line: &str, span: Option<Span>) -> Result<DetectorDecl, ParseError> {
    // Remove "detect " prefix
    let rest = line.strip_prefix("detect ").ok_or_else(|| {
        ParseError::new("Expected 'detect' keyword", span)
    })?;
    
    // Find " = "
    let eq_pos = rest.find(" = ").ok_or_else(|| {
        ParseError::new(format!("Expected '=' in detector: {}", line), span)
    })?;
    
    let name = rest[..eq_pos].trim().to_string();
    let rest = &rest[eq_pos + 3..].trim();
    
    let kind = if rest.starts_with("position(") {
        // Parse: position(name)
        let start = rest.find('(').unwrap();
        let end = rest.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in position detector: {}", line), span)
        })?;
        let particle_name = rest[start + 1..end].trim().to_string();
        DetectorKind::Position(particle_name)
    } else if rest.starts_with("distance(") {
        // Parse: distance(a, b)
        let start = rest.find('(').unwrap();
        let end = rest.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in distance detector: {}", line), span)
        })?;
        let args_str = &rest[start + 1..end];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        if args.len() != 2 {
            return Err(ParseError::new(
                format!("Expected two particle names in distance detector: {}", line),
                span,
            ));
        }
        DetectorKind::Distance {
            a: args[0].to_string(),
            b: args[1].to_string(),
        }
    } else {
        return Err(ParseError::new(
            format!("Unknown detector type: {}", rest),
            span,
        ));
    };
    
    Ok(DetectorDecl { name, kind })
}

// ============================================================================
// v0.2: Loop and Well Parsing
// ============================================================================

/// Parse a loop declaration (handles multi-line bodies)
fn parse_loop(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(LoopDecl, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    let rest = line.strip_prefix("loop ").ok_or_else(|| {
        ParseError::new("Expected 'loop' keyword", Some(line_span))
    })?;

    let (kind, body_start) = if rest.starts_with("for ") {
        // `loop for <integer> cycles with frequency <float> damping <float> on <ident> {`
        let after_for = rest.strip_prefix("for ").unwrap();
        
        // Find " cycles"
        let cycles_end = after_for.find(" cycles").ok_or_else(|| {
            ParseError::new(format!("Expected 'cycles' in for loop: {}", line), Some(line_span))
        })?;
        let cycles_str = after_for[..cycles_end].trim();
        let cycles_expr = parse_expr(cycles_str, Some(line_span))?;
        
        let after_cycles = &after_for[cycles_end + 7..];
        
        // Find "with frequency"
        let freq_start = after_cycles.find("with frequency ").ok_or_else(|| {
            ParseError::new(format!("Expected 'with frequency' in for loop: {}", line), Some(line_span))
        })?;
        let after_freq = &after_cycles[freq_start + 15..];
        
        // Find frequency value
        let freq_end = after_freq.find(" damping ").ok_or_else(|| {
            ParseError::new(format!("Expected 'damping' after frequency: {}", line), Some(line_span))
        })?;
        let frequency_expr = parse_expr(after_freq[..freq_end].trim(), Some(line_span))?;
        
        let after_damp = &after_freq[freq_end + 9..];
        
        // Find damping value
        let damp_end = after_damp.find(" on ").ok_or_else(|| {
            ParseError::new(format!("Expected 'on' after damping: {}", line), Some(line_span))
        })?;
        let damping_expr = parse_expr(after_damp[..damp_end].trim(), Some(line_span))?;
        
        // Find particle name
        let after_on = &after_damp[damp_end + 4..];
        let target = if after_on.ends_with(" {") {
            after_on[..after_on.len() - 2].trim().to_string()
        } else if after_on.ends_with('{') {
            after_on[..after_on.len() - 1].trim().to_string()
        } else {
            return Err(ParseError::new(
                format!("Expected '{{' after particle name: {}", line),
                Some(line_span),
            ));
        };
        
        (LoopKind::ForCycles {
            cycles: cycles_expr,
            frequency: frequency_expr,
            damping: damping_expr,
            target,
        }, start_idx + 1)
    } else if rest.starts_with("while ") {
        // `loop while <condition> with frequency <float> damping <float> on <ident> {`
        let after_while = rest.strip_prefix("while ").unwrap();
        
        // Find " with frequency"
        let with_pos = after_while.find(" with frequency ").ok_or_else(|| {
            ParseError::new(format!("Expected 'with frequency' in while loop: {}", line), Some(line_span))
        })?;
        
        let condition_str = &after_while[..with_pos];
        let condition = parse_condition(condition_str.trim(), Some(line_span))?;
        
        let after_with = &after_while[with_pos + 16..];
        
        // Parse frequency
        let freq_end = after_with.find(" damping ").ok_or_else(|| {
            ParseError::new(format!("Expected 'damping' after frequency: {}", line), Some(line_span))
        })?;
        let frequency_expr = parse_expr(after_with[..freq_end].trim(), Some(line_span))?;
        
        let after_damp = &after_with[freq_end + 9..];
        
        // Parse damping
        let damp_end = after_damp.find(" on ").ok_or_else(|| {
            ParseError::new(format!("Expected 'on' after damping: {}", line), Some(line_span))
        })?;
        let damping_expr = parse_expr(after_damp[..damp_end].trim(), Some(line_span))?;
        
        // Parse target particle
        let after_on = &after_damp[damp_end + 4..];
        let target = if after_on.ends_with(" {") {
            after_on[..after_on.len() - 2].trim().to_string()
        } else if after_on.ends_with('{') {
            after_on[..after_on.len() - 1].trim().to_string()
        } else {
            return Err(ParseError::new(
                format!("Expected '{{' after particle name: {}", line),
                Some(line_span),
            ));
        };
        
        (LoopKind::WhileCondition {
            condition,
            frequency: frequency_expr,
            damping: damping_expr,
            target,
        }, start_idx + 1)
    } else {
        return Err(ParseError::new(format!("Unknown loop type: {}", line), Some(line_span)));
    };

    // Parse loop body (lines until closing brace)
    let mut body = Vec::new();
    let mut i = body_start;
    let mut brace_count = 1; // We've seen the opening brace
    
    while i < lines.len() && brace_count > 0 {
        let body_line = lines[i].trim();
        let body_span = ctx.full_line_span(i);
        
        if body_line == "}" {
            brace_count -= 1;
            if brace_count == 0 {
                i += 1;
                break;
            }
        } else if body_line.ends_with('{') {
            brace_count += 1;
        }
        
        if brace_count > 0 && !body_line.is_empty() && !body_line.starts_with('#') {
            if body_line.starts_with("force push(") {
                body.push(parse_loop_body_stmt(body_line, Some(body_span))?);
            }
        }
        
        i += 1;
    }
    
    if brace_count > 0 {
        return Err(ParseError::new("Unclosed loop body".to_string(), Some(line_span)));
    }
    
    Ok((LoopDecl {
        name: None, // v0.2 ignores loop labels
        kind,
        body,
    }, i))
}

/// Parse a condition expression
fn parse_condition(cond_str: &str, span: Option<Span>) -> Result<ConditionExpr, ParseError> {
    // Try position(<ident>).x < float or position(<ident>).x > float
    if cond_str.contains("position(") {
        let pos_start = cond_str.find("position(").unwrap();
        let pos_end = cond_str.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in position condition: {}", cond_str), span)
        })?;
        let particle_name = cond_str[pos_start + 9..pos_end].trim().to_string();
        
        let after_paren = &cond_str[pos_end + 1..];
        if after_paren.starts_with(".x < ") {
            let threshold_expr = parse_expr(&after_paren[5..].trim(), span)?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::PositionX(particle_name),
                threshold_expr,
            ));
        } else if after_paren.starts_with(".x > ") {
            let threshold_expr = parse_expr(&after_paren[5..].trim(), span)?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::PositionX(particle_name),
                threshold_expr,
            ));
        } else if after_paren.starts_with(".y < ") {
            let threshold_expr = parse_expr(&after_paren[5..].trim(), span)?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::PositionY(particle_name),
                threshold_expr,
            ));
        } else if after_paren.starts_with(".y > ") {
            let threshold_expr = parse_expr(&after_paren[5..].trim(), span)?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::PositionY(particle_name),
                threshold_expr,
            ));
        }
    }
    
    // Try distance(a, b) < float or distance(a, b) > float
    if cond_str.starts_with("distance(") {
        let after_dist = cond_str.strip_prefix("distance(").unwrap();
        let paren_end = after_dist.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in distance condition: {}", cond_str), span)
        })?;
        let args_str = &after_dist[..paren_end];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        if args.len() != 2 {
            return Err(ParseError::new(
                format!("Expected two particle names in distance condition: {}", cond_str),
                span,
            ));
        }
        
        let rest = &after_dist[paren_end + 1..].trim();
        if rest.starts_with("< ") {
            let threshold_expr = parse_expr(&rest[2..].trim(), span)?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::Distance(args[0].to_string(), args[1].to_string()),
                threshold_expr,
            ));
        } else if rest.starts_with("> ") {
            let threshold_expr = parse_expr(&rest[2..].trim(), span)?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::Distance(args[0].to_string(), args[1].to_string()),
                threshold_expr,
            ));
        }
    }
    
    Err(ParseError::new(format!("Unknown condition format: {}", cond_str), span))
}

/// Parse a loop body statement
fn parse_loop_body_stmt(line: &str, span: Option<Span>) -> Result<LoopBodyStmt, ParseError> {
    // `force push(<ident>) magnitude <float> direction (<float>, <float>)`
    let rest = line.strip_prefix("force push(").ok_or_else(|| {
        ParseError::new("Expected 'force push('", span)
    })?;
    
    let paren_end = rest.find(')').ok_or_else(|| {
        ParseError::new(format!("Expected ')' in push force: {}", line), span)
    })?;
    let particle = rest[..paren_end].trim().to_string();
    
    let rest = &rest[paren_end + 1..].trim();
    
    // Parse magnitude
    let mag_start = rest.find("magnitude ").ok_or_else(|| {
        ParseError::new(format!("Expected 'magnitude' in push force: {}", line), span)
    })?;
    let after_mag = &rest[mag_start + 10..];
    let mag_end = after_mag.find(" direction ").ok_or_else(|| {
        ParseError::new(format!("Expected 'direction' in push force: {}", line), span)
    })?;
    let magnitude_expr = parse_expr(after_mag[..mag_end].trim(), span)?;
    
    // Parse direction
    let after_dir = &after_mag[mag_end + 11..];
    let dir_start = after_dir.find('(').ok_or_else(|| {
        ParseError::new(format!("Expected '(' in direction: {}", line), span)
    })?;
    let dir_end = after_dir.find(')').ok_or_else(|| {
        ParseError::new(format!("Expected ')' in direction: {}", line), span)
    })?;
    let dir_str = &after_dir[dir_start + 1..dir_end];
    let coords: Vec<&str> = dir_str.split(',').map(|s| s.trim()).collect();
    if coords.len() != 2 {
        return Err(ParseError::new(
            format!("Expected two coordinates in direction: {}", line),
            span,
        ));
    }
    let x_expr = parse_expr(coords[0], span)?;
    let y_expr = parse_expr(coords[1], span)?;
    
    Ok(LoopBodyStmt::ForcePush {
        particle,
        magnitude: magnitude_expr,
        direction: (x_expr, y_expr),
    })
}

/// Parse a well declaration: `well <name> on <ident> if position(<ident>).x >= <float> depth <float>`
fn parse_well(line: &str, span: Option<Span>) -> Result<WellDecl, ParseError> {
    // Remove "well " prefix
    let rest = line.strip_prefix("well ").ok_or_else(|| {
        ParseError::new("Expected 'well' keyword", span)
    })?;
    
    // Find " on "
    let on_pos = rest.find(" on ").ok_or_else(|| {
        ParseError::new(format!("Expected 'on' in well declaration: {}", line), span)
    })?;
    let name = rest[..on_pos].trim().to_string();
    
    let after_on = &rest[on_pos + 4..];
    
    // Find " if " (can be followed by position( or distance()
    let if_pos = after_on.find(" if ").ok_or_else(|| {
        ParseError::new(format!("Expected 'if' in well declaration: {}", line), span)
    })?;
    let particle = after_on[..if_pos].trim().to_string();
    
    let after_if = &after_on[if_pos + 4..];
    
    // Try to parse position(...) or distance(...)
    if after_if.starts_with("position(") {
        // Parse position(particle).x or position(particle).y
        let pos_start = after_if.find('(').unwrap();
        let pos_end = after_if.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in position: {}", line), span)
        })?;
        let pos_particle = after_if[pos_start + 1..pos_end].trim().to_string();
        
        let after_paren = &after_if[pos_end + 1..];
        if after_paren.starts_with(".x >= ") {
            let after_x = &after_paren[6..];
            let depth_pos = after_x.find(" depth ").ok_or_else(|| {
                ParseError::new(format!("Expected 'depth' in well: {}", line), span)
            })?;
            let threshold_expr = parse_expr(after_x[..depth_pos].trim(), span)?;
            let depth_expr = parse_expr(&after_x[depth_pos + 7..].trim(), span)?;
            
            return Ok(WellDecl {
                name,
                particle,
                observable: ObservableExpr::PositionX(pos_particle),
                threshold: threshold_expr,
                depth: depth_expr,
            });
        } else if after_paren.starts_with(".y >= ") {
            let after_y = &after_paren[6..];
            let depth_pos = after_y.find(" depth ").ok_or_else(|| {
                ParseError::new(format!("Expected 'depth' in well: {}", line), span)
            })?;
            let threshold_expr = parse_expr(after_y[..depth_pos].trim(), span)?;
            let depth_expr = parse_expr(&after_y[depth_pos + 7..].trim(), span)?;
            
            return Ok(WellDecl {
                name,
                particle,
                observable: ObservableExpr::PositionY(pos_particle),
                threshold: threshold_expr,
                depth: depth_expr,
            });
        } else {
            return Err(ParseError::new(
                format!("Expected '.x >= ' or '.y >= ' after position: {}", line),
                span,
            ));
        }
    } else if after_if.starts_with("distance(") {
        // Parse distance(a, b) >= threshold
        let dist_start = after_if.find('(').unwrap();
        let dist_end = after_if.find(')').ok_or_else(|| {
            ParseError::new(format!("Expected ')' in distance: {}", line), span)
        })?;
        let args_str = &after_if[dist_start + 1..dist_end];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        if args.len() != 2 {
            return Err(ParseError::new(
                format!("Expected two particle names in distance: {}", line),
                span,
            ));
        }
        
        let after_paren = &after_if[dist_end + 1..];
        if !after_paren.starts_with(" >= ") {
            return Err(ParseError::new(
                format!("Expected ' >= ' after distance: {}", line),
                span,
            ));
        }
        
        let after_ge = &after_paren[4..];
        let depth_pos = after_ge.find(" depth ").ok_or_else(|| {
            ParseError::new(format!("Expected 'depth' in well: {}", line), span)
        })?;
        let threshold_expr = parse_expr(after_ge[..depth_pos].trim(), span)?;
        let depth_expr = parse_expr(&after_ge[depth_pos + 7..].trim(), span)?;
        
        return Ok(WellDecl {
            name,
            particle,
            observable: ObservableExpr::Distance(args[0].to_string(), args[1].to_string()),
            threshold: threshold_expr,
            depth: depth_expr,
        });
    } else {
        return Err(ParseError::new(
            format!("Expected 'position(' or 'distance(' in well: {}", line),
            span,
        ));
    }
}

// ============================================================================
// v0.6: Expression Parsing
// ============================================================================

/// Parse a let declaration: `let name = expr`
fn parse_let(line: &str, span: Option<Span>) -> Result<LetDecl, ParseError> {
    let rest = line.strip_prefix("let ").ok_or_else(|| {
        ParseError::new("Expected 'let' keyword", span)
    })?;
    
    // Find " = "
    let eq_pos = rest.find(" = ").ok_or_else(|| {
        ParseError::new(format!("Expected '=' in let declaration: {}", line), span)
    })?;
    
    let name = rest[..eq_pos].trim().to_string();
    if name.is_empty() {
        return Err(ParseError::new(
            "Empty variable name in let declaration".to_string(),
            span,
        ));
    }
    
    let expr_str = rest[eq_pos + 3..].trim();
    let expr = parse_expr(expr_str, span)?;
    
    Ok(LetDecl { name, expr })
}

/// Parse an expression from a string
/// Grammar: ExprAdd (with precedence: add/sub < mul/div < unary < primary)
fn parse_expr(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    parse_expr_comparison(s.trim(), span)
}

/// Parse comparison operators (lowest precedence)
/// Supports: ==, !=, <, >, <=, >=
fn parse_expr_comparison(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    let s = s.trim();
    
    // Find comparison operators at paren depth 0
    // Check for two-character operators first (==, !=, <=, >=), then single-character
    let operators = [
        ("==", BinaryOp::Equal),
        ("!=", BinaryOp::NotEqual),
        ("<=", BinaryOp::LessEqual),
        (">=", BinaryOp::GreaterEqual),
        ("<", BinaryOp::LessThan),
        (">", BinaryOp::GreaterThan),
    ];
    
    let mut paren_depth = 0;
    let mut op_pos = None;
    let mut op = None;
    
    // Search from right to left for the rightmost comparison operator
    for (i, ch) in s.char_indices().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth -= 1,
            _ => {}
        }
        
        if paren_depth == 0 {
            // Check for two-character operators
            if i + 1 < s.len() {
                let two_char = &s[i..i + 2];
                for (op_str, op_type) in &operators[..4] {
                    if two_char == *op_str {
                        op_pos = Some(i);
                        op = Some(*op_type);
                        break;
                    }
                }
            }
            
            // Check for single-character operators if no two-char found
            if op_pos.is_none() {
                for (op_str, op_type) in &operators[4..] {
                    if s[i..].starts_with(*op_str) {
                        op_pos = Some(i);
                        op = Some(*op_type);
                        break;
                    }
                }
            }
        }
        
        if op_pos.is_some() {
            break;
        }
    }
    
    if let (Some(pos), Some(op_type)) = (op_pos, op) {
        let op_len = match op_type {
            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessEqual | BinaryOp::GreaterEqual => 2,
            _ => 1,
        };
        let left_str = s[..pos].trim();
        let right_str = s[pos + op_len..].trim();
        
        Ok(Expr::Binary {
            op: op_type,
            left: Box::new(parse_expr_comparison(left_str, span)?),
            right: Box::new(parse_expr_add(right_str, span)?),
        })
    } else {
        parse_expr_add(s, span)
    }
}

/// Parse addition/subtraction (lowest precedence)
/// Uses a simple recursive approach: find the rightmost + or - at depth 0
fn parse_expr_add(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    let s = s.trim();
    
    // Find the rightmost + or - operator at paren depth 0
    let mut paren_depth = 0;
    let mut op_pos = None;
    let mut op_char = None;
    
    for (i, ch) in s.char_indices().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth -= 1,
            '+' | '-' if paren_depth == 0 => {
                // Make sure it's not a unary minus (at the start or after an operator/paren)
                // Find the last non-space character before this position
                let prev_non_space = s[..i].trim_end().chars().last();
                if let Some(prev_ch) = prev_non_space {
                    // It's binary if the previous non-space char is not an operator or open paren
                    if prev_ch != '(' && prev_ch != ',' && prev_ch != '+' && prev_ch != '-' 
                        && prev_ch != '*' && prev_ch != '/' && prev_ch != '=' && prev_ch != '<'
                        && prev_ch != '>' && prev_ch != '!' {
                        op_pos = Some(i);
                        op_char = Some(ch);
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    
    if let Some(pos) = op_pos {
        let left_str = s[..pos].trim();
        let right_str = s[pos + 1..].trim();
        let op = if op_char.unwrap() == '+' {
            BinaryOp::Add
        } else {
            BinaryOp::Sub
        };
        
        Ok(Expr::Binary {
            op,
            left: Box::new(parse_expr_add(left_str, span)?),
            right: Box::new(parse_expr_mul(right_str, span)?),
        })
    } else {
        parse_expr_mul(s, span)
    }
}

/// Parse multiplication/division
/// Uses a simple recursive approach: find the rightmost * or / at depth 0
fn parse_expr_mul(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    let s = s.trim();
    
    // Find the rightmost * or / operator at paren depth 0
    let mut paren_depth = 0;
    let mut op_pos = None;
    let mut op_char = None;
    
    for (i, ch) in s.char_indices().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth -= 1,
            '*' | '/' if paren_depth == 0 => {
                op_pos = Some(i);
                op_char = Some(ch);
                break;
            }
            _ => {}
        }
    }
    
    if let Some(pos) = op_pos {
        let left_str = s[..pos].trim();
        let right_str = s[pos + 1..].trim();
        let op = if op_char.unwrap() == '*' {
            BinaryOp::Mul
        } else {
            BinaryOp::Div
        };
        
        Ok(Expr::Binary {
            op,
            left: Box::new(parse_expr_mul(left_str, span)?),
            right: Box::new(parse_expr_unary(right_str, span)?),
        })
    } else {
        parse_expr_unary(s, span)
    }
}

/// Parse unary minus
fn parse_expr_unary(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    let s = s.trim();
    if s.starts_with('-') {
        let inner = parse_expr_unary(&s[1..], span)?;
        Ok(Expr::UnaryMinus(Box::new(inner)))
    } else {
        parse_expr_primary(s, span)
    }
}

/// Parse primary expressions: literals, variables, function calls, parentheses
fn parse_expr_primary(s: &str, span: Option<Span>) -> Result<Expr, ParseError> {
    let s = s.trim();
    
    // Try parsing as float literal
    if let Ok(val) = s.parse::<f32>() {
        return Ok(Expr::Literal(val));
    }
    
    // Try parsing as string literal: "..."
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let content = &s[1..s.len()-1];
        return Ok(Expr::StringLiteral(content.to_string()));
    }
    
    // Try parsing as function call: ident(...)
    if let Some(paren_pos) = s.find('(') {
        let func_name = s[..paren_pos].trim();
        let rest = &s[paren_pos..];
        
        // Find matching closing paren
        let mut paren_count = 0;
        let mut end_pos = None;
        for (i, ch) in rest.chars().enumerate() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        end_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(end) = end_pos {
            let args_str = &rest[1..end];
            
            // Parse arguments (handle nested parentheses correctly)
            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                parse_comma_separated_exprs(args_str, span)?
            };
            
            // Check if it's a built-in function
            let builtin_func = match func_name {
                "sin" => Some(FuncName::Sin),
                "cos" => Some(FuncName::Cos),
                "sqrt" => Some(FuncName::Sqrt),
                "clamp" => Some(FuncName::Clamp),
                _ => None,
            };
            
            if let Some(func) = builtin_func {
                return Ok(Expr::Call { func, args });
            } else if is_valid_identifier(func_name) {
                // User-defined function call
                return Ok(Expr::UserCall { 
                    name: func_name.to_string(), 
                    args,
                });
            } else {
                return Err(ParseError::new(
                    format!("Invalid function name: '{}'", func_name),
                    span,
                ));
            }
        }
    }
    
    // Try parsing as variable (identifier)
    if is_valid_identifier(s) {
        return Ok(Expr::Var(s.to_string()));
    }
    
    // Try parsing as parenthesized expression: (expr)
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        return parse_expr(inner, span);
    }
    
    Err(ParseError::new(
        format!("Invalid expression: {}", s),
        span,
    ))
}


/// Parse comma-separated expressions, handling nested parentheses correctly
fn parse_comma_separated_exprs(s: &str, span: Option<Span>) -> Result<Vec<Expr>, ParseError> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;
    
    for ch in s.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if paren_depth == 0 => {
                if !current.trim().is_empty() {
                    args.push(parse_expr(current.trim(), span)?);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    
    // Don't forget the last argument
    if !current.trim().is_empty() {
        args.push(parse_expr(current.trim(), span)?);
    }
    
    Ok(args)
}

/// Check if a string is a valid identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    
    // First char must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    
    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Strip quotes from a string literal, returning the inner content
/// If not quoted, returns the original string
fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        // Remove surrounding quotes
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

// ============================================================================
// v0.7: Function and Statement Parsing
// ============================================================================

/// Parse a function declaration: `fn name(params) { body }`
fn parse_function(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(FunctionDecl, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    
    // Remove "fn " prefix
    let rest = line.strip_prefix("fn ").ok_or_else(|| {
        ParseError::new("Expected 'fn' keyword", Some(line_span))
    })?;
    
    // Find opening parenthesis
    let paren_start = rest.find('(').ok_or_else(|| {
        ParseError::new(format!("Expected '(' in function declaration: {}", line), Some(line_span))
    })?;
    
    let name = rest[..paren_start].trim().to_string();
    if !is_valid_identifier(&name) {
        return Err(ParseError::new(
            format!("Invalid function name: {}", name),
            Some(line_span),
        ));
    }
    
    // Find closing parenthesis
    let rest = &rest[paren_start + 1..];
    let paren_end = rest.find(')').ok_or_else(|| {
        ParseError::new(format!("Expected ')' in function declaration: {}", line), Some(line_span))
    })?;
    
    // Parse parameters
    let params_str = rest[..paren_end].trim();
    let params = if params_str.is_empty() {
        Vec::new()
    } else {
        params_str
            .split(',')
            .map(|p| {
                let p = p.trim();
                if !is_valid_identifier(p) {
                    return Err(ParseError::new(
                        format!("Invalid parameter name: {}", p),
                        Some(line_span),
                    ));
                }
                Ok(p.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    
    // Check for duplicate parameter names
    let mut param_set = std::collections::HashSet::new();
    for param in &params {
        if !param_set.insert(param) {
            return Err(ParseError::new(
                format!("Duplicate parameter name: {}", param),
                Some(line_span),
            ));
        }
    }
    
    // Find opening brace (could be on same line or next line)
    let after_paren = &rest[paren_end + 1..].trim();
    let body_start = if after_paren.starts_with('{') {
        // Opening brace on same line as function declaration: "fn foo() {"
        // Pass the same line to parse_block so it can see the '{'
        start_idx
    } else if after_paren.is_empty() {
        // Opening brace on next line
        if start_idx + 1 >= lines.len() {
            return Err(ParseError::new(
                "Expected '{' after function declaration".to_string(),
                Some(line_span),
            ));
        }
        let next_line = lines[start_idx + 1].trim();
        if !next_line.starts_with('{') {
            return Err(ParseError::new(
                "Expected '{' after function declaration".to_string(),
                Some(line_span),
            ));
        }
        start_idx + 1
    } else {
        return Err(ParseError::new(
            format!("Expected '{{' after function declaration: {}", line),
            Some(line_span),
        ));
    };
    
    trace_parse!("  function body_start: line {}", body_start + 1);
    
    // Parse function body (statements until closing brace)
    let (body, next_line) = parse_block(lines, body_start, ctx)?;
    
    Ok((
        FunctionDecl {
            name,
            params,
            body,
        },
        next_line,
    ))
}

/// Parse a block of statements: `{ stmt1 stmt2 ... }`
fn parse_block(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(Vec<Stmt>, usize), ParseError> {
    trace_parse!("parse_block starting at line {}", start_idx + 1);
    let mut stmts = Vec::new();
    let mut i = start_idx;
    let mut brace_count = 0;
    
    // Check if we need to skip opening brace
    if i < lines.len() {
        let first_line = lines[i].trim();
        trace_parse!("  block first_line: '{}'", first_line);
        
        if first_line == "{" || first_line.starts_with("{") {
            trace_parse!("  -> found opening brace at start");
            brace_count = 1;
            if first_line.len() > 1 {
                // Opening brace with content on same line
                let after_brace = first_line[1..].trim();
                if !after_brace.is_empty() && !after_brace.starts_with('#') {
                    trace_parse!("  -> content after brace: '{}'", after_brace);
                    // Try to parse statement on same line
                    let (stmt, _) = parse_stmt(&[after_brace], 0, ctx)?;
                    stmts.push(stmt);
                }
            }
            i += 1;
        } else if first_line.ends_with('{') && !first_line.starts_with('#') {
            // Line ends with '{' (e.g., "pattern => {" or "if condition {")
            // Treat '{' as opening brace and move to next line
            // Don't try to parse the line as a statement - it's just a brace opener
            trace_parse!("  -> line ends with '{{', treating as block start");
            brace_count = 1;
            i += 1;
        } else {
            trace_parse!("  -> WARNING: block doesn't start with '{{', first_line = '{}'", first_line);
        }
    }
    
    trace_parse!("  block parsing statements, brace_count={}, starting at line {}", brace_count, i + 1);
    
    // Parse statements until closing brace
    while i < lines.len() && brace_count > 0 {
        let line = lines[i].trim();
        
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }
        
        trace_parse!("  block line {}: '{}' (brace_count={})", i + 1, line, brace_count);
        
        // Check for closing brace of the block itself
        // Handle both "}" alone and "} else {" (for if-else statements)
        if line == "}" || line.starts_with("} else") {
            brace_count -= 1;
            trace_parse!("  -> closing brace found, brace_count now {}", brace_count);
            if brace_count == 0 {
                // Don't increment i - let the caller handle "} else {" or move past "}"
                if line == "}" {
                    i += 1;
                }
                // For "} else {", we leave i pointing at this line so the caller can process else
                break;
            }
            i += 1;
            continue;
        }
        
        // Try to parse as a statement
        trace_parse!("  -> attempting to parse as statement");
        match parse_stmt(lines, i, ctx) {
            Ok((stmt, next_i)) => {
                trace_parse!("  -> parsed statement, next line: {}", next_i + 1);
                stmts.push(stmt);
                i = next_i;
            }
            Err(e) => {
                trace_parse!("  -> parse_stmt failed: {}", e);
                // If it's not a statement, it might be a closing brace variant
                if line == "}" || line.starts_with("} else") {
                    brace_count -= 1;
                    if brace_count == 0 {
                        if line == "}" {
                            i += 1;
                        }
                        break;
                    }
                    i += 1;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    if brace_count > 0 {
        return Err(ctx.error(
            format!("Unclosed block (brace_count={}, last line={})", brace_count, i + 1),
            start_idx,
            "parse_block",
        ));
    }
    
    trace_parse!("  parse_block complete, {} statements, ended at line {}", stmts.len(), i + 1);
    Ok((stmts, i))
}

/// Parse an if statement: `if condition { then } else { else }`
fn parse_if_stmt(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(Stmt, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    
    trace_parse!("parse_if_stmt starting at line {}: '{}'", start_idx + 1, line);
    
    // Parse: if condition {
    if !line.starts_with("if ") {
        return Err(ctx.error("Expected 'if' keyword", start_idx, "parse_if_stmt"));
    }
    
    // Find the opening brace
    let brace_pos = line.find('{').ok_or_else(|| {
        ctx.error("Expected '{' after if condition", start_idx, "parse_if_stmt")
    })?;
    
    let condition_str = line[3..brace_pos].trim();
    trace_parse!("  if condition: '{}'", condition_str);
    let condition = parse_expr(condition_str, Some(line_span))?;
    
    // Parse then branch - pass the line with '{' to parse_block
    // so it can correctly initialize brace_count
    let then_start = start_idx;
    trace_parse!("  if then_start: line {}", then_start + 1);
    
    let (then_branch, after_then) = parse_block(lines, then_start, ctx)?;
    trace_parse!("  if then branch: {} statements, after_then: line {}", then_branch.len(), after_then + 1);
    
    // Check for else
    // The block may have stopped at "} else {" on the same line
    let mut else_branch = Vec::new();
    let mut next_line = after_then;
    
    if after_then < lines.len() {
        let else_line = lines[after_then].trim();
        trace_parse!("  checking for else at line {}: '{}'", after_then + 1, else_line);
        
        // Handle "} else {" on the same line (block stopped at this line without incrementing)
        if else_line.starts_with("} else") {
            trace_parse!("  -> found '}} else' on same line");
            if else_line == "} else {" || else_line.starts_with("} else {") {
                // Parse else block - the line has "} else {", treat it as block start
                let (else_body, after_else) = parse_block(lines, after_then, ctx)?;
                else_branch = else_body;
                next_line = after_else;
                trace_parse!("  -> else branch: {} statements, next_line: {}", else_branch.len(), next_line + 1);
            } else if else_line == "} else" {
                // "} else" alone, opening brace on next line
                if after_then + 1 < lines.len() {
                    let (else_body, after_else) = parse_block(lines, after_then + 1, ctx)?;
                    else_branch = else_body;
                    next_line = after_else;
                }
            }
        } else if else_line.starts_with("else") {
            if else_line == "else" || else_line == "else {" || else_line.starts_with("else {") {
                // Parse else block - pass the line with "else {" to parse_block
                trace_parse!("  -> found else block");
                let (else_body, after_else) = parse_block(lines, after_then, ctx)?;
                else_branch = else_body;
                next_line = after_else;
                trace_parse!("  -> else branch: {} statements, next_line: {}", else_branch.len(), next_line + 1);
            } else {
                // else with condition on same line (not supported in v0.8)
                return Err(ctx.error(
                    "else if not supported in v0.8",
                    after_then,
                    "parse_if_stmt",
                ));
            }
        }
    }
    
    trace_parse!("  parse_if_stmt complete, next_line: {}", next_line + 1);
    Ok((
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        },
        next_line,
    ))
}

/// Parse a for statement: `for var in start..end { body }`
fn parse_for_stmt(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(Stmt, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    
    // Parse: for var in start..end {
    if !line.starts_with("for ") {
        return Err(ParseError::new("Expected 'for' keyword", Some(line_span)));
    }
    
    let after_for = &line[4..];
    
    // Find " in "
    let in_pos = after_for.find(" in ").ok_or_else(|| {
        ParseError::new("Expected ' in ' after for variable", Some(line_span))
    })?;
    
    let var_name = after_for[..in_pos].trim();
    if !is_valid_identifier(var_name) {
        return Err(ParseError::new(
            format!("Invalid variable name in for loop: {}", var_name),
            Some(line_span),
        ));
    }
    
    let after_in = &after_for[in_pos + 4..];
    
    // Find ".."
    let dotdot_pos = after_in.find("..").ok_or_else(|| {
        ParseError::new("Expected '..' in for loop range", Some(line_span))
    })?;
    
    let start_str = after_in[..dotdot_pos].trim();
    let after_dotdot = &after_in[dotdot_pos + 2..];
    
    // Find opening brace
    let brace_pos = after_dotdot.find('{').ok_or_else(|| {
        ParseError::new("Expected '{' after for loop range", Some(line_span))
    })?;
    
    let end_str = after_dotdot[..brace_pos].trim();
    
    let start = parse_expr(start_str, Some(line_span))?;
    let end = parse_expr(end_str, Some(line_span))?;
    
    // Parse body block - pass the line with '{' to parse_block
    let body_start = start_idx;
    trace_parse!("  for body_start: line {}", body_start + 1);
    
    let (body, next_line) = parse_block(lines, body_start, ctx)?;
    
    Ok((
        Stmt::For {
            var_name: var_name.to_string(),
            start,
            end,
            body,
        },
        next_line,
    ))
}

/// Parse a match statement: `match expr { arms }`
fn parse_match_stmt(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(Stmt, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    
    trace_parse!("parse_match_stmt starting at line {}: '{}'", start_idx + 1, line);
    
    // Parse: match expr {
    if !line.starts_with("match ") {
        return Err(ctx.error("Expected 'match' keyword", start_idx, "parse_match_stmt"));
    }
    
    let after_match = &line[6..];
    
    // Find opening brace
    let brace_pos = after_match.find('{').ok_or_else(|| {
        ctx.error("Expected '{' after match expression", start_idx, "parse_match_stmt")
    })?;
    
    let scrutinee_str = after_match[..brace_pos].trim();
    trace_parse!("  match scrutinee: '{}'", scrutinee_str);
    let scrutinee = parse_expr(scrutinee_str, Some(line_span))?;
    
    // Parse match arms
    let mut arms = Vec::new();
    let mut i = if after_match[brace_pos + 1..].trim().is_empty() {
        start_idx + 1
    } else {
        start_idx
    };
    
    trace_parse!("  parsing match arms starting at line {}", i + 1);
    
    // Find the closing brace of the match
    // We start with brace_count = 1 because we've seen the opening brace of the match
    let mut brace_count = 1;
    let match_start = i;
    
    while i < lines.len() && brace_count > 0 {
        let arm_line = lines[i].trim();
        
        if arm_line.is_empty() || arm_line.starts_with('#') {
            i += 1;
            continue;
        }
        
        trace_parse!("  match line {}: '{}' (brace_count={})", i + 1, arm_line, brace_count);
        
        // Check for closing brace of the match statement itself
        if arm_line == "}" && brace_count == 1 {
            // This is the closing brace of the match statement
            trace_parse!("  -> found closing brace of match statement");
            brace_count = 0;
            i += 1;
            break;
        }
        
        // Parse arm: pattern => { body }
        if arm_line.contains("=>") {
            let arrow_pos = arm_line.find("=>").unwrap();
            let pattern_str = arm_line[..arrow_pos].trim();
            let after_arrow = arm_line[arrow_pos + 2..].trim();
            
            trace_parse!("  -> match arm pattern: '{}', after_arrow: '{}'", pattern_str, after_arrow);
            
            // Parse pattern
            let pattern = if pattern_str == "_" {
                MatchPattern::Wildcard
            } else {
                // Try parsing as integer literal
                let int_val = pattern_str.parse::<i64>().map_err(|_| {
                    ctx.error(
                        format!("Match pattern must be integer literal or '_': '{}'", pattern_str),
                        i,
                        "parse_match_stmt",
                    )
                })?;
                MatchPattern::Literal(int_val)
            };
            
            // Parse body block
            let body_start = if after_arrow == "{" || after_arrow.starts_with("{") {
                // Body starts on same line: "pattern => {"
                trace_parse!("  -> arm body starts on same line (line {})", i + 1);
                i
            } else if after_arrow.is_empty() {
                // Body starts on next line - need to check if next line has opening brace
                if i + 1 < lines.len() && lines[i + 1].trim().starts_with('{') {
                    trace_parse!("  -> arm body starts on next line (line {})", i + 2);
                    i + 1
                } else {
                    return Err(ctx.error(
                        format!("Expected '{{' after match arm pattern, but got: '{}'", 
                            if i + 1 < lines.len() { lines[i + 1].trim() } else { "<EOF>" }),
                        i,
                        "parse_match_stmt",
                    ));
                }
            } else {
                // Body starts on same line after =>
                return Err(ctx.error(
                    format!("Match arm body must be '{{' or on new line, got: '{}'", after_arrow),
                    i,
                    "parse_match_stmt",
                ));
            };
            
            trace_parse!("  -> calling parse_block for arm body at line {}", body_start + 1);
            let (body, after_body) = parse_block(lines, body_start, ctx)?;
            trace_parse!("  -> arm body parsed, {} statements, next line: {}", body.len(), after_body + 1);
            
            arms.push(MatchArm { pattern, body });
            i = after_body;
        } else {
            // Unexpected line - might be a syntax error
            return Err(ctx.error(
                format!("Unexpected token in match statement: '{}'. Expected pattern => {{ body }} or '}}'", arm_line),
                i,
                "parse_match_stmt",
            ));
        }
    }
    
    if brace_count > 0 {
        return Err(ctx.error(
            format!("Unclosed match statement (started at line {})", match_start + 1),
            match_start,
            "parse_match_stmt",
        ));
    }
    
    trace_parse!("  parse_match_stmt complete, {} arms, ended at line {}", arms.len(), i + 1);
    Ok((
        Stmt::Match {
            scrutinee,
            arms,
        },
        i,
    ))
}

/// Parse a statement
fn parse_stmt(
    lines: &[&str],
    start_idx: usize,
    ctx: &ParseContext,
) -> Result<(Stmt, usize), ParseError> {
    let line = lines[start_idx].trim();
    let line_span = ctx.full_line_span(start_idx);
    
    trace_parse!("parse_stmt line {}: '{}'", start_idx + 1, line);
    
    // Remove semicolon if present (for return statements)
    let line_no_semi = line.strip_suffix(';').unwrap_or(line).trim();
    
    // v0.8: Parse control flow statements
    if line_no_semi.starts_with("if ") {
        trace_parse!("  -> if statement");
        return parse_if_stmt(lines, start_idx, ctx);
    } else if line_no_semi.starts_with("for ") {
        trace_parse!("  -> for loop");
        return parse_for_stmt(lines, start_idx, ctx);
    } else if line_no_semi.starts_with("match ") {
        trace_parse!("  -> match statement");
        return parse_match_stmt(lines, start_idx, ctx);
    } else if line_no_semi.starts_with("let ") {
        trace_parse!("  -> let declaration");
        let let_decl = parse_let(line_no_semi, Some(line_span))?;
        Ok((
            Stmt::Let {
                name: let_decl.name,
                expr: let_decl.expr,
            },
            start_idx + 1,
        ))
    } else if line_no_semi.starts_with("return ") {
        trace_parse!("  -> return statement");
        let expr_str = line_no_semi.strip_prefix("return ").ok_or_else(|| {
            ctx.error_simple("Expected 'return' keyword", start_idx)
        })?;
        let expr = parse_expr(expr_str.trim(), Some(line_span))?;
        Ok((Stmt::Return(expr), start_idx + 1))
    } else if line_no_semi.starts_with("particle ") {
        trace_parse!("  -> particle declaration");
        let particle = parse_particle(line_no_semi, Some(line_span))?;
        Ok((Stmt::ParticleDecl(particle), start_idx + 1))
    } else if line_no_semi.starts_with("force ") && !line_no_semi.contains("push") {
        trace_parse!("  -> force declaration");
        let force = parse_force(line_no_semi, Some(line_span))?;
        Ok((Stmt::ForceDecl(force), start_idx + 1))
    } else if line_no_semi.starts_with("detect ") {
        trace_parse!("  -> detect declaration");
        let detector = parse_detector(line_no_semi, Some(line_span))?;
        Ok((Stmt::DetectorDecl(detector), start_idx + 1))
    } else if line_no_semi.starts_with("well ") {
        trace_parse!("  -> well declaration");
        let well = parse_well(line_no_semi, Some(line_span))?;
        Ok((Stmt::WellDecl(well), start_idx + 1))
    } else if line_no_semi.starts_with("loop ") {
        trace_parse!("  -> loop declaration");
        let (loop_decl, next_line) = parse_loop(lines, start_idx, ctx)?;
        Ok((Stmt::LoopDecl(loop_decl), next_line))
    } else {
        // Try parsing as function call: ident(args)
        if let Some(paren_pos) = line_no_semi.find('(') {
            let func_name = line_no_semi[..paren_pos].trim();
            if is_valid_identifier(func_name) {
                trace_parse!("  -> function call: {}", func_name);
                let rest = &line_no_semi[paren_pos..];
                let paren_end = rest.find(')').ok_or_else(|| {
                    ctx.error(
                        format!("Expected ')' in function call"),
                        start_idx,
                        "parse_stmt",
                    )
                })?;
                
                let args_str = &rest[1..paren_end];
                let args = if args_str.trim().is_empty() {
                    Vec::new()
                } else {
                    args_str
                        .split(',')
                        .map(|arg| parse_expr(arg.trim(), Some(line_span)))
                        .collect::<Result<Vec<_>, _>>()?
                };
                
                return Ok((
                    Stmt::ExprCall {
                        name: func_name.to_string(),
                        args,
                    },
                    start_idx + 1,
                ));
            }
        }
        
        trace_parse!("  -> FAILED: no valid statement pattern matched");
        Err(ctx.error(
            format!("Invalid statement: '{}'", line),
            start_idx,
            "parse_stmt",
        ))
    }
}
