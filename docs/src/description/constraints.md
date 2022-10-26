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
    # at least one public input section is required.
    program_hash: [4]

boundary_constraints:
    # these are main constraints.
    enf a.first = 0
    enf a.last = 10

    # these are auxiliary constraints, since they are defined against auxiliary trace columns.
    enf p.first = 1
    enf p.last = 1
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
```

## Transition constraints (`transition_constraints`)

TODO
