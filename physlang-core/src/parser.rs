use crate::ast::{
    ConditionExpr, DetectorDecl, DetectorKind, ForceDecl, ForceKind, LoopBodyStmt, LoopDecl,
    LoopKind, ObservableExpr, ParticleDecl, Program, SimulateDecl, WellDecl,
};
use glam::Vec2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Message(String),
}

/// Parse a PhysLang program from source code
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
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
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("particle ") {
            particles.push(parse_particle(line)?);
            i += 1;
        } else if line.starts_with("force ") && !line.contains("push") {
            forces.push(parse_force(line)?);
            i += 1;
        } else if line.starts_with("simulate ") {
            simulate = Some(parse_simulate(line)?);
            i += 1;
        } else if line.starts_with("detect ") {
            detectors.push(parse_detector(line)?);
            i += 1;
        } else if line.starts_with("loop ") {
            let (loop_decl, next_line) = parse_loop(&lines, i)?;
            loops.push(loop_decl);
            i = next_line;
        } else if line.starts_with("well ") {
            wells.push(parse_well(line)?);
            i += 1;
        } else {
            i += 1;
        }
    }

    let simulate = simulate.ok_or_else(|| {
        ParseError::Message("Missing 'simulate' declaration".to_string())
    })?;

    Ok(Program {
        particles,
        forces,
        simulate,
        detectors,
        loops,
        wells,
    })
}

/// Parse a particle declaration: `particle name at (x, y) mass m`
fn parse_particle(line: &str) -> Result<ParticleDecl, ParseError> {
    // Remove "particle " prefix
    let rest = line.strip_prefix("particle ").unwrap();
    
    // Find " at "
    let at_pos = rest.find(" at ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'at' in particle declaration: {}", line))
    })?;
    
    let name = rest[..at_pos].trim().to_string();
    let rest = &rest[at_pos + 4..];
    
    // Parse position: (x, y)
    let pos_start = rest.find('(').ok_or_else(|| {
        ParseError::Message(format!("Expected '(' in position: {}", line))
    })?;
    let pos_end = rest.find(')').ok_or_else(|| {
        ParseError::Message(format!("Expected ')' in position: {}", line))
    })?;
    
    let pos_str = &rest[pos_start + 1..pos_end];
    let coords: Vec<&str> = pos_str.split(',').map(|s| s.trim()).collect();
    if coords.len() != 2 {
        return Err(ParseError::Message(format!(
            "Expected two coordinates in position: {}",
            line
        )));
    }
    
    let x: f32 = coords[0].parse().map_err(|_| {
        ParseError::Message(format!("Invalid x coordinate: {}", coords[0]))
    })?;
    let y: f32 = coords[1].parse().map_err(|_| {
        ParseError::Message(format!("Invalid y coordinate: {}", coords[1]))
    })?;
    
    let rest = &rest[pos_end + 1..];
    
    // Parse mass
    let mass_start = rest.find("mass ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'mass' in particle declaration: {}", line))
    })?;
    
    let mass_str = &rest[mass_start + 5..].trim();
    let mass: f32 = mass_str.parse().map_err(|_| {
        ParseError::Message(format!("Invalid mass value: {}", mass_str))
    })?;
    
    Ok(ParticleDecl {
        name,
        position: Vec2::new(x, y),
        mass,
    })
}

