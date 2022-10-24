# Type declaration sections

## Trace columns (`trace_columns`)

A `trace_columns` section contains declarations for `main` trace columns or `aux` (auxiliary) trace columns.

The `main` and `aux` declarations define the shape of the main and auxiliary execution traces respectively and declare identifiers which can be used to refer to each of the columns in that trace.

**A `trace_columns` section with a `main` declaration is required for an Air DSL to be valid.** The `aux` declaration is optional.

The following is a valid `trace_columns` block:

```
trace_columns:
    main: [a, b, c]
    aux: [d, e]
```

In the above example, the main execution trace for the AIR has 3 columns which can be referenced by `a`, `b`, and `c`. Internally, these identifiers will reference the trace columns with indices 0, 1, and 2 respectively. Similarly, the auxiliary execution trace has 2 columns which can be referenced by `d` and `e`.

## Public inputs (`public_inputs`)

A `public_inputs` section contains declarations for public inputs. Currently, each public input must be provided as a vector of a fixed size, but there is no limit to how many of them can be declared within the `public_inputs` section.

**Public inputs are required.** There must be at least one pubic input declared.

Each public input is described by an identifier and an array length (`n`) in the following format:

```
identifier: [n]
```

The following is an example of a valid `public_inputs` block:

```
public_inputs:
    program_hash: [4]
    stack_inputs: [16]
    stack_outputs: [16]
```

In the above example, the public input `program_hash` is an array of length `4`. `stack_inputs` and `stack_outputs` are both arrays of length `16`.

Public inputs can be referenced by [boundary constraints](./constraints.md#boundary_constraints) by using the identifier and an index. For example, the 3rd element of the `program_hash` declared above would be referenced as `program_hash[2]`.

## Periodic columns (`periodic_columns`)

A `periodic_columns` section contains declarations for periodic columns used in the description and evaluation of transition constraints. Each periodic column declares an array of periodic values which can then be referenced by the declared identifier.

There is no limit to how many of them can be declared within the `periodic_columns` section.

**Periodic columns are optional.** It is equally valid to define an empty `periodic_columns` section or to omit the `periodic_columns` section declaration entirely.

Each periodic column is described by an identifier and an array of integers in the following format. These integers are the periodic values.

```
identifier: [i, j, k, n]
```

The length of each of the array must be a power of two which is greater than or equal to `2`.

The following is an example of a valid `periodic_columns` block:

```
periodic_columns:
    k0: [0, 0, 0, 1]
    k1: [1, 1, 1, 1, 1, 1, 1, 0]
```

In the above example, `k0` declares a periodic column with a cycle of length `4`, and `k1` declares a periodic column with a cycle of length `8`.

Periodic columns can be referenced by [transition constraints](./constraints.md#transition_constraints) by using the column's identifier.

When constraints are evaluated, these periodic values always refer to the value of the column in the current row. For example, when evaluating a transition constraint such as `enf k0 * a = 0`, `k0` would be evaluated as `0` in rows `0`, `1`, `2` of the trace and as `1` in row `3`, and then the cycle would repeat. Attempting to refer to the "next" row of a periodic column, such as by `k0'`, is invalid and will cause a `ParseError`.
