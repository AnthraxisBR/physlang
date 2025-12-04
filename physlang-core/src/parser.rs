use crate::ast::{
    DetectorDecl, DetectorKind, ForceDecl, ForceKind, ParticleDecl, Program, SimulateDecl,
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

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("particle ") {
            particles.push(parse_particle(line)?);
        } else if line.starts_with("force ") {
            forces.push(parse_force(line)?);
        } else if line.starts_with("simulate ") {
            simulate = Some(parse_simulate(line)?);
        } else if line.starts_with("detect ") {
            detectors.push(parse_detector(line)?);
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
