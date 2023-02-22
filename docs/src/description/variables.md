# Local Variables and Built-in Variables
This section describes the syntax for declaring local variables and built-in variables.

## Variables

In AirScript, variables can be declared in `boundary_constraints` and `integrity_constraints` sections and can contain any expression that would be valid within that source section. Variables can be of type scalar, vector or matrix. In the example below, `x` is a variable of type `scalar`, `y` is a variable of type `vector` and `z` is a variable of type `matrix`.

```
def VariablesExample

const A = 1
const B = 2

trace_columns:
    main: [a, b, c, d]
    aux: [e, f]

public_inputs:
    stack_inputs: [16]

random_values:
    rand: [16]

boundary_constraints:
    let x = stack_inputs[0] + stack_inputs[1]   
    let y = [$rand[0], $rand[1]]  
    enf e.first = x + y[0] + y[1]

integrity_constraints:
    let z = [
        [a + b, c + d],
        [A * a, B * b]
    ]
    enf a' = z[0][0] + z[0][1] + z[1][0] + z[1][1]
```

## Built-in variables

Built-in variables are identified by the starting character `$`. There are two built-in variables:

### \$main

`$main` is used to access columns in the [main execution trace](./appendix.md#main-vs-auxiliary-execution-trace-segments-main-and-aux).

These columns may be accessed by using the indexing operator on `$main`. For example, `$main[i]` provides the `(i+1)th` column in the main execution trace.

Columns using the `$main` built-in may only be accessed within source sections for integrity constraints, i.e. the [`integrity_constraints` section](./constraints.md#integrity-constraints-integrity_constraints).

### \$aux

`$aux` is used to access columns in the [auxiliary execution trace](./appendix.md#main-vs-auxiliary-execution-trace-segments-main-and-aux).

These columns may be accessed by using the indexing operator on `$aux`. For example, `$aux[i]` provides the `(i+1)th` column in the auxiliary execution trace.

Columns using the `$aux` built-in may only be accessed within source sections for integrity constraints, i.e. the [`integrity_constraints` section](./constraints.md#integrity-constraints-integrity_constraints).