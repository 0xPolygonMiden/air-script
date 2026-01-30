#[rustfmt::skip]
#[allow(clippy::all)]
pub mod evaluators;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod evaluators_plonky3;

#[rustfmt::skip]
#[allow(clippy::all)]
pub mod evaluators_slice;
#[rustfmt::skip]
#[allow(clippy::all)]
pub mod evaluators_nested_slice_call;

#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod evaluators_slice_plonky3;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod evaluators_nested_slice_call_plonky3;

mod test_air_plonky3;
mod test_air_winterfell;
