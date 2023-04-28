use super::{
    AccessType, BTreeMap, ComprehensionContext, ConstraintBuilder, Expression, Identifier,
    Iterable, ListComprehension, ListFolding, ListFoldingValueExpr, NodeIndex, SemanticError,
    Symbol, SymbolAccess, SymbolBinding, VariableValueExpr,
};
pub mod constraint_comprehension;
pub mod list_comprehension;

/// Maps each identifier in the list or constraint comprehension to its corresponding [Iterable].
/// For e.g. if the list comprehension is:
/// \[x + y for (x, y) in (a, b)\],
/// the IterableContext will be:
/// { x: Identifier(a), y: Identifier(b) }
/// Similarly if the constraint comprehension is:
/// enf x = y for (x, y) in (a, b),
/// the IterableContext will be:
/// { x: Identifier(a), y: Identifier(b) }
type IterableContext = BTreeMap<Identifier, Iterable>;

impl ConstraintBuilder {
    /// Parses a comprehension expression and creates an expression based on the index of the
    /// expression in the list or constraint comprehension.
    ///
    /// # Errors
    /// - Returns an error if there is an error while parsing the sub-expression.
    fn parse_comprehension_expr(
        &self,
        expression: &Expression,
        iterable_context: &IterableContext,
        i: usize,
    ) -> Result<Expression, SemanticError> {
        match expression {
            Expression::SymbolAccess(symbol_access) => {
                self.parse_symbol_access(symbol_access, iterable_context, i)
            }
            Expression::Add(lhs, rhs) => {
                let lhs = self.parse_comprehension_expr(lhs, iterable_context, i)?;
                let rhs = self.parse_comprehension_expr(rhs, iterable_context, i)?;
                Ok(Expression::Add(Box::new(lhs), Box::new(rhs)))
            }
            Expression::Sub(lhs, rhs) => {
                let lhs = self.parse_comprehension_expr(lhs, iterable_context, i)?;
                let rhs = self.parse_comprehension_expr(rhs, iterable_context, i)?;
                Ok(Expression::Sub(Box::new(lhs), Box::new(rhs)))
            }
            Expression::Mul(lhs, rhs) => {
                let lhs = self.parse_comprehension_expr(lhs, iterable_context, i)?;
                let rhs = self.parse_comprehension_expr(rhs, iterable_context, i)?;
                Ok(Expression::Mul(Box::new(lhs), Box::new(rhs)))
            }
            Expression::Exp(lhs, rhs) => {
                let lhs = self.parse_comprehension_expr(lhs, iterable_context, i)?;
                let rhs = self.parse_comprehension_expr(rhs, iterable_context, i)?;
                Ok(Expression::Exp(Box::new(lhs), Box::new(rhs)))
            }
            Expression::ListFolding(lf_type) => self.parse_list_folding(lf_type, expression, i),
            _ => Ok(expression.clone()),
        }
    }

    /// Parses an identifier in a list or constraint comprehension expression.
    ///
    /// # Errors
    /// - Returns an error if the iterable is an identifier and that identifier does not correspond to
    ///   a vector.
    /// - Returns an error if the iterable is an identifier but is not of a type in set:
    ///   { Trace, Variable, PublicInput, RandomValues }.
    /// - Returns an error if the iterable is a slice and that identifier does not correspond to
    ///   a vector.
    fn parse_symbol_access(
        &self,
        symbol_access: &SymbolAccess,
        iterable_context: &IterableContext,
        i: usize,
    ) -> Result<Expression, SemanticError> {
        let iterable = iterable_context.get(symbol_access.ident());
        match iterable {
            // if the corresponding iterable is not present in the iterable context that means the
            // identifier is not part of the comprehension and we just return it as it is.
            None => Ok(Expression::SymbolAccess(symbol_access.clone())),
            Some(iterable_type) => match iterable_type {
                Iterable::Identifier(ident) => {
                    let symbol = self.symbol_table.get_symbol(ident.name())?;
                    build_ident_expression(symbol, i, symbol_access.offset())
                }
                // TODO: check range handling now that trace bindings are included in SymbolAccess
                Iterable::Range(range) => Ok(Expression::Const((range.start() + i) as u64)),
                Iterable::Slice(ident, range) => {
                    let symbol = self.symbol_table.get_symbol(ident.name())?;
                    build_slice_ident_expression(symbol, range.start(), i, symbol_access.offset())
                }
            },
        }
    }

