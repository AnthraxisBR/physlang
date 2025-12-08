# Semantics

This document defines the static and dynamic semantics of PhysLang.

## Static Semantics

### Type System

PhysLang uses a statically-checked type system with four core types, an optional dimensional analysis layer, and effect annotations for functions.

#### Core Types

| Type | Description | Representation |
|------|-------------|----------------|
| `Scalar` | Floating-point numbers | `f32` |
| `Vec2` | 2D vectors (position, velocity, direction) | `(f32, f32)` |
| `Bool` | Boolean values for conditions | `true` / `false` |
| `ParticleRef` | Reference to a declared particle | Stable identifier |

**Scalar** represents dimensionless or dimensioned numeric quantities. All arithmetic operations on numbers produce Scalars. Scalars are used for masses, constants, distances, coordinates, and time values.

**Vec2** represents two-dimensional vectors. Positions, velocities, and directions are Vec2 values. Component access (`.x`, `.y`) yields a Scalar.

**Bool** represents truth values. Comparison expressions evaluate to Bool. Bools are used in conditions for while-loops, wells, and language-level control flow (`if`, `match`).

**ParticleRef** is a reference type that names a specific particle declaration. ParticleRef values cannot be created dynamically—they are bound to particle declarations at parse time. See [Particle References and Lifetimes](#particle-references-and-lifetimes) for details.

#### Typing Judgments

We use standard notation: $\Gamma \vdash e : \tau$ means "in context $\Gamma$, expression $e$ has type $\tau$."

**Literals**:
$$\Gamma \vdash n : \text{Scalar} \quad \text{(numeric literals)}$$

**Variables**:
$$\frac{x : \tau \in \Gamma}{\Gamma \vdash x : \tau}$$

**Arithmetic operations** (for $\oplus \in \{+, -, *, /\}$):
$$\frac{\Gamma \vdash e_1 : \text{Scalar} \quad \Gamma \vdash e_2 : \text{Scalar}}{\Gamma \vdash e_1 \oplus e_2 : \text{Scalar}}$$

**Unary negation**:
$$\frac{\Gamma \vdash e : \text{Scalar}}{\Gamma \vdash -e : \text{Scalar}}$$

**Comparison operations** (for $\bowtie \in \{<, >, \leq, \geq, ==, \neq\}$):
$$\frac{\Gamma \vdash e_1 : \text{Scalar} \quad \Gamma \vdash e_2 : \text{Scalar}}{\Gamma \vdash e_1 \bowtie e_2 : \text{Bool}}$$

**Vector component access**:
$$\frac{\Gamma \vdash e : \text{Vec2}}{\Gamma \vdash e.x : \text{Scalar}} \quad \frac{\Gamma \vdash e : \text{Vec2}}{\Gamma \vdash e.y : \text{Scalar}}$$

**Built-in functions**:
$$\frac{\Gamma \vdash e : \text{Scalar}}{\Gamma \vdash \text{sin}(e) : \text{Scalar}} \quad \frac{\Gamma \vdash e : \text{Scalar}}{\Gamma \vdash \text{cos}(e) : \text{Scalar}} \quad \frac{\Gamma \vdash e : \text{Scalar}}{\Gamma \vdash \text{sqrt}(e) : \text{Scalar}}$$

$$\frac{\Gamma \vdash e : \text{Scalar} \quad \Gamma \vdash e_{min} : \text{Scalar} \quad \Gamma \vdash e_{max} : \text{Scalar}}{\Gamma \vdash \text{clamp}(e, e_{min}, e_{max}) : \text{Scalar}}$$

**Observables on particles**:
$$\frac{\Gamma \vdash p : \text{ParticleRef}}{\Gamma \vdash \text{position}(p) : \text{Vec2}}$$

$$\frac{\Gamma \vdash p_1 : \text{ParticleRef} \quad \Gamma \vdash p_2 : \text{ParticleRef}}{\Gamma \vdash \text{distance}(p_1, p_2) : \text{Scalar}}$$

#### Observable Typing

Observables have the following types:

- `position(a) : Vec2` (internally, v0.2 returns x-coordinate as Scalar)
- `position(a).x : Scalar`
- `position(a).y : Scalar`
- `distance(a, b) : Scalar`
- `ObservableRel : Bool` (e.g., `position(a).x < 5.0`)

Numeric literals are `Scalar`.

---

### Dimensional Analysis (Units System)

PhysLang provides an **optional static analysis layer** that tracks physical dimensions on Scalar values. This layer runs after basic type checking and warns (or errors) when expressions combine quantities with incompatible physical dimensions.

#### Base Dimensions

The dimensional system tracks the following base dimensions:

| Dimension | Symbol | Examples |
|-----------|--------|----------|
| Length | L | positions, distances, rest lengths |
| Time | T | time step `dt`, oscillator periods |
| Mass | M | particle masses |

Derived dimensions are expressed as products of base dimensions:

| Derived Dimension | Expression | Examples |
|-------------------|------------|----------|
| Velocity | L·T⁻¹ | particle velocities |
| Acceleration | L·T⁻² | force per mass |
| Force | M·L·T⁻² | spring force, gravity |
| Spring constant | M·T⁻² | `k` in springs |
| Gravitational constant | L³·M⁻¹·T⁻² | `G` in gravity forces |
| Frequency | T⁻¹ | oscillator frequency |
| Well depth | M·T⁻² | well strength parameter |

#### Dimension Annotations

Dimensions are inferred from context or can be explicitly annotated in future syntax extensions. Currently, the checker infers dimensions from:

1. **Particle declarations**: `at (x, y)` implies x, y have dimension L
2. **Particle properties**: `mass m` implies m has dimension M
3. **Simulation config**: `dt = ...` implies the value has dimension T
4. **Force parameters**: `G`, `k`, `rest`, etc. have known dimensional signatures
5. **Observables**: `position(p)` returns L (per component), `distance(a, b)` returns L

#### Dimension Propagation Rules

**Addition/Subtraction**: operands must have identical dimensions:
$$\text{dim}(e_1 + e_2) = \text{dim}(e_1) = \text{dim}(e_2)$$

**Multiplication**:
$$\text{dim}(e_1 * e_2) = \text{dim}(e_1) \cdot \text{dim}(e_2)$$

**Division**:
$$\text{dim}(e_1 / e_2) = \text{dim}(e_1) \cdot \text{dim}(e_2)^{-1}$$

**Dimensionless operations**: `sin`, `cos` require dimensionless arguments and return dimensionless results. `sqrt` returns a value with dimension $\text{dim}(e)^{1/2}$.

**Comparisons**: operands must have identical dimensions (result is Bool, which is dimensionless).

#### Dimension Checker Warnings

The dimension checker **warns** about the following violations:

**Incompatible addition**:
```phys
let bad = position(a).x + 3.0;  # ERROR: L + dimensionless
```
*Diagnostic*: "Cannot add Length to dimensionless quantity"

**Wrong dimension for dt**:
```phys
simulate dt = 5.0 steps = 1000  # OK if 5.0 is Time
simulate dt = position(a).x steps = 1000  # ERROR: dt must be Time, got Length
```
*Diagnostic*: "Time step 'dt' requires dimension T, got L"

**Incompatible force parameters**:
```phys
force spring(a, b) k = 2.0 rest = 3.0  # OK
force spring(a, b) k = 2.0 rest = dt   # WARNING: rest length should be L, got T
```
*Diagnostic*: "Spring rest length requires dimension L, got T"

**Meaningless comparisons**:
```phys
loop while position(a).x < dt with frequency 1.0 damping 0.0 on a {
    # ERROR: comparing L < T
}
```
*Diagnostic*: "Cannot compare Length with Time"

#### Dimensionless Scalars

Literal numbers without context are treated as **dimensionless**. Dimensionless values can be:
- Used directly in trigonometric functions
- Multiplied with any dimensioned quantity (scaling)
- Compared only with other dimensionless values

Example of correct usage:
```phys
let pi = 3.14159;           # dimensionless
let angle = pi / 2.0;       # dimensionless
let scale = 2.0;            # dimensionless
let offset = scale * 5.0;   # dimensionless * dimensionless = dimensionless

particle a at (offset, 0.0) mass 1.0  # offset used as Length (implicit conversion OK)
```

#### Enabling Dimensional Analysis

Dimensional analysis is an **opt-in** static check. When enabled:
- The checker infers dimensions for all Scalar expressions
- Violations produce warnings (not errors by default)
- A strict mode can be enabled to treat violations as errors

*Implementation note*: This is a design-time specification. The reference implementation may initially implement a subset of these checks.

---

### Effect System

PhysLang distinguishes between **pure** functions and **world-building** functions based on their effects on the simulation world.

#### Effect Categories

| Effect | Description | Marker |
|--------|-------------|--------|
| `pure` | Computes values only; no world modification | (default, no marker) |
| `world` | May create particles, forces, wells, loops, detectors | `world` keyword |

#### Pure Functions

Pure functions:
- Only compute Scalar, Vec2, or Bool values
- Cannot contain particle, force, well, loop, or detector declarations
- Can call other pure functions
- Can use `return expr;` to return a value
- Are safe to call in any expression context

**Syntax**:
```phys
fn compute_distance(x1, y1, x2, y2) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return sqrt(dx * dx + dy * dy);
}
```

**Typing rule for pure functions**:
$$\frac{\Gamma, x_1:\text{Scalar}, \ldots, x_n:\text{Scalar} \vdash \text{body} : \tau \quad \text{body contains no world-building statements}}{\Gamma \vdash \texttt{fn } f(x_1, \ldots, x_n) \{ \text{body} \} : (\text{Scalar}^n \to \tau, \text{pure})}$$

#### World-Building Functions

World-building functions:
- May contain particle, force, well, loop, or detector declarations
- Execute before simulation to construct the physical world
- Do not return a value (conceptually return `unit`)
- Can only be called at the top level or from other world-building functions
- Cannot be called from within pure expression evaluation

**Syntax** (with `world` marker):
```phys
fn make_chain(n, spacing) world {
    for i in 0..n {
        let x = i * spacing;
        particle p at (x, 0.0) mass 1.0
    }
}
```

**Typing rule for world functions**:
$$\frac{\Gamma, x_1:\text{Scalar}, \ldots, x_n:\text{Scalar} \vdash \text{body ok}}{\Gamma \vdash \texttt{fn } f(x_1, \ldots, x_n) \texttt{ world } \{ \text{body} \} : (\text{Scalar}^n \to \text{unit}, \text{world})}$$

#### Effect Checking Rules

The compiler enforces the following effect constraints:

1. **Pure functions cannot contain world-building statements**:
   ```phys
   fn bad_pure(x) {
       particle p at (x, 0.0) mass 1.0  # ERROR: world-building in pure function
       return x;
   }
   ```
   *Error*: "Cannot declare particle inside pure function; add 'world' marker"

2. **Pure functions cannot call world functions**:
   ```phys
   fn setup() world {
       particle a at (0.0, 0.0) mass 1.0
   }
   
   fn compute(x) {
       setup();  # ERROR: calling world function from pure context
       return x * 2.0;
   }
   ```
   *Error*: "Cannot call world function 'setup' from pure function 'compute'"

3. **World functions cannot be called in expression position**:
   ```phys
   fn setup() world {
       particle a at (0.0, 0.0) mass 1.0
   }
   
   let x = setup();  # ERROR: world function has no return value
   ```
   *Error*: "World function 'setup' does not return a value"

4. **World functions can call pure functions**:
   ```phys
   fn circle_x(i, n, r) {
       return r * cos(2.0 * 3.14159 * i / n);
   }
   
   fn make_ring(n, r) world {
       for i in 0..n {
           let x = circle_x(i, n, r);  # OK: pure call from world function
           particle p at (x, 0.0) mass 1.0
       }
   }
   ```

#### Backward Compatibility

For backward compatibility with pre-v0.9 code, functions without explicit effect markers are classified as follows:
- If the function body contains any world-building statement → implicitly `world`
- Otherwise → implicitly `pure`

The compiler may emit a warning for implicit world functions, encouraging explicit `world` annotation.

---

### Particle References and Lifetimes

#### ParticleRef Semantics

A `ParticleRef` is a **stable identifier** that names a particle declaration. Unlike pointers in imperative languages:

- ParticleRef values are **not** memory addresses
- ParticleRef values **cannot** become invalid or dangling
- ParticleRef values are **immutable** identifiers

#### Obtaining ParticleRef Values

ParticleRef values are obtained from:

1. **Particle declarations**: The identifier in `particle name at ...` creates a ParticleRef binding
   ```phys
   particle a at (0.0, 0.0) mass 1.0  # 'a' is now a ParticleRef
   ```

2. **Function parameters** (when used in world-building contexts):
   ```phys
   fn connect(p1, p2, k) world {
       force spring(p1, p2) k = k rest = 1.0  # p1, p2 are ParticleRef
   }
   ```

3. **String literals** (v0.8+) can be used as particle names in certain contexts, resolved to ParticleRef at elaboration time.

#### ParticleRef Usage

ParticleRef values can be used in:

- **Force declarations**: `force gravity(a, b) ...`, `force spring(a, b) ...`
- **Well declarations**: `well w on a if ...`
- **Loop declarations**: `loop ... on a { ... }`
- **Observable expressions**: `position(a)`, `distance(a, b)`
- **Detector declarations**: `detect d = position(a)`
- **Function arguments**: Both pure and world functions can accept ParticleRef parameters

#### Static Particle Model

In the current version of PhysLang:

1. **No dynamic creation**: Particles cannot be created during simulation. All particles are declared before simulation begins.

2. **No destruction**: Particles exist for the entire duration of the simulation. There is no mechanism to remove particles.

3. **Stable identity**: Each particle has a unique, stable identifier determined at parse time.

4. **No aliasing concerns**: Since ParticleRef values are immutable identifiers (not mutable pointers), there are no aliasing issues.

#### Invariants

The following invariants hold for all well-formed PhysLang programs:

- **Referential integrity**: Every ParticleRef used in forces, wells, loops, detectors, or observables refers to a particle that is declared in the program.

- **Lifetime guarantee**: Every ParticleRef remains valid for the entire execution of the program. There is no possibility of a "dangling reference."

- **Unique naming**: No two particles share the same identifier.

Example demonstrating referential integrity checking:
```phys
particle a at (0.0, 0.0) mass 1.0

force gravity(a, b) G = 1.0  # ERROR: 'b' is not declared
```
*Error*: "Particle 'b' not found"

---

### Environments

#### Particle Environment

The particle environment $\Gamma_p$ maps identifiers to particle declarations:

$$\Gamma_p : \text{Ident} \rightarrow \text{ParticleDecl}$$

This environment is built during parsing by collecting all `particle` declarations.

#### Variable Environment (v0.6+)

The variable environment $\Gamma_v$ maps identifiers to evaluated scalar values:

$$\Gamma_v : \text{Ident} \rightarrow \text{Scalar}$$

This environment is built by evaluating all top-level `let` bindings before simulation.

#### Function Environment (v0.7+)

The function environment $\Gamma_f$ maps identifiers to function declarations with their effect annotations:

$$\Gamma_f : \text{Ident} \rightarrow (\text{FunctionDecl}, \text{Effect})$$

Where $\text{Effect} \in \{\text{pure}, \text{world}\}$.

This environment is built during parsing by collecting all `fn` declarations.

---

### Well-Formedness Rules

A PhysLang program is **well-formed** if it satisfies the following rules:

#### 1. Unique Particle Names

No duplicate particle identifiers:

$$\forall i, j: i \neq j \Rightarrow \text{name}(p_i) \neq \text{name}(p_j)$$

**Error**: "Duplicate particle name: `<name>`"

#### 2. Forces Reference Existing Particles

In `force gravity(a, b) ...` or `force spring(a, b) ...`:

$$a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

#### 3. Loops Reference Existing Target

In `loop ... on <particle>`:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for loop"

#### 4. Loop Body Push Target Exists

In `force push(<particle>) ...` inside a loop body:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

#### 5. Wells Reference Existing Particle

In `well <name> on <particle> ...`:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for well"

#### 6. Detectors Reference Existing Particles

In `detect ... = position(<particle>)` or `detect ... = distance(<a>, <b>)`:

$$\text{particle} \in \text{dom}(\Gamma_p) \land a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for detector"

#### 7. Observable References

In conditions like `position(<particle>).x` or `distance(<a>, <b>)`:

$$\text{particle} \in \text{dom}(\Gamma_p) \land a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

#### 8. Effect Consistency

Functions marked as pure contain no world-building statements. Functions calling world-building functions are themselves world-building.

**Error**: "Cannot declare particle inside pure function; add 'world' marker"

#### 9. Return Type Consistency

Pure functions with return statements must return Scalar values. World functions may not have return statements.

**Error**: "World function cannot return a value"

---

### Guaranteed Properties (Invariants)

A well-typed, well-formed PhysLang program satisfies the following invariants:

1. **No free particles**: Every `ParticleRef` in expressions, forces, wells, loops, and detectors corresponds to a declared particle.

2. **No type mismatches**: The static type checker rejects programs where expressions have incompatible types (e.g., adding Vec2 and Scalar directly, or passing the wrong type to a built-in function).

3. **No dangling references**: Every `ParticleRef` remains valid throughout program execution. Particles are never deallocated or destroyed.

4. **Dimensional consistency** (when enabled): The dimensional analysis layer warns or rejects expressions that combine quantities with incompatible physical dimensions (e.g., adding length to time).

5. **Effect soundness**: Pure functions are guaranteed not to modify the world configuration. Only world-building functions (explicitly marked or inferred) can create particles, forces, wells, loops, or detectors.

6. **Deterministic execution**: Given identical source code and inputs, the program produces identical outputs. This follows from the fixed integration scheme and step count.

7. **Termination**: Every PhysLang program terminates after exactly `steps` simulation steps.

---

## Dynamic Semantics

### World State

The world state at time $t$ is:

$$W(t) = \{p_i(t)\}_{i=1..N}$$

Where each particle $p_i$ has:
- `pos: Vec2` - Position vector
- `vel: Vec2` - Velocity vector
- `mass: f32` - Mass (positive)

**Implementation**:
```rust
struct World {
    particles: Vec<Particle>,
    forces: Vec<Force>,
}
```

### Force Evaluation

Forces are evaluated at each time step based on the current world state.

#### Gravity Force

For `force gravity(a, b) G = g`:

1. Compute relative position: $r = \text{pos}_b - \text{pos}_a$
2. Compute squared distance: $\text{dist}^2 = \max(|r|^2, \epsilon)$ (avoid division by zero)
3. Compute force magnitude: $f_{\text{mag}} = G \cdot m_a \cdot m_b / \text{dist}^2$
4. Compute force vector: $f_{\text{vec}} = \text{normalize}(r) \cdot f_{\text{mag}}$
5. Apply forces:
   - Add $+f_{\text{vec}}$ to particle $a$
   - Add $-f_{\text{vec}}$ to particle $b$ (Newton's third law)

#### Spring Force

For `force spring(a, b) k = k rest = r`:

1. Compute relative position: $r = \text{pos}_b - \text{pos}_a$
2. Compute distance: $\text{dist} = \max(|r|, \epsilon)$
3. Compute extension: $\text{extension} = \text{dist} - r$ (displacement from rest length)
4. Compute force vector: $f_{\text{vec}} = \text{normalize}(r) \cdot (k \cdot \text{extension})$
5. Apply forces:
   - Add $+f_{\text{vec}}$ to particle $a$
   - Add $-f_{\text{vec}}$ to particle $b$

#### Push Force (in Loop Bodies)

For `force push(a) magnitude m direction (dx, dy)`:

1. Normalize direction: $\hat{d} = \text{normalize}((dx, dy))$
2. Apply impulse: $\text{vel}_a \leftarrow \text{vel}_a + \hat{d} \cdot m$

This is an **instantaneous impulse** applied when the loop iteration fires, not a continuous force.

### Potential Wells

For `well name on a if position(a).x >= T depth D`:

At each time step:

1. Evaluate observable: $x = \text{pos}_a.x$
2. If $x \geq T$:
   - Compute displacement: $\Delta x = x - T$
   - Compute restoring force: $F_x = -D \cdot \Delta x$
   - Apply acceleration: $a_x = F_x / m_a$
   - Update velocity: $\text{vel}_a.x \leftarrow \text{vel}_a.x + a_x \cdot \Delta t$

This creates a spring-like force that pulls the particle toward the threshold when it enters the well region.

**Note**: v0.2 only supports 1D wells along the x-axis. Future versions will support y-axis and distance-based wells.

### Loops (Oscillator-Based)

#### Loop State

Each loop maintains:
- `phase: f32` - Current oscillator phase $\phi \in [0, 2\pi)$
- `frequency: f32` - Oscillation frequency $f$
- `damping: f32` - Damping coefficient $\gamma$
- `active: bool` - Whether the loop is still active

For for-loops:
- `cycles_remaining: u32` - Number of cycles left

For while-loops:
- `condition: ConditionRuntime` - Condition to evaluate

#### Phase Evolution

At each global time step $\Delta t$:

1. **Advance phase**:
   $$\phi(t + \Delta t) = (\phi(t) + 2\pi f \cdot \Delta t) \cdot (1 - \gamma \cdot \Delta t)$$

   The damping term $(1 - \gamma \cdot \Delta t)$ gradually reduces oscillation amplitude.

2. **Check for phase wrap**:
   If $\phi \geq 2\pi$:
   - $\phi \leftarrow \phi - 2\pi$ (reset phase)
   - **Fire loop iteration event**

#### Loop Iteration Event

When a loop iteration fires:

**For for-loops**:
1. Apply loop body (e.g., push impulses)
2. Decrement `cycles_remaining`
3. If `cycles_remaining == 0`: Set `active = false`

**For while-loops**:
1. Evaluate condition based on current world state
2. If condition is `false`: Set `active = false` (do nothing)
3. If condition is `true`: Apply loop body

#### Condition Evaluation

Conditions are evaluated using the current world state:

- `position(a).x < T`: Check if `particles[a].pos.x < T`
- `position(a).x > T`: Check if `particles[a].pos.x > T`
- `position(a).y < T`: Check if `particles[a].pos.y < T`
- `position(a).y > T`: Check if `particles[a].pos.y > T`
- `distance(a, b) < T`: Check if `|particles[a].pos - particles[b].pos| < T`
- `distance(a, b) > T`: Check if `|particles[a].pos - particles[b].pos| > T`

### Simulation Step

A single simulation step with time step $\Delta t$:

1. **Update loops**: Advance oscillators, fire iterations if phase wraps
2. **Apply wells**: Compute well forces and apply to particles
3. **Compute accelerations**: For each particle, sum all forces:
   $$a_i = \frac{1}{m_i} \sum_k F_k^i$$
4. **Update velocities** (semi-implicit Euler):
   $$v_i(t + \Delta t) = v_i(t) + a_i(t) \cdot \Delta t$$
5. **Update positions**:
   $$x_i(t + \Delta t) = x_i(t) + v_i(t + \Delta t) \cdot \Delta t$$
6. **Evaluate loop conditions**: Check while-loop conditions to deactivate finished loops

### Simulation and Termination

A `simulate dt = Δt steps = N` declaration defines:

- Initial time: $t = 0$
- For $i = 1..N$:
  - Perform one simulation step
  - $t \leftarrow t + \Delta t$

**Termination**: The program terminates after exactly $N$ steps.

**Future**: Equilibrium-based termination may be added, where the program stops when kinetic energy falls below a threshold.

### Detector Evaluation

At the end of simulation (after $N$ steps), detectors are evaluated on the final world state $W(T)$ where $T = N \cdot \Delta t$:

**Position detector**:
- `position(a)` → Returns `particles[a].pos.x` (v0.2)
- Future: May return full `Vec2`

**Distance detector**:
- `distance(a, b)` → Returns `|particles[a].pos - particles[b].pos|`

Detectors return a list of `(name, value)` pairs, where `value` is a `Scalar`.

## Expression Evaluation (v0.6+)

Expressions are evaluated at "compile-time" (before simulation) to concrete `f32` values.

### Evaluation Context

An evaluation context $E$ maps variable names to their values:

$$E : \text{Ident} \rightarrow \text{Scalar}$$

### Expression Semantics

- **Literals**: `1.0` → $1.0$, `42` → $42.0$
- **Variables**: `x` → $E(x)$ if $x \in \text{dom}(E)$, else error
- **Unary minus**: `-e` → $-v$ where $v = \text{eval}(e, E)$
- **Binary operations**: `e1 + e2` → $\text{eval}(e1, E) + \text{eval}(e2, E)$ (similar for `-`, `*`, `/`)
- **Function calls**:
  - `sin(x)` → $\sin(\text{eval}(x, E))$
  - `cos(x)` → $\cos(\text{eval}(x, E))$
  - `sqrt(x)` → $\sqrt{\text{eval}(x, E)}$ (error if negative)
  - `clamp(x, min, max)` → $\text{clamp}(\text{eval}(x, E), \text{eval}(min, E), \text{eval}(max, E))$

### Let Binding Evaluation

Let bindings are evaluated in order:

1. Evaluate expression: $v = \text{eval}(\text{expr}, E)$
2. Extend environment: $E' = E[x \mapsto v]$
3. Continue with $E'$ for subsequent bindings

**Error conditions**:
- Unknown variable in expression
- Division by zero
- Negative argument to `sqrt`
- Invalid `clamp` arguments (min > max)

## Function Execution (v0.7+)

Functions are executed before simulation to generate world-building statements.

### Function Call Semantics

When a function `fn f(x1, x2, ...) { body }` is called with arguments `(a1, a2, ...)`:

1. **Create local context**: $E_{\text{local}} = \{x1 \mapsto \text{eval}(a1, E), x2 \mapsto \text{eval}(a2, E), ...\}$
2. **Execute body**: For each statement in `body`:
   - If `let y = e`: Evaluate $e$ in $E_{\text{local}} \cup E_{\text{global}}$, extend $E_{\text{local}}$
   - If `f(...)`: Recursively call function
   - If world-building statement: Generate AST node
   - If `return e`: Evaluate and return (if scalar return)
3. **Merge generated statements** into main program AST

### Scope Resolution

Variable lookup order (highest to lowest priority):
1. Local `let` bindings in current function
2. Function parameters
3. Global `let` bindings

### Function Execution Model

Functions operate in one of two modes based on their effect annotation:

**Pure functions** (`fn f(...) { ... }`):
- Compute and return Scalar values
- Cannot generate world-building statements
- Safe to call in expression contexts

**World-building functions** (`fn f(...) world { ... }`):
- Execute to generate particles, forces, loops, wells, or detectors
- Modify the AST by generating declarations
- Must be called at statement level (not in expressions)
- Execute before simulation begins

## Operational Semantics Summary

The execution of a PhysLang program can be summarized as:

1. **Parse** → Build AST and environments ($\Gamma_p$, $\Gamma_f$)
2. **Validate** → Check well-formedness rules
3. **Effect check** → Verify pure/world annotations are consistent
4. **Evaluate Global Lets** → Build variable environment $\Gamma_v$ (v0.6+)
5. **Execute Functions** → Generate world-building statements from function calls (v0.7+)
6. **Re-validate** → Check well-formedness of generated world
7. **Dimension check** (optional) → Verify dimensional consistency
8. **Build World** → Create initial world state $W(0)$
9. **Build Loops/Wells** → Create loop and well instances
10. **Simulate** → For $i = 1..N$:
   - Update loops
   - Apply wells
   - Integrate physics
   - Evaluate conditions
11. **Detect** → Evaluate detectors on $W(T)$
12. **Output** → Return detector results

This provides a deterministic, physically-grounded execution model where computation emerges from the evolution of a dynamical system.