/// Parse a force declaration: `force gravity(a, b) G = x` or `force spring(a, b) k = x rest = y`
fn parse_force(line: &str) -> Result<ForceDecl, ParseError> {
    // Remove "force " prefix
    let rest = line.strip_prefix("force ").unwrap();
    
    // Find the opening parenthesis
    let paren_start = rest.find('(').ok_or_else(|| {
        ParseError::Message(format!("Expected '(' in force declaration: {}", line))
    })?;
    
    let force_type = rest[..paren_start].trim();
    let rest = &rest[paren_start + 1..];
    
    // Find closing parenthesis
    let paren_end = rest.find(')').ok_or_else(|| {
        ParseError::Message(format!("Expected ')' in force declaration: {}", line))
    })?;
    
    let args_str = &rest[..paren_end];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    if args.len() != 2 {
        return Err(ParseError::Message(format!(
            "Expected two particle names in force: {}",
            line
        )));
    }
    
    let a = args[0].to_string();
    let b = args[1].to_string();
    
    let rest = &rest[paren_end + 1..].trim();
    
    let kind = match force_type {
        "gravity" => {
            // Parse: G = value
            let g_str = rest.strip_prefix("G = ").ok_or_else(|| {
                ParseError::Message(format!("Expected 'G =' in gravity force: {}", line))
            })?;
            let g: f32 = g_str.trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid G value: {}", g_str))
            })?;
            ForceKind::Gravity { g }
        }
        "spring" => {
            // Parse: k = value rest = value
            let k_start = rest.find("k = ").ok_or_else(|| {
                ParseError::Message(format!("Expected 'k =' in spring force: {}", line))
            })?;
            let after_k = &rest[k_start + 4..];
            let k_end = after_k.find(" rest = ").ok_or_else(|| {
                ParseError::Message(format!("Expected 'rest =' in spring force: {}", line))
            })?;
            
            let k_str = &after_k[..k_end].trim();
            let k: f32 = k_str.parse().map_err(|_| {
                ParseError::Message(format!("Invalid k value: {}", k_str))
            })?;
            
            let rest_str = &after_k[k_end + 8..].trim();
            let rest: f32 = rest_str.parse().map_err(|_| {
                ParseError::Message(format!("Invalid rest value: {}", rest_str))
            })?;
            
            ForceKind::Spring { k, rest }
        }
        _ => {
            return Err(ParseError::Message(format!(
                "Unknown force type: {}",
                force_type
            )));
        }
    };
    
    Ok(ForceDecl { a, b, kind })
}

/// Parse a simulate declaration: `simulate dt = x steps = n`
fn parse_simulate(line: &str) -> Result<SimulateDecl, ParseError> {
    // Remove "simulate " prefix
    let rest = line.strip_prefix("simulate ").unwrap();
    
    // Parse dt = value
    let dt_start = rest.find("dt = ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'dt =' in simulate: {}", line))
    })?;
    let after_dt = &rest[dt_start + 5..];
    let dt_end = after_dt.find(" steps = ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'steps =' in simulate: {}", line))
    })?;
    
    let dt_str = &after_dt[..dt_end].trim();
    let dt: f32 = dt_str.parse().map_err(|_| {
        ParseError::Message(format!("Invalid dt value: {}", dt_str))
    })?;
    
    let steps_str = &after_dt[dt_end + 9..].trim();
    let steps: usize = steps_str.parse().map_err(|_| {
        ParseError::Message(format!("Invalid steps value: {}", steps_str))
    })?;
    
    Ok(SimulateDecl { dt, steps })
}

/// Parse a detector declaration: `detect name = position(a)` or `detect name = distance(a, b)`
fn parse_detector(line: &str) -> Result<DetectorDecl, ParseError> {
    // Remove "detect " prefix
    let rest = line.strip_prefix("detect ").unwrap();
    
    // Find " = "
    let eq_pos = rest.find(" = ").ok_or_else(|| {
        ParseError::Message(format!("Expected '=' in detector: {}", line))
    })?;
    
    let name = rest[..eq_pos].trim().to_string();
    let rest = &rest[eq_pos + 3..].trim();
    
    let kind = if rest.starts_with("position(") {
        // Parse: position(name)
        let start = rest.find('(').unwrap();
        let end = rest.find(')').ok_or_else(|| {
            ParseError::Message(format!("Expected ')' in position detector: {}", line))
        })?;
        let particle_name = rest[start + 1..end].trim().to_string();
        DetectorKind::Position(particle_name)
    } else if rest.starts_with("distance(") {
        // Parse: distance(a, b)
        let start = rest.find('(').unwrap();
        let end = rest.find(')').ok_or_else(|| {
            ParseError::Message(format!("Expected ')' in distance detector: {}", line))
        })?;
        let args_str = &rest[start + 1..end];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        if args.len() != 2 {
            return Err(ParseError::Message(format!(
                "Expected two particle names in distance detector: {}",
                line
            )));
        }
        DetectorKind::Distance {
            a: args[0].to_string(),
            b: args[1].to_string(),
        }
    } else {
        return Err(ParseError::Message(format!(
            "Unknown detector type: {}",
            rest
        )));
    };
    
    Ok(DetectorDecl { name, kind })
}

