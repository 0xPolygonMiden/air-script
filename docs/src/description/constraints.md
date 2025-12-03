# Constraint description sections

## Boundary constraints (`boundary_constraints`)

The `boundary_constraints` section consists of expressions describing the expected value of columns in the main trace or for the buses at the specified boundary. Column boundaries can be selected using boundary accessors. Valid boundary accessors are `.first`, which selects the first cell of the column to which it is applied, and `.last`, which selects the last cell of the column to which it is applied.

**Boundary constraints are required.** The `boundary_constraints` section must be defined and contain at least one boundary constraint.

A boundary constraint definition must:

1. start with a block indentation and the `enf` keyword to indicate that the constraint must be _enforced_.
2. continue by specifying a column identifier with a boundary accessor, e.g. `a.first` or `a.last`.
3. continue with `=`
4. continue with a right-hand-side "value" expression that evaluates to the required value of the specified column at the specified boundary. The expression may include numbers, named constants, variables, public inputs, the `null` identifier in the case of buses, and any of the available [operations](./syntax.md#operations).
5. end with a `;` and a newline.

### Simple example of boundary constraints

The following is a simple example of a valid `boundary_constraints` source section:

```{{#include ../../examples/boundary_constraints_simple.air}}
```

### Public inputs

Boundary constraints can access public input values provided by the verifier in their value expressions.

To use public inputs, the public input must be declared in the `public_inputs` source section. They can be accessed using array indexing syntax, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

### Example of bus boundary constraints with public inputs

The following is an example of a valid bus `boundary_constraints` source section that uses public inputs:

```{{#include ../../examples/boundary_constraints_buses.air}}
```

### Intermediate variables

Boundary constraints can use intermediate variables to express more complex constraints. Intermediate variables are declared using the `let` keyword, as described in the [variables section](./variables.md).

### Example of boundary constraints with intermediate variables

The following is an example of a valid `boundary_constraints` source section that uses intermediate variables:

```{{#include ../../examples/boundary_constraints_variables.air}}
```

## Integrity constraints (`integrity_constraints`)

The `integrity_constraints` section consists of expressions describing constraints that must be true at each row of the execution trace in order for the proof to be valid.

**Integrity constraints are required.** The `integrity_constraints` section must be defined and contain at least one integrity constraint.

An integrity constraint definition must:

1. start with a block indentation and the `enf` keyword to indicate that the constraint must be _enforced_.
2. continue with an equality expression that describes the constraint. The expression may include numbers, constants, variables, trace columns, periodic columns, bus operations and any of the available [operations](./syntax.md#operations).
3. end with a `;` and a newline.

### Current and next rows

Integrity constraints have access to values in the "current" row of the trace to which the constraint is being applied, as well as the "next" row of the trace. The value of a trace column in the next row is specified with the `'` postfix operator, as described by the [accessor syntax rules](./syntax.md#section-specific-accessors).

### Simple example of integrity constraints

The following is a simple example of a valid `integrity_constraints` source section using values from the current and next rows of the main trace:

```{{#include ../../examples/integrity_constraints_simple.air}}
```

### Periodic columns

Integrity constraints can access the value of any periodic column in the current row.

To use periodic column values, the periodic column must be declared in the `periodic_columns` source section. The value in the current row can then be accessed by using the defined identifier of the periodic column.

### Example of integrity constraints with periodic columns

The following is an example of a valid `integrity_constraints` source section that uses periodic columns:

```{{#include ../../examples/integrity_constraints_periodic.air}}
```

### Buses

Integrity constraints can constrain insertions and removal of elements into / from a given bus. The bus must first be declared in the `buses` source section. More information on bus types and the associated constraints can be found in the [buses](./buses.md) section.

### Example of integrity constraints with buses

The following is an example of a valid `integrity_constraints` source section that uses buses:

```{{#include ../../examples/integrity_constraints_buses.air}}
```

### Intermediate variables

Integrity constraints can use intermediate variables to express more complex constraints. Intermediate variables are declared using the `let` keyword, as described in the [variables section](./variables.md).

### Example of integrity constraints with intermediate variables

The following is an example of a valid `integrity_constraints` source section that uses intermediate variables:

```{{#include ../../examples/integrity_constraints_variables.air}}
```