    /// Parses a list folding expression inside a list or constraint comprehension expression.
    ///
    /// # Errors
    /// - Returns an error if there is an error while unfolding the comprehension expression.
    fn parse_list_folding(
        &self,
        lf_type: &ListFolding,
        expression: &Expression,
        i: usize,
    ) -> Result<Expression, SemanticError> {
        match lf_type {
            ListFolding::Sum(lf_value_type) | ListFolding::Prod(lf_value_type) => {
                let list = self.build_list_from_list_folding_value(lf_value_type)?;
                let iterable_context =
                    if let ListFoldingValueExpr::ListComprehension(lc) = lf_value_type {
                        build_iterable_context(&lc.context().to_vec())?
                    } else {
                        BTreeMap::new()
                    };
                if list.is_empty() {
                    return Err(SemanticError::list_folding_empty_list(lf_value_type));
                }
                let mut acc = self.parse_comprehension_expr(expression, &iterable_context, i)?;
                for elem in list.iter().skip(1) {
                    let expr = self.parse_comprehension_expr(elem, &iterable_context, i)?;
                    acc = match lf_type {
                        ListFolding::Sum(_) => Expression::Add(Box::new(acc), Box::new(expr)),
                        ListFolding::Prod(_) => Expression::Mul(Box::new(acc), Box::new(expr)),
                    };
                }
                Ok(acc)
            }
        }
    }

    /// Validates and returns the length of a list or constraint comprehension. Checks that the
    /// length of all iterables in the list or constraint comprehension is the same.
    ///
    /// # Errors
    /// - Returns an error if the length of any of the iterables in the comprehension is not the
    ///   same.
    fn get_num_iterations(
        &self,
        comprehension_context: &[(Identifier, Iterable)],
    ) -> Result<usize, SemanticError> {
        let comprehension_len = self.get_iterable_len(&comprehension_context[0].1)?;
        for (_, iterable) in comprehension_context.iter().skip(1) {
            let iterable_len = self.get_iterable_len(iterable)?;
            if iterable_len != comprehension_len {
                return Err(SemanticError::InvalidComprehension(
                    "All iterables in a comprehension must have the same length".to_string(),
                ));
            }
        }
        Ok(comprehension_len)
    }

    /// Returns the length of an iterable.
    ///
    /// # Errors
    /// - Returns an error if the iterable identifier is anything other than a vector in the symbol
    ///   table if it's a variable.
    /// - Returns an error if the iterable is not of type in set:
    ///   { Variable, PublicInput, Trace }
    fn get_iterable_len(&self, iterable: &Iterable) -> Result<usize, SemanticError> {
        match iterable {
            Iterable::Identifier(ident) => {
                let symbol = self.symbol_table.get_symbol(ident.name())?;
                match symbol.binding() {
                    SymbolBinding::Variable(variable_type) => match variable_type {
                        VariableValueExpr::Vector(vector) => Ok(vector.len()),
                        _ => Err(SemanticError::InvalidComprehension(format!(
                            "VariableBinding {} should be a vector for a valid comprehension.",
                            symbol.name(),
                        ))),
                    },
                    SymbolBinding::PublicInput(size) => Ok(*size),
                    SymbolBinding::Trace(trace_columns) => Ok(trace_columns.size()),
                    _ => Err(SemanticError::InvalidComprehension(format!(
                        "SymbolBinding {} not supported for comprehensions",
                        symbol.binding(),
                    ))),
                }
            }
            Iterable::Range(range) | Iterable::Slice(_, range) => Ok(range.end() - range.start()),
        }
    }
}

/// Checks if the access index is valid. Returns an error if the access index is greater than
/// the size of the vector.
///
/// # Errors
/// - Returns an error if the access index is greater than the size of the vector.
fn validate_access(i: usize, size: usize) -> Result<(), SemanticError> {
    if i < size {
        Ok(())
    } else {
        Err(SemanticError::IndexOutOfRange(format!(
            "Invalid access index {i} used in comprehension"
        )))
    }
}