// ============================================================================
// v0.2: Loop and Well Parsing
// ============================================================================

/// Parse a loop declaration (handles multi-line bodies)
fn parse_loop(lines: &[&str], start_idx: usize) -> Result<(LoopDecl, usize), ParseError> {
    let line = lines[start_idx].trim();
    let rest = line.strip_prefix("loop ").unwrap();

    let (kind, body_start) = if rest.starts_with("for ") {
        // `loop for <integer> cycles with frequency <float> damping <float> on <ident> {`
        let after_for = rest.strip_prefix("for ").unwrap();
        
        // Find " cycles"
        let cycles_end = after_for.find(" cycles").ok_or_else(|| {
            ParseError::Message(format!("Expected 'cycles' in for loop: {}", line))
        })?;
        let cycles: u32 = after_for[..cycles_end].trim().parse().map_err(|_| {
            ParseError::Message(format!("Invalid cycle count: {}", &after_for[..cycles_end]))
        })?;
        
        let after_cycles = &after_for[cycles_end + 7..];
        
        // Find "with frequency"
        let freq_start = after_cycles.find("with frequency ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'with frequency' in for loop: {}", line))
        })?;
        let after_freq = &after_cycles[freq_start + 15..];
        
        // Find frequency value
        let freq_end = after_freq.find(" damping ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'damping' after frequency: {}", line))
        })?;
        let frequency: f32 = after_freq[..freq_end].trim().parse().map_err(|_| {
            ParseError::Message(format!("Invalid frequency: {}", &after_freq[..freq_end]))
        })?;
        
        let after_damp = &after_freq[freq_end + 9..];
        
        // Find damping value
        let damp_end = after_damp.find(" on ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'on' after damping: {}", line))
        })?;
        let damping: f32 = after_damp[..damp_end].trim().parse().map_err(|_| {
            ParseError::Message(format!("Invalid damping: {}", &after_damp[..damp_end]))
        })?;
        
        // Find particle name
        let after_on = &after_damp[damp_end + 4..];
        let target = if after_on.ends_with(" {") {
            after_on[..after_on.len() - 2].trim().to_string()
        } else if after_on.ends_with('{') {
            after_on[..after_on.len() - 1].trim().to_string()
        } else {
            return Err(ParseError::Message(format!(
                "Expected '{{' after particle name: {}",
                line
            )));
        };
        
        (LoopKind::ForCycles {
            cycles,
            frequency,
            damping,
            target,
        }, start_idx + 1)
    } else if rest.starts_with("while ") {
        // `loop while <condition> with frequency <float> damping <float> on <ident> {`
        let after_while = rest.strip_prefix("while ").unwrap();
        
        // Find " with frequency"
        let with_pos = after_while.find(" with frequency ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'with frequency' in while loop: {}", line))
        })?;
        
        let condition_str = &after_while[..with_pos];
        let condition = parse_condition(condition_str.trim())?;
        
        let after_with = &after_while[with_pos + 16..];
        
        // Parse frequency
        let freq_end = after_with.find(" damping ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'damping' after frequency: {}", line))
        })?;
        let frequency: f32 = after_with[..freq_end].trim().parse().map_err(|_| {
            ParseError::Message(format!("Invalid frequency: {}", &after_with[..freq_end]))
        })?;
        
        let after_damp = &after_with[freq_end + 9..];
        
        // Parse damping
        let damp_end = after_damp.find(" on ").ok_or_else(|| {
            ParseError::Message(format!("Expected 'on' after damping: {}", line))
        })?;
        let damping: f32 = after_damp[..damp_end].trim().parse().map_err(|_| {
            ParseError::Message(format!("Invalid damping: {}", &after_damp[..damp_end]))
        })?;
        
        // Parse target particle
        let after_on = &after_damp[damp_end + 4..];
        let target = if after_on.ends_with(" {") {
            after_on[..after_on.len() - 2].trim().to_string()
        } else if after_on.ends_with('{') {
            after_on[..after_on.len() - 1].trim().to_string()
        } else {
            return Err(ParseError::Message(format!(
                "Expected '{{' after particle name: {}",
                line
            )));
        };
        
        (LoopKind::WhileCondition {
            condition,
            frequency,
            damping,
            target,
        }, start_idx + 1)
    } else {
        return Err(ParseError::Message(format!("Unknown loop type: {}", line)));
    };

    // Parse loop body (lines until closing brace)
    let mut body = Vec::new();
    let mut i = body_start;
    let mut brace_count = 1; // We've seen the opening brace
    
    while i < lines.len() && brace_count > 0 {
        let body_line = lines[i].trim();
        
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
                body.push(parse_loop_body_stmt(body_line)?);
            }
        }
        
        i += 1;
    }
    
    if brace_count > 0 {
        return Err(ParseError::Message("Unclosed loop body".to_string()));
    }
    
    Ok((LoopDecl {
        name: None, // v0.2 ignores loop labels
        kind,
        body,
    }, i))
}

