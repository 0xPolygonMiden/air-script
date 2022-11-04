# File Structure

An AIR Script file consists of a name definition and several source sections which contain declarations and constraints. The declarations describe the shape of the execution trace to which constraints are applied and the public inputs, and periodic columns that are used for computing those constraints. The constraints describe boundary and transition constraints which must hold for an execution trace and set of public inputs in order for them to be valid (i.e. in order for a valid proof to be generated or verification to pass).

## Air name definition

An AirScript file starts with a definition of the name of the Air module, such as:

```
def ExampleAir
```

It must:

- Begin with the `def` keyword, with no indentation.
- Continue with a string that does not begin with a number.
- End with a newline.

## Source sections

All source sections must:

- Begin with a valid source section keyword, followed by `:` and a newline.
- Have all contents in an indented block.

This is an example of a well-formed source section:

```
# source section keyword
trace_columns:
    # indented content block
    main: [a, b, c, d]
    aux: [x, y, z]
```

Valid keywords for type declaration sections are the following:

- `trace_columns`
- `public_inputs`
- `periodic_columns`

Valid keywords for constraint description sections are the following:

- `boundary_constraints`
- `transition_constraints`

By convention, type declaration sections precede constraint description sections, although this is not a requirement of the language.