/// Builds an [IterableContext] from a given list or constraint comprehension.
///
/// # Errors
/// - Returns an error if there are duplicate members in the comprehension context.
fn build_iterable_context(
    comprehension_context: &ComprehensionContext,
) -> Result<IterableContext, SemanticError> {
    let mut iterable_context = IterableContext::new();
    for (member, iterable) in comprehension_context {
        if iterable_context
            .insert(member.clone(), iterable.clone())
            .is_some()
        {
            return Err(SemanticError::InvalidComprehension(format!(
                "Duplicate member {member} in comprehension"
            )));
        }
    }
    Ok(iterable_context)
}

/// Builds an [Expression] from a given identifier and the index i at which it is being accessed.
///
/// # Errors
/// - Returns an error if the identifier is not of type in set:
///  { Variable, PublicInput, Trace, RandomValues }
/// - Returns an error if the access index is greater than the size of the vector.
/// - Returns an error if the identifier is not a vector in the symbol table if it's a variable.
fn build_ident_expression(
    symbol: &Symbol,
    i: usize,
    offset: usize,
) -> Result<Expression, SemanticError> {
    match symbol.binding() {
        SymbolBinding::Trace(trace_columns) => {
            validate_access(i, trace_columns.size())?;
            Ok(Expression::SymbolAccess(SymbolAccess::new(
                Identifier(symbol.name().to_string()),
                AccessType::Vector(i),
                offset,
            )))
        }
        SymbolBinding::Variable(variable_type) => {
            match variable_type {
                VariableValueExpr::Vector(vector) => {
                    validate_access(i, vector.len())?;
                    Ok(vector[i].clone())
                }
                // TODO: Handle matrix access
                _ => Err(SemanticError::InvalidComprehension(format!(
                    "Iterable {} should be a vector",
                    symbol.name()
                )))?,
            }
        }
        SymbolBinding::PublicInput(size) | SymbolBinding::RandomValues(_, size) => {
            validate_access(i, *size)?;
            let access_type = AccessType::Vector(i);
            let symbol_access =
                SymbolAccess::new(Identifier(symbol.name().to_string()), access_type, offset);
            Ok(Expression::SymbolAccess(symbol_access))
        }
        _ => Err(SemanticError::InvalidComprehension(
            "{ident_type} is an invalid type for a vector".to_string(),
        ))?,
    }
}

/// Builds an [Expression] from a given identifier and a range of the iterable slice.
///
/// # Errors
/// - Returns an error if the identifier is not of type in set:
/// { Variable, PublicInput, Trace, RandomValues }
/// - Returns an error if the access index is greater than the size of the vector.
/// - Returns an error if the identifier is not a vector in the symbol table if it's a variable.
fn build_slice_ident_expression(
    symbol: &Symbol,
    range_start: usize,
    i: usize,
    offset: usize,
) -> Result<Expression, SemanticError> {
    match symbol.binding() {
        SymbolBinding::Trace(trace_columns) => {
            validate_access(i, trace_columns.size())?;
            Ok(Expression::SymbolAccess(SymbolAccess::new(
                Identifier(symbol.name().to_string()),
                AccessType::Vector(range_start + i),
                offset,
            )))
        }
        SymbolBinding::Variable(variable) => {
            match variable {
                VariableValueExpr::Vector(vector) => {
                    validate_access(i, vector.len())?;
                    Ok(vector[range_start + i].clone())
                }
                // TODO: Handle matrix access
                _ => Err(SemanticError::InvalidComprehension(format!(
                    "VariableBinding {} should be a vector for a valid comprehension",
                    symbol.name(),
                )))?,
            }
        }
        SymbolBinding::PublicInput(size) | SymbolBinding::RandomValues(_, size) => {
            validate_access(i, *size)?;
            let access_type = AccessType::Vector(range_start + i);
            let symbol_access =
                SymbolAccess::new(Identifier(symbol.name().to_string()), access_type, offset);
            Ok(Expression::SymbolAccess(symbol_access))
        }
        _ => Err(SemanticError::InvalidComprehension(
            "{ident_type} is an invalid type for a vector".to_string(),
        ))?,
    }
}