/// Parse a condition expression
fn parse_condition(cond_str: &str) -> Result<ConditionExpr, ParseError> {
    // Try position(<ident>).x < float or position(<ident>).x > float
    if cond_str.contains("position(") {
        let pos_start = cond_str.find("position(").unwrap();
        let pos_end = cond_str.find(')').ok_or_else(|| {
            ParseError::Message(format!("Expected ')' in position condition: {}", cond_str))
        })?;
        let particle_name = cond_str[pos_start + 9..pos_end].trim().to_string();
        
        let after_paren = &cond_str[pos_end + 1..];
        if after_paren.starts_with(".x < ") {
            let threshold: f32 = after_paren[5..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &after_paren[5..]))
            })?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::PositionX(particle_name),
                threshold,
            ));
        } else if after_paren.starts_with(".x > ") {
            let threshold: f32 = after_paren[5..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &after_paren[5..]))
            })?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::PositionX(particle_name),
                threshold,
            ));
        } else if after_paren.starts_with(".y < ") {
            let threshold: f32 = after_paren[5..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &after_paren[5..]))
            })?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::PositionY(particle_name),
                threshold,
            ));
        } else if after_paren.starts_with(".y > ") {
            let threshold: f32 = after_paren[5..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &after_paren[5..]))
            })?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::PositionY(particle_name),
                threshold,
            ));
        }
    }
    
    // Try distance(a, b) < float or distance(a, b) > float
    if cond_str.starts_with("distance(") {
        let after_dist = cond_str.strip_prefix("distance(").unwrap();
        let paren_end = after_dist.find(')').ok_or_else(|| {
            ParseError::Message(format!("Expected ')' in distance condition: {}", cond_str))
        })?;
        let args_str = &after_dist[..paren_end];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        if args.len() != 2 {
            return Err(ParseError::Message(format!(
                "Expected two particle names in distance condition: {}",
                cond_str
            )));
        }
        
        let rest = &after_dist[paren_end + 1..].trim();
        if rest.starts_with("< ") {
            let threshold: f32 = rest[2..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &rest[2..]))
            })?;
            return Ok(ConditionExpr::LessThan(
                ObservableExpr::Distance(args[0].to_string(), args[1].to_string()),
                threshold,
            ));
        } else if rest.starts_with("> ") {
            let threshold: f32 = rest[2..].trim().parse().map_err(|_| {
                ParseError::Message(format!("Invalid threshold: {}", &rest[2..]))
            })?;
            return Ok(ConditionExpr::GreaterThan(
                ObservableExpr::Distance(args[0].to_string(), args[1].to_string()),
                threshold,
            ));
        }
    }
    
    Err(ParseError::Message(format!("Unknown condition format: {}", cond_str)))
}

