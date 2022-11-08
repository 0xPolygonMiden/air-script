# Constraint description sections

## Boundary constraints (`boundary_constraints`)

The `boundary_constraints` section consists of expressions describing the expected value of columns in the main or auxiliary traces at the specified boundary. Column boundaries can be selected using boundary accessors. Valid boundary accessors are `.first`, which selects the first cell of the column to which it is applied, and `.last`, which selects the last cell of the column column to which it is applied.

**Boundary constraints are required.** The `boundary_constraints` section must be defined and contain at least one boundary constraint.

Boundary constraints that are defined against auxiliary columns or that use random values from the built-in `$rand` array will be identified as auxiliary constraints.

A boundary constraint definition must:

1. start with a block indentation and the `enf` keyword to indicate that the constraint must be _enforced_.
2. continue by specifying a column identifier with a boundary accessor, e.g. `a.first` or `a.last`.
3. continue with `=`
4. continue with a right-hand-size "value" expression that evaluates to the required value of the specified column at the specified boundary. The expression may include numbers, trace columns, public inputs, random values, and any of the available [operations](./syntax.md#operations).
5. end with a newline.

### Simple example of boundary constraints

The following is a simple example of a valid `boundary_constraints` block:

```
def BoundaryConstraintsExample

trace_columns:
    main: [a]
    aux: [p]

public_inputs:
    <omitted for brevity>

boundary_constraints:
    # these are main constraints.
    enf a.first = 0
    enf a.last = 10

    # these are auxiliary constraints, since they are defined against auxiliary trace columns.
    enf p.first = 1
    enf p.last = 1

transition_constraints:
    <omitted for brevity>
```

### Public inputs and random values

Boundary constraints can access public input values and random values provided by the verifier in their value expressions.

To use public inputs, the public input must be declared in the `public_inputs` source section. They can be accessed using array indexing syntax, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

Random values can be accessed by using array indexing syntax on the `$rand` built-in, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

### Example of boundary constraints with public inputs and random values

The following is an example of a valid `boundary_constraints` block that uses public inputs and random values:

```
def BoundaryConstraintsExample

trace_columns:
    main: [a, b]
    aux: [p0, p1]

public_inputs:
    stack_inputs: [16]
    stack_outputs: [16]

boundary_constraints:
    # these are main constraints that use public input values.
    enf a.first = stack_inputs[0]
    enf a.last = stack_outputs[0]

    # these are auxiliary constraints that use public input values.
    enf p0.first = stack_inputs[1]
    enf p0.last = stack_outputs[1]

    # these are auxiliary constraints that use random values from the verifier.
    enf b.first = a + $rand[0]
    enf p1.first = (stack_inputs[2] + $rand[0]) * (stack_inputs[3] + $rand[1])

transition_constraints:
    <omitted for brevity>
```

## Transition constraints (`transition_constraints`)

The `transition_constraints` section consists of expressions describing constraints that must be true at each row of the execution trace in order for the proof to be valid.

**Transition constraints are required.** The `transition_constraints` section must be defined and contain at least one transition constraint.

Transition constraints that are defined against auxiliary columns or that use random values from the built-in `$rand` array will be identified as auxiliary constraints.

A transition constraint definition must:

1. start with a block indentation and the `enf` keyword to indicate that the constraint must be _enforced_.
2. continue with an equality expression that describes the constraint. The expression may include numbers, trace columns, periodic columns, random values, and any of the available [operations](./syntax.md#operations).
3. end with a newline.

### Current and next rows

Transition constraints have access to values in the "current" row of the trace to which the constraint is being applied, as well as the "next" row of the trace. The value of a trace column in the next row is specified with the `'` postfix operator, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

### Simple example of transition constraints

The following is a simple example of a valid `transition_constraints` block using values from the current and next rows of the main and auxiliary traces:

```
def TransitionConstraintsExample

trace_columns:
    main: [a, b]
    aux: [p]

public_inputs:
    <omitted for brevity>

boundary_constraints:
    <omitted for brevity>

transition_constraints:
    # these are main constraints. they both express the same constraint.
    enf a' = a + 1
    enf b' - b - 1 = 0

    # this is an auxiliary constraint, since it uses an auxiliary trace column.
    enf p = p' * a
```

### Periodic columns and random values

Transition constraints can access the value of any periodic column in the current row, as well as random values provided by the verifier.

To use periodic column values, the periodic column must be declared in the `periodic_columns` source section. The value in the current row can then be accessed by using the defined identifier of the periodic column.

Random values can be accessed by using array indexing syntax on the `$rand` built-in, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

### Example of transition constraints with periodic columns and random values

The following is an example of a valid `transition_constraints` block that uses periodic columns and random values:

```
def TransitionConstraintsExample

trace_columns:
    main: [a, b]
    aux: [p0, p1]

public_inputs:
    <omitted for brevity>

periodic_columns:
    k: [1, 1, 1, 0]

boundary_constraints:
    <omitted for brevity>

transition_constraints:
    # this is a main constraint that uses a periodic column.
    enf a' = k * a

    # this is an auxiliary constraint that uses a periodic column.
    enf p0' = k * p0

    # these are auxiliary constraints that use random values from the verifier.
    enf b = a + $rand[0]
    enf p1 = k * (a + $rand[0]) * (b + $rand[1])
```
