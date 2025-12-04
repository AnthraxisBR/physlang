use crate::ast::{
    BinaryOp, ConditionExpr, DetectorDecl, DetectorKind, Expr, ForceDecl, ForceKind, FuncName,
    LetDecl, LoopBodyStmt, LoopDecl, LoopKind, ObservableExpr, ParticleDecl, Program,
    SimulateDecl, WellDecl,
};
use crate::diagnostics::Span;
use glam::Vec2;
use thiserror::Error;

/// Parse error with optional span information
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{message}")]
    SyntaxError {
        message: String,
        span: Option<Span>,
    },
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span,
        }
    }

    pub fn message(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            span: None,
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::SyntaxError { span, .. } => *span,
        }
    }
}

/// Helper to track byte offsets while parsing
struct ParseContext {
    #[allow(dead_code)]
    source: String,
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
}

/// Parse a PhysLang program from source code
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    let ctx = ParseContext::new(source);
    let mut lets = Vec::new();
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

        if line.starts_with("let ") {
            lets.push(parse_let(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("particle ") {
            particles.push(parse_particle(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("force ") && !line.contains("push") {
            forces.push(parse_force(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("simulate ") {
            simulate = Some(parse_simulate(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("detect ") {
            detectors.push(parse_detector(line, Some(line_span))?);
            i += 1;
        } else if line.starts_with("loop ") {
            let (loop_decl, next_line) = parse_loop(&lines, i, &ctx)?;
            loops.push(loop_decl);
            i = next_line;
        } else if line.starts_with("well ") {
            wells.push(parse_well(line, Some(line_span))?);
            i += 1;
        } else {
            return Err(ParseError::new(
                format!("Unexpected token: {}", line.split_whitespace().next().unwrap_or("")),
                Some(line_span),
            ));
        }
    }

    let simulate = simulate.ok_or_else(|| {
        ParseError::message("Missing 'simulate' declaration".to_string())
    })?;

    Ok(Program {
        lets,
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
    
    let a = args[0].to_string();
    let b = args[1].to_string();
    
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
    parse_expr_add(s.trim(), span)
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
                // Make sure it's not a unary minus
                if i > 0 {
                    let prev_ch = s[..i].chars().last().unwrap();
                    if prev_ch != ' ' && prev_ch != '(' && prev_ch != ',' {
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
            let func = match func_name {
                "sin" => FuncName::Sin,
                "cos" => FuncName::Cos,
                "sqrt" => FuncName::Sqrt,
                "clamp" => FuncName::Clamp,
                _ => {
                    return Err(ParseError::new(
                        format!("Unknown function '{}'", func_name),
                        span,
                    ));
                }
            };
            
            // Parse arguments
            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                args_str
                    .split(',')
                    .map(|arg| parse_expr(arg.trim(), span))
                    .collect::<Result<Vec<_>, _>>()?
            };
            
            return Ok(Expr::Call { func, args });
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