/// Parse a loop body statement
fn parse_loop_body_stmt(line: &str) -> Result<LoopBodyStmt, ParseError> {
    // `force push(<ident>) magnitude <float> direction (<float>, <float>)`
    let rest = line.strip_prefix("force push(").unwrap();
    
    let paren_end = rest.find(')').ok_or_else(|| {
        ParseError::Message(format!("Expected ')' in push force: {}", line))
    })?;
    let particle = rest[..paren_end].trim().to_string();
    
    let rest = &rest[paren_end + 1..].trim();
    
    // Parse magnitude
    let mag_start = rest.find("magnitude ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'magnitude' in push force: {}", line))
    })?;
    let after_mag = &rest[mag_start + 10..];
    let mag_end = after_mag.find(" direction ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'direction' in push force: {}", line))
    })?;
    let magnitude: f32 = after_mag[..mag_end].trim().parse().map_err(|_| {
        ParseError::Message(format!("Invalid magnitude: {}", &after_mag[..mag_end]))
    })?;
    
    // Parse direction
    let after_dir = &after_mag[mag_end + 11..];
    let dir_start = after_dir.find('(').ok_or_else(|| {
        ParseError::Message(format!("Expected '(' in direction: {}", line))
    })?;
    let dir_end = after_dir.find(')').ok_or_else(|| {
        ParseError::Message(format!("Expected ')' in direction: {}", line))
    })?;
    let dir_str = &after_dir[dir_start + 1..dir_end];
    let coords: Vec<&str> = dir_str.split(',').map(|s| s.trim()).collect();
    if coords.len() != 2 {
        return Err(ParseError::Message(format!(
            "Expected two coordinates in direction: {}",
            line
        )));
    }
    let x: f32 = coords[0].parse().map_err(|_| {
        ParseError::Message(format!("Invalid x coordinate: {}", coords[0]))
    })?;
    let y: f32 = coords[1].parse().map_err(|_| {
        ParseError::Message(format!("Invalid y coordinate: {}", coords[1]))
    })?;
    
    Ok(LoopBodyStmt::ForcePush {
        particle,
        magnitude,
        direction: Vec2::new(x, y),
    })
}

/// Parse a well declaration: `well <name> on <ident> if position(<ident>).x >= <float> depth <float>`
fn parse_well(line: &str) -> Result<WellDecl, ParseError> {
    // Remove "well " prefix
    let rest = line.strip_prefix("well ").unwrap();
    
    // Find " on "
    let on_pos = rest.find(" on ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'on' in well declaration: {}", line))
    })?;
    let name = rest[..on_pos].trim().to_string();
    
    let after_on = &rest[on_pos + 4..];
    
    // Find " if position("
    let if_pos = after_on.find(" if position(").ok_or_else(|| {
        ParseError::Message(format!("Expected 'if position(' in well: {}", line))
    })?;
    let particle = after_on[..if_pos].trim().to_string();
    
    let after_if = &after_on[if_pos + 13..];
    
    // Find closing paren for position
    let pos_end = after_if.find(')').ok_or_else(|| {
        ParseError::Message(format!("Expected ')' in position: {}", line))
    })?;
    let pos_particle = after_if[..pos_end].trim().to_string();
    
    // Check for ".x >= "
    let after_paren = &after_if[pos_end + 1..];
    if !after_paren.starts_with(".x >= ") {
        return Err(ParseError::Message(format!(
            "Expected '.x >= ' after position: {}",
            line
        )));
    }
    
    let after_x = &after_paren[6..];
    
    // Find " depth "
    let depth_pos = after_x.find(" depth ").ok_or_else(|| {
        ParseError::Message(format!("Expected 'depth' in well: {}", line))
    })?;
    let threshold: f32 = after_x[..depth_pos].trim().parse().map_err(|_| {
        ParseError::Message(format!("Invalid threshold: {}", &after_x[..depth_pos]))
    })?;
    
    let depth: f32 = after_x[depth_pos + 7..].trim().parse().map_err(|_| {
        ParseError::Message(format!("Invalid depth: {}", &after_x[depth_pos + 7..]))
    })?;
    
    Ok(WellDecl {
        name,
        particle,
        observable: ObservableExpr::PositionX(pos_particle),
        threshold,
        depth,
    })
}
