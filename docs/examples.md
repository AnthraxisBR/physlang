# Examples

This document describes the example programs included with PhysLang and demonstrates various use cases.

## Included Examples

### 1. Simple Two-Particle System

**File**: `examples/simple.phys`

**Description**: Basic example with two particles connected by gravity and a spring.

```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0

force gravity(a, b) G = 1.0
force spring(a, b) k = 5.0 rest = 3.0

simulate dt = 0.01 steps = 5000

detect a_pos = position(a)
detect dist_ab = distance(a, b)
```

**What it demonstrates**:
- Basic particle declarations
- Gravity and spring forces
- Detector evaluation

**Run**:
```bash
cargo run --bin physlang -- run examples/simple.phys
```

**Expected behavior**: Particles `a` and `b` are pulled together by gravity and connected by a spring. The spring tries to maintain a distance of 3.0, while gravity pulls them closer. The system reaches an equilibrium.

### 2. Force-Directed Graph Layout

**File**: `examples/graph_layout.phys`

**Description**: Computes a force-directed graph layout using physics primitives.

**Concept**: 
- Graph nodes → particles
- Graph edges → springs
- Center particle → gravity anchor

**Graph structure**:
- Nodes: A, B, C, D, E
- Edges: (A-B), (A-C), (B-D), (C-D), (C-E)

**What it demonstrates**:
- Multiple particles and forces
- Graph layout algorithm
- Using gravity to center the layout

**Run**:
```bash
cargo run --bin physlang -- run examples/graph_layout.phys
```

**Expected behavior**: The particles arrange themselves to minimize spring energy while staying centered due to gravity. The final positions approximate an optimal graph layout.

**Output interpretation**: The x-coordinates and distances show the final layout. In future versions with 2D visualization, this would display as a graph.

### 3. For-Loop Push

**File**: `examples/loop_for_push.phys`

**Description**: Demonstrates oscillator-based for-loops by pushing a particle multiple times.

```phys
particle a at (0.0, 0.0) mass 1.0

simulate dt = 0.01 steps = 5000

loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}

detect a_x = position(a)
```

**What it demonstrates**:
- For-loops via oscillators
- Loop body execution (push forces)
- Phase-based iteration

**Run**:
```bash
cargo run --bin physlang -- run examples/loop_for_push.phys
```

**Expected behavior**: The oscillator completes 10 cycles. Each cycle triggers a push to the right. The particle accumulates velocity from these repeated pushes, moving to the right.

**Key parameters**:
- `frequency = 1.0`: One cycle per time unit
- `damping = 0.0`: No amplitude reduction
- `magnitude = 0.5`: Impulse strength

### 4. While-Loop with Well

**File**: `examples/loop_while_well.phys`

**Description**: Demonstrates while-loops and potential wells working together.

```phys
particle a at (0.0, 0.0) mass 1.0

well target on a if position(a).x >= 5.0 depth 10.0

simulate dt = 0.01 steps = 8000

loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}

detect a_x = position(a)
```

**What it demonstrates**:
- While-loops with conditions
- Potential wells as attractors
- Conditional behavior

**Run**:
```bash
cargo run --bin physlang -- run examples/loop_while_well.phys
```

**Expected behavior**:
1. While-loop pushes particle right while `x < 5.0`
2. When particle reaches `x >= 5.0`, loop stops
3. Well captures particle at threshold, pulling it toward `x = 5.0`
4. Final position should be near `5.0`

**Key parameters**:
- `threshold = 5.0`: Well activation point
- `depth = 10.0`: Well strength
- `magnitude = 0.3`: Push strength (smaller than well to allow capture)

## Visual Evaluation Loop (VEL) Examples

The following examples are designed specifically for interactive visualization using the Visual Evaluation Loop (VEL). They demonstrate complex, real-world scenarios with rich visual dynamics.

### 5. Market Stress Propagation Demo

**File**: `examples/vel/market_stress_demo.phys`

**Description**: A financial market stress model that visualizes how stress propagates through a network of interconnected companies. This example demonstrates how localized shocks create cascading effects through financial exposure links.

**Model Components**:
- **6 Companies** (A-F): Represented as particles with different masses
- **Spring Network**: Financial exposure links between companies
- **Gravity**: Market stability forces keeping the network centered
- **Shock Loops**: Multiple periodic stress events on Company A
- **Default Well**: Risk threshold for Company B (x >= 4.0)

