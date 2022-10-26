# Reserved keywords and basic types

This page specifies the basic syntax, types, and keywords of the AirScript.

## Keywords

AirScript defines the following keywords:

- `$rand`: used to access random values provided by the verifier.
- `def`: used to [define the name](./structure.md) of an AirScript module.
- `boundary_constraints`: used to declare the [source section](./structure.md) where the [boundary constraints are described](./constraints.md#boundary_constraints).
  - `first`: used to access the value of a trace column at the first row of the trace. _It may only be used when defining boundary constraints._
  - `last`: used to access the value of a trace column at the last row of the trace. _It may only be used when defining boundary constraints._
- `enf`: used to describe a single [constraint](./constraints.md).
- `public_inputs`: used to declare the [source section](./structure.md) where the [public inputs are declared](./declarations.md). _They may only be referenced when defining boundary constraints._
- `periodic_columns`: used to declare the [source section](./structure.md) where the [periodic columns are declared](./declarations.md). _They may only be referenced when defining transition constraints._
- `trace_columns`: used to declare the [source section](./structure.md) where the [execution trace is described](./declarations.md).
  - `main`: used to declare the main execution trace
  - `aux`: used to declare the auxiliary execution trace
- `transition_constraints`: used to declare the [source section](./structure.md) where the [transition constraints are described](./constraints.md#transition_constraints).

## Built-in variables

Built-in variables are identified by the starting character `$`.

### $rand

Currently, the only built-in is `$rand`, which is used to get random values provided by the verifier.

These random values may be accessed by using the indexing operator on `$rand`. For example, `$rand[i]` provides the `ith` random value.

Random values may only be accessed within source sections for constraints, i.e. the [`boundary_constraints` section](./constraints.md#boundary-constraints-boundary_constraints) and the [`transition_constraints` section](./constraints.md#transition-constraints-transition_constraints).

## Delimiters and special characters

- `:` is used as a delimiter when declaring [source sections](./primitives.md) and [types](./declarations.md)
- `.` is used to access a boundary on a trace column, e.g. `a.first` or `a.last`
- `[` and `]` are used for defining arrays in [type declarations](./declarations.md) and for indexing in [constraint descriptions](./constraints.md)
- `,` is used as a delimiter for defining arrays in [type declarations](./declarations.md)
- `$` is used to indicate a special built-in value. Currently, it is only used with `$rand` for accessing random values.

## Identifiers

Valid identifiers are strings that start with a letter `a-z` or `A-Z` followed by any combination of letters, digits `0-9` or an underscore `_`.

## Numbers

The only supported numbers are integers, and all integers are parsed as u64. Using a number larger than 2^64 - 1 will result in a `ParseError`.

## Operations

The following operations are supported in [constraint descriptions](./constraints.md) with the specified syntax:

- Equality (`a = b`)
- Addition (`a + b`)
- Subtraction (`a - b`)
- Multiplication (`a * b`)
- Exponentiation by a constant integer x (`a^x`)

The following operations are **not supported**:

- Negation
- Division
- Inversion

### Parentheses and complex expressions

Parentheses (`(` and `)`) are supported and can be included in any expression except exponentiation, where complex expressions are not allowed.

The following is allowed:

```
a * (b + c)
```

The following is not allowed:

```
a^(2 + 3)
```

## Section-specific accessors

These accessors may only be used in the specified [source section](./structure.md) in which they are described below.

### [Boundary constraints](./constraints.md#boundary_constraints)

The following accessors may only be applied to trace columns when they are in boundary constraint definitions.

- First boundary (`.first`): accesses the trace column's value in the first row. It is only supported in [boundary constraint descriptions](./constraints.md#boundary_constraints)
- Last boundary (`.last`): accesses the trace column's value in the first row. It is only supported in [boundary constraint descriptions](./constraints.md#boundary_constraints)

The following accessor may only be applied to public inputs declared in `public_inputs` when they are referenced in boundary constraint definitions.

- Indexing (`input_name[i]`): public inputs may be accessed by using the indexing operator on the declared identifier name with an index value that is less than the declared size of its array.

Here is an example of usage of first and last boundaries and a public input within a boundary constraint:

```
trace_columns:
    main: [a]

public_inputs:
    stack_inputs: [4]
    stack_outputs: [4]

boundary_constraints:
    enf a.first = stack_inputs[0]
    enf a.last = stack_outputs[0]
```

### [Transition constraints](./constraints.md#transition_constraints)

The following accessor may only be applied to trace columns when they are referenced in transition constraint definitions.

- Next Row (`a'`): `'` is a postfix operator that indicates the value of the specified trace column in the next row. It is only supported in [transition constraint descriptions](./constraints.md#transition_constraints).

Here is an example of usage of the Next Row operator within a transition constraint:

```
trace_columns:
  main: [a]
  aux: [p]

transition_constraints:
  enf p' = p * a
```
