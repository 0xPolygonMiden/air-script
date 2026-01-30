#[rustfmt::skip]
#[allow(clippy::all)]
pub mod selectors;
#[rustfmt::skip]
#[allow(clippy::all)]
pub mod selectors_combine_simple;
#[rustfmt::skip]
#[allow(clippy::all)]
mod selectors_combine_complex;
#[rustfmt::skip]
#[allow(clippy::all)]
pub mod selectors_combine_with_list_comprehensions;
#[rustfmt::skip]
#[allow(clippy::all)]
pub mod selectors_with_evaluators;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod selectors_plonky3;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod selectors_combine_simple_plonky3;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
mod selectors_combine_complex_plonky3;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod selectors_with_evaluators_plonky3;
#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(unused_imports)]
pub mod selectors_combine_with_list_comprehensions_plonky3;

mod test_air_plonky3;
mod test_air_winterfell;