**What it demonstrates**:
- Complex multi-particle networks
- Stress propagation through spring connections
- Multiple simultaneous loop-based events
- Default threshold visualization via wells
- Real-time stress dynamics

**Run in VEL**:
```bash
cargo run --bin physlang -- visual examples/vel/market_stress_demo.phys
```

**Expected visual behavior**:
1. Companies start in a circular arrangement
2. Company A receives periodic shocks from multiple loops
3. Stress ripples through the spring network to connected companies
4. Companies oscillate and respond dynamically
5. Company B may drift toward default threshold (x >= 4.0)
6. If threshold crossed, well creates dramatic pull effect

**Key parameters**:
- `dt = 0.02`: Smooth animation timestep
- `steps = 8000`: ~160 time units of simulation
- Spring `k = 1.0-1.8`: Exposure intensity
- Gravity `G = 0.15`: Moderate stability
- Well `depth = 8.0`: Default risk strength

**Interpretation**:
- Spring stiffness = financial exposure intensity
- Particle mass = company size/stability
- Well depth = default risk (inverse of stability)
- Loop pushes = market stress events (liquidity withdrawal, asset markdowns)

### 6. Systemic Risk Visualizer

**File**: `examples/vel/systemic_risk_visualizer/systemic_risk.phys`

**Description**: A comprehensive systemic risk propagation simulation modeling how financial stress spreads through an interconnected banking network. This is a production-ready example demonstrating real-world financial system dynamics.

**Model Components**:
- **7 Banks** (BANK_A through BANK_G): Financial institutions with asset-based masses
- **9 Interbank Exposures**: Credit links represented as springs
- **7 Default Wells**: Capital ratio boundaries for each bank
- **Shock Loop**: Continuous stress events originating at BANK_A
- **Systemic Gravity**: Market-wide pressure stabilizing the network

**Financial Interpretation**:
- **Particles (Banks)**: Mass = institution size (normalized from trillions)
- **Springs (Exposures)**: Stiffness = exposure intensity (normalized from billions)
- **Wells (Default Thresholds)**: Triggered at x >= 10.0, depth = default risk
- **Shock Loop**: Represents liquidity withdrawal, asset markdowns, bank runs
- **Gravity**: Systemic pressure, liquidity contraction, regulatory forces

**Bank Network**:
- BANK_A: 3.1T assets (mass 1.86) - Largest, shock origin
- BANK_B: 2.4T assets (mass 1.44)
- BANK_C: 1.9T assets (mass 1.14)
- BANK_D: 1.6T assets (mass 0.96)
- BANK_E: 1.1T assets (mass 0.66)
- BANK_F: 0.8T assets (mass 0.48) - Vulnerable
- BANK_G: 0.6T assets (mass 0.36) - Most vulnerable

**Exposure Network**:
- A→B: 120B exposure (k=1.2)
- A→C: 80B exposure (k=0.8)
- B→C: 90B exposure (k=0.9)
- B→D: 110B exposure (k=1.1)
- C→E: 70B exposure (k=0.7)
- D→E: 60B exposure (k=0.6)
- D→F: 50B exposure (k=0.5)
- E→G: 40B exposure (k=0.4)
- F→G: 30B exposure (k=0.3)

**What it demonstrates**:
- Real-world financial system modeling
- Systemic risk propagation
- Cascade effects through exposure network
- Default risk visualization
- Network resilience analysis
- Quantitative risk metrics via detectors

**Run in VEL**:
```bash
cargo run --bin physlang -- visual examples/vel/systemic_risk_visualizer/systemic_risk.phys
```

**Expected visual behavior**:
1. **Initial State**: Banks arranged in a circle, springs connecting exposure links
2. **Shock Events**: BANK_A jitters and moves due to periodic loop pushes (every ~5 seconds)
3. **Stress Propagation**: Forces travel through spring network to connected banks
   - Connected banks (B, C) respond to BANK_A's movements
   - Network vibrates and oscillates as stress propagates
   - Springs stretch and contract, showing exposure transmission
4. **Default Risk**: Smaller banks (F, G) may drift toward x >= 10 threshold
   - When threshold crossed, wells activate with strong restoring forces
   - Visual indicator of financial distress
5. **System Dynamics**: Overall network behavior
   - **Stable**: Network oscillates but returns to equilibrium
   - **Unstable**: Network destabilizes, banks drift far from center
   - **Cascading**: Multiple banks approach default thresholds

**Key parameters**:
- `dt = 0.02`: Smooth animation timestep
- `steps = 12000`: 240 time units of simulation
- Spring `k = 0.3-1.2`: Exposure intensity (normalized)
- Gravity `G = 0.08`: Gentle systemic pressure
- Well `depth = 1.04-3.28`: Default risk (inverse of bank stability)
- Loop: 120 cycles at frequency 0.2 Hz with damping 0.03

**Detector Interpretation**:
After simulation, detectors output risk metrics:

- `core_link_stress`: Distance between BANK_A and BANK_B
  - Large values indicate core network disruption
  - Normal range: 3-5 units
  
- `secondary_stress`: Distance between BANK_D and BANK_E
  - Measures stress in secondary network connections
  - Large values indicate cascading effects
  
- `vulnerable_bank_x`: Final X-position of BANK_G
  - Values >= 10 indicate default threshold crossed
  - Higher values = deeper into default zone
  
- `energy_component_*`: Components for system energy calculation
  - Sum externally to get total system stress
  - Normal: ~15-25, Stressed: 25-40, Critical: >40

**Use Cases**:
- Financial system stress testing
- Systemic risk analysis
- Network resilience evaluation
- Default risk assessment
- Regulatory capital analysis
- Educational demonstration of financial contagion

**Modification Ideas**:
- Adjust shock frequency/magnitude to test system resilience
- Modify exposure network structure (add/remove springs)
- Change bank sizes (masses) to model different scenarios
- Adjust well depths to model different regulatory regimes
- Vary gravity strength to simulate different market conditions

## Expressions and Functions Examples (v0.6+)

### Using Expressions

**File**: Create `examples/expressions.phys`

```phys
let pi = 3.14159;
let mass = 1.0;
let frequency = 2.0;
let omega = 2.0 * pi * frequency;
let k = mass * omega * omega;

particle a at (0.0, 0.0) mass mass
particle b at (sqrt(2.0) * 5.0, 0.0) mass mass

force spring(a, b) k = k rest = 5.0

simulate dt = 0.01 steps = 10000

detect dist = distance(a, b)
```

**What it demonstrates**:
- Using `let` bindings for reusable values
- Arithmetic expressions (`*`, `/`)
- Built-in functions (`sqrt`)
- Expressions in all numeric positions

### Using Functions

**File**: Create `examples/functions.phys`

```phys
fn make_particle(name, x, y, m) {
    particle name at (x, y) mass m
}

fn connect_spring(a, b, stiffness) {
    let rest = distance(a, b);
    force spring(a, b) k = stiffness rest = rest
}

make_particle(a, 0.0, 0.0, 1.0);
make_particle(b, 5.0, 0.0, 1.0);
make_particle(c, 2.5, 4.33, 1.0);

connect_spring(a, b, 2.0);
connect_spring(b, c, 2.0);
connect_spring(c, a, 2.0);

simulate dt = 0.01 steps = 10000

detect dist_ab = distance(a, b)
```

**What it demonstrates**:
- Function definitions for reusable patterns
- Functions generating particles and forces
- Parameterized world-building

### Advanced: Functions with Return Values

**File**: Create `examples/function_returns.phys`

```phys
fn compute_spring_k(mass, frequency) {
    let pi = 3.14159;
    let omega = 2.0 * pi * frequency;
    return mass * omega * omega;
}

let k1 = compute_spring_k(1.0, 2.0);
let k2 = compute_spring_k(2.0, 1.5);

particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0

force spring(a, b) k = k1 rest = 5.0

simulate dt = 0.01 steps = 10000

detect dist = distance(a, b)
```

**What it demonstrates**:
- Functions returning scalar values
- Using function return values in expressions
- Computed parameters

## Use Case Patterns

### Pattern 1: Mass-Spring Oscillator

Create a simple harmonic oscillator:

