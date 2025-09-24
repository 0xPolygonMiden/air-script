use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_field::Field;
use p3_matrix::{
    Matrix,
    dense::{RowMajorMatrix, RowMajorMatrixView},
    stack::VerticalPair,
};

pub trait BaseAirWithPeriodicColumns<F>: BaseAir<F> {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![]
    }
}

pub trait AirBuilderWithPeriodicColumns: AirBuilder {
    type PeriodicColumnsVar: Field + Into<Self::Expr>;

    fn periodic_columns(&self) -> Vec<Self::PeriodicColumnsVar> {
        vec![]
    }
}

pub(crate) fn check_constraints_with_periodic_columns<F, A>(
    air: &A,
    main: &RowMajorMatrix<F>,
    public_values: &Vec<F>,
) where
    F: Field,
    A: for<'a> Air<DebugConstraintBuilderWithPeriodicColumns<'a, F>>
        + BaseAirWithPeriodicColumns<F>,
{
    let height = main.height();

    (0..height).for_each(|i| {
        let i_next = (i + 1) % height;

        let local = main.row_slice(i).unwrap(); // i < height so unwrap should never fail.
        let next = main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
        let main = VerticalPair::new(
            RowMajorMatrixView::new_row(&*local),
            RowMajorMatrixView::new_row(&*next),
        );
        let periodic_columns = air.get_periodic_columns();

        let mut builder = DebugConstraintBuilderWithPeriodicColumns {
            row_index: i,
            main,
            public_values,
            is_first_row: F::from_bool(i == 0),
            is_last_row: F::from_bool(i == height - 1),
            is_transition: F::from_bool(i != height - 1),
            periodic_columns,
        };

        air.eval(&mut builder);
    });
}

/// A builder that runs constraint assertions during testing.
///
/// Used in conjunction with [`check_constraints`] to simulate
/// an execution trace and verify that the AIR logic enforces all constraints.
#[derive(Debug)]
pub struct DebugConstraintBuilderWithPeriodicColumns<'a, F: Field> {
    /// The index of the row currently being evaluated.
    row_index: usize,
    /// A view of the current and next row as a vertical pair.
    main: VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>,
    /// The public values provided for constraint validation (e.g. inputs or outputs).
    public_values: &'a [F],
    /// A flag indicating whether this is the first row.
    is_first_row: F,
    /// A flag indicating whether this is the last row.
    is_last_row: F,
    /// A flag indicating whether this is a transition row (not the last row).
    is_transition: F,
    /// The periodic columns provided for constraint validation.
    periodic_columns: Vec<Vec<F>>,
}

impl<'a, F> AirBuilderWithPeriodicColumns for DebugConstraintBuilderWithPeriodicColumns<'a, F>
where
    F: Field + Into<Self::Expr>,
{
    type PeriodicColumnsVar = F;

    fn periodic_columns(&self) -> Vec<Self::PeriodicColumnsVar> {
        self.periodic_columns
            .iter()
            .map(|col| col[self.row_index % col.len()])
            .collect::<Vec<_>>()
    }
}

impl<'a, F> AirBuilder for DebugConstraintBuilderWithPeriodicColumns<'a, F>
where
    F: Field,
{
    type F = F;
    type Expr = F;
    type Var = F;
    type M = VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>;

    fn main(&self) -> Self::M {
        self.main
    }

    fn is_first_row(&self) -> Self::Expr {
        self.is_first_row
    }

    fn is_last_row(&self) -> Self::Expr {
        self.is_last_row
    }

    /// # Panics
    /// This function panics if `size` is not `2`.
    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            self.is_transition
        } else {
            panic!("only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        assert_eq!(x.into(), F::ZERO, "constraints had nonzero value on row {}", self.row_index);
    }

    fn assert_eq<I1: Into<Self::Expr>, I2: Into<Self::Expr>>(&mut self, x: I1, y: I2) {
        let x = x.into();
        let y = y.into();
        assert_eq!(x, y, "values didn't match on row {}: {} != {}", self.row_index, x, y);
    }
}

impl<'a, F> AirBuilderWithPublicValues for DebugConstraintBuilderWithPeriodicColumns<'a, F>
where
    F: Field,
{
    type PublicVar = Self::F;

    fn public_values(&self) -> &[Self::F] {
        self.public_values
    }
}
