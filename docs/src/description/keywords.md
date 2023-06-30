# Keywords

AirScript defines the following keywords:

- `boundary_constraints`: used to declare the [source section](./structure.md#source-sections) where the [boundary constraints are described](./constraints.md#boundary_constraints).
  - `first`: used to access the value of a trace column at the first row of the trace. _It may only be used when defining boundary constraints._
  - `last`: used to access the value of a trace column at the last row of the trace. _It may only be used when defining boundary constraints._
- `const`: used to declare [constants](./declarations.md#constant-constant).
- `def`: used to [define the name](./structure.md) of an AirScript module.
- `enf`: used to describe a single [constraint](./constraints.md).
- `integrity_constraints`: used to declare the [source section](./structure.md#source-sections) where the [integrity constraints are described](./constraints.md#integrity_constraints).
- `let`: used to declare intermediate variables in the boundary_constraints or integrity_constraints source sections.
- `periodic_columns`: used to declare the [source section](./structure.md#source-sections) where the [periodic columns are declared](./declarations.md). _They may only be referenced when defining integrity constraints._
- `prod`: used to fold a list into a single value by multiplying all of the values in the list together.
- `public_inputs`: used to declare the [source section](./structure.md#source-sections) where the [public inputs are declared](./declarations.md). _They may only be referenced when defining boundary constraints._
- `random_values`: used to declare the [source section](./structure.md#source-sections) where the [random values are described](./declarations.md).
- `sum`: used to fold a list into a single value by summing all of the values in the list.
- `trace_columns`: used to declare the [source section](./structure.md#source-sections) where the [execution trace is described](./declarations.md). _They may only be referenced when defining integrity constraints._
  - `main`: used to declare the main execution trace.
  - `aux`: used to declare the auxiliary execution trace.
- `$<identifier>`: used to access random values provided by the verifier.
- `$main`: used to access columns in the main execution trace by index.
- `$aux`: used to access columns in the auxiliary execution trace by index.