```phys
particle mass at (0.0, 0.0) mass 1.0
particle anchor at (0.0, 0.0) mass 1000.0  # Heavy, fixed

force spring(mass, anchor) k = 10.0 rest = 0.0

simulate dt = 0.001 steps = 10000

detect mass_x = position(mass)
```

**Key**: Heavy anchor acts as fixed point. Spring with `rest = 0.0` creates restoring force.

### Pattern 2: Soft Body (Triangle)

Create a triangle of connected particles:

```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
particle c at (2.5, 4.33) mass 1.0

# Connect all edges
force spring(a, b) k = 2.0 rest = 5.0
force spring(b, c) k = 2.0 rest = 5.0
force spring(c, a) k = 2.0 rest = 5.0

simulate dt = 0.01 steps = 10000

detect dist_ab = distance(a, b)
detect dist_bc = distance(b, c)
detect dist_ca = distance(c, a)
```

**Key**: All particles connected by springs maintain triangle shape.

### Pattern 3: Iterative Optimization

Use loops to iteratively push toward a goal:

```phys
particle a at (0.0, 0.0) mass 1.0
particle goal at (10.0, 0.0) mass 1.0

# Attract toward goal
force gravity(a, goal) G = 0.5

simulate dt = 0.01 steps = 10000

# Push toward goal each cycle
loop for 20 cycles with frequency 0.5 damping 0.0 on a {
    force push(a) magnitude 0.2 direction (1.0, 0.0)
}

detect a_x = position(a)
detect dist_to_goal = distance(a, goal)
```

**Key**: Combination of continuous force (gravity) and discrete pushes (loop).

### Pattern 4: Conditional Capture

Use wells to create conditional regions:

```phys
particle a at (0.0, 0.0) mass 1.0

# Two wells at different thresholds
well left on a if position(a).x >= 2.0 depth 5.0
well right on a if position(a).x >= 8.0 depth 5.0

simulate dt = 0.01 steps = 10000

# Push right
loop for 15 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.4 direction (1.0, 0.0)
}

detect a_x = position(a)
```

**Key**: Particle may be captured by first well (if it reaches `x = 2.0`) or continue to second well.

## Experimentation Ideas

### 1. Parameter Tuning

Try different values:
- **Time step**: `dt = 0.001` vs `0.01` vs `0.1`
- **Spring constant**: `k = 0.5` vs `5.0` vs `50.0`
- **Gravity**: `G = 0.1` vs `1.0` vs `10.0`
- **Well depth**: `depth = 1.0` vs `10.0` vs `100.0`

Observe how stability and behavior change.

### 2. Multi-Particle Systems

Create larger systems:
- 10 particles in a chain (connected by springs)
- Grid of particles (springs between neighbors)
- Star topology (one center, many satellites)

### 3. Complex Loops

Combine loops:
- Nested behavior (one loop inside another's body - future feature)
- Multiple loops on different particles
- Loops with varying frequencies

### 4. Well Patterns

Experiment with wells:
- Multiple wells at different thresholds
- Wells on different axes (x and y)
- Varying well depths

## Debugging Tips

### Check Detector Values

Always include detectors to verify behavior:

```phys
detect particle_x = position(particle)
detect dist = distance(a, b)
```

### Monitor Stability

If results are unexpected:
1. Check if particles have reasonable final positions
2. Verify distances are positive and finite
3. Try smaller time step if oscillations are wild

### Validate Forces

Make sure forces are reasonable:
- Gravity: `G` typically 0.1-1.0
- Spring: `k` typically 1.0-10.0
- Well: `depth` typically 5.0-20.0

### Test Incrementally

Start simple and add complexity:
1. Single particle, no forces
2. Two particles, one force
3. Add more forces
4. Add loops/wells

## Advanced Examples (Future)

These patterns may be possible in future versions:

### Equilibrium Detection

```phys
# Future: automatic termination
simulate dt = 0.01 until equilibrium energy < 0.001
```

### Custom Forces

```phys
# Future: user-defined forces
force custom(a, b) {
    # Custom force calculation
}
```

### Visualization

```phys
# Future: real-time visualization
visualize particles forces
```

## Contributing Examples

If you create interesting examples, consider:
1. Adding them to `examples/` directory
2. Documenting them here
3. Explaining the physical interpretation
4. Providing parameter ranges for stability

Happy experimenting!

