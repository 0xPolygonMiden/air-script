# Pure Functions

Pure functions let you package reusable expressions that return values. Unlike
[evaluators](./evaluators.md), pure functions do not describe constraints directly. Use a pure
function when you want to reuse a computed value; use an evaluator when you want to reuse one or
more `enf` statements.

## Defining a function

A pure function is declared with the `fn` keyword, followed by the function name, typed
parameters, a return type, and a body:

```
fn fold_sum(a: felt[4]) -> felt {
    return a[0] + a[1] + a[2] + a[3];
}
```

The examples below use `felt` scalars and `felt[n]` fixed-size vectors.

## Local bindings and return values

Function bodies can contain local `let` bindings before the final `return`:

```
fn fold_vec(a: felt[4]) -> felt {
    let m = a[0] * a[1];
    let n = m * a[2];
    let o = n * a[3];
    return o;
}
```

This is useful when the same expression would otherwise be repeated across multiple constraints.

## Calling functions from constraints

Pure functions can be called anywhere a value expression is valid inside
`integrity_constraints`: in local bindings, in binary expressions, and on the right-hand side of
`enf` statements.

```
trace_columns {
    main: [t, v, b[4]],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf v.first = 0;
}

fn fold_sum(a: felt[4]) -> felt {
    return a[0] + a[1] + a[2] + a[3];
}

fn fold_vec(a: felt[4]) -> felt {
    let m = a[0] * a[1];
    let n = m * a[2];
    let o = n * a[3];
    return o;
}

integrity_constraints {
    let folded = fold_vec(b);
    enf folded = 1;
    enf t * fold_vec(b) = 1;
    enf v' = fold_sum(b) * fold_vec(b);
}
```

In the example above, `fold_vec(b)` and `fold_sum(b)` are used both as standalone computed values
and as part of larger expressions.

## Calling functions from other functions

Pure functions can call other pure functions:

```
fn fold_sum(a: felt[4]) -> felt {
    return a[0] + a[1] + a[2] + a[3];
}

fn fold_vec(a: felt[4]) -> felt {
    let m = a[0] * a[1];
    let n = m * a[2];
    let o = n * a[3];
    return o;
}

fn combined(a: felt[4]) -> felt {
    return fold_sum(a) * fold_vec(a);
}
```

## Using functions inside evaluators

Pure functions can also be called from [evaluators](./evaluators.md):

```
fn fold_sum(a: felt[4]) -> felt {
    return a[0] + a[1] + a[2] + a[3];
}

ev transition([a, b[4]]) {
    enf a' = fold_sum(b);
}
```

Use pure functions for reusable value-level logic, and evaluators for reusable constraint-level
logic.
