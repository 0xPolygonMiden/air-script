#!/bin/bash

# Notes:
# - Run from root of repository
# - We avoid looping on all found air-script/src/tests/**/*.air files to make it easier to notice changes

cargo build --release

# Winterfell Backend

./target/release/airc transpile --target winterfell ./air-script/src/tests/binary/binary.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/bitwise/bitwise.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_complex.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_simple.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_simple_with_evaluators.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_varlen_boundary_both.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_varlen_boundary_last.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/buses/buses_varlen_boundary_first.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/constant_in_range/constant_in_range.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/constants/constants.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/constraint_comprehension/constraint_comprehension.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/evaluators/evaluators.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/fibonacci/fibonacci.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/functions/functions_simple.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/functions/functions_complex.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/indexed_trace_access/indexed_trace_access.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/list_comprehension/list_comprehension_nested.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/list_comprehension/list_comprehension.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/list_folding/list_folding.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/periodic_columns/periodic_columns.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/pub_inputs/pub_inputs.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/selectors/selectors_combine_complex.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/selectors/selectors_combine_simple.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/selectors/selectors_combine_with_list_comprehensions.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/selectors/selectors_with_evaluators.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/selectors/selectors.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/system/system.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/trace_col_groups/trace_col_groups.air
./target/release/airc transpile --target winterfell ./air-script/src/tests/variables/variables.air

# Plonky3 Backend
./target/release/airc transpile --target plonky3 ./air-script/src/tests/binary/binary.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/bitwise/bitwise.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_complex.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_simple.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_simple_with_evaluators.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_varlen_boundary_both.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_varlen_boundary_last.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/buses/buses_varlen_boundary_first.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/computed_indices/computed_indices_complex.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/computed_indices/computed_indices_simple.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/constant_in_range/constant_in_range.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/constants/constants.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/constraint_comprehension/constraint_comprehension.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/evaluators/evaluators.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/fibonacci/fibonacci.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/functions/functions_simple.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/functions/functions_complex.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/indexed_trace_access/indexed_trace_access.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/list_comprehension/list_comprehension_nested.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/list_comprehension/list_comprehension.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/list_folding/list_folding.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/periodic_columns/periodic_columns.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/pub_inputs/pub_inputs.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/selectors/selectors_combine_complex.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/selectors/selectors_combine_simple.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/selectors/selectors_combine_with_list_comprehensions.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/selectors/selectors_with_evaluators.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/selectors/selectors.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/system/system.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/trace_col_groups/trace_col_groups.air
./target/release/airc transpile --target plonky3 ./air-script/src/tests/variables/variables.air
