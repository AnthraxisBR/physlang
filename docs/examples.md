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

