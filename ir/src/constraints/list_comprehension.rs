use super::{graph::CURRENT_ROW, IdentifierType, SemanticError, SymbolTable};
use air_script_core::{
    Expression, Identifier, IndexedTraceAccess, Iterable, ListComprehension, ListFoldingType,
    NamedTraceAccess, VariableType, VectorAccess,
};
use std::collections::BTreeMap;

/// Maps each identifier in the list comprehension to its corresponding [Iterable].
/// For e.g. if the list comprehension is:
/// \[x + y for (x, y) in (a, b)\],
/// the IterableContext will be:
/// { x: Identifier(a), y: Identifier(b) }
type IterableContext = BTreeMap<Identifier, Iterable>;

/// Unfolds a list comprehension into a vector of expressions.
///
/// # Errors
/// - Returns an error if there is an error while parsing any of the expressions in the expanded
/// vector from the list comprehension.
pub fn unfold_lc(
    lc: &ListComprehension,
    symbol_table: &SymbolTable,
) -> Result<Vec<Expression>, SemanticError> {
    let num_iterations = get_num_iterations(lc, symbol_table)?;
    if num_iterations == 0 {
        return Err(SemanticError::InvalidListComprehension(
            "List comprehensions must have at least one iteration.".to_string(),
        ));
    }

    let iterable_context = build_iterable_context(lc)?;
    let vector = (0..num_iterations)
        .map(|i| parse_lc_expr(lc.expression(), &iterable_context, symbol_table, i))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(vector)
}

/// Parses a list comprehension expression and creates an expression based on the index of the
/// expression in the list comprehension.
///
/// # Errors
/// - Returns an error if there is an error while parsing the sub-expression.
fn parse_lc_expr(
    expression: &Expression,
    iterable_context: &IterableContext,
    symbol_table: &SymbolTable,
    i: usize,
) -> Result<Expression, SemanticError> {
    match expression {
        Expression::Elem(ident) => parse_elem(ident, iterable_context, symbol_table, i),
        Expression::NamedTraceAccess(named_trace_access) => {
            parse_named_trace_access(named_trace_access, iterable_context, symbol_table, i)
        }
        Expression::Add(lhs, rhs) => {
            let lhs = parse_lc_expr(lhs, iterable_context, symbol_table, i)?;
            let rhs = parse_lc_expr(rhs, iterable_context, symbol_table, i)?;
            Ok(Expression::Add(Box::new(lhs), Box::new(rhs)))
        }
        Expression::Sub(lhs, rhs) => {
            let lhs = parse_lc_expr(lhs, iterable_context, symbol_table, i)?;
            let rhs = parse_lc_expr(rhs, iterable_context, symbol_table, i)?;
            Ok(Expression::Sub(Box::new(lhs), Box::new(rhs)))
        }
        Expression::Mul(lhs, rhs) => {
            let lhs = parse_lc_expr(lhs, iterable_context, symbol_table, i)?;
            let rhs = parse_lc_expr(rhs, iterable_context, symbol_table, i)?;
            Ok(Expression::Mul(Box::new(lhs), Box::new(rhs)))
        }
        Expression::Exp(lhs, rhs) => {
            let lhs = parse_lc_expr(lhs, iterable_context, symbol_table, i)?;
            let rhs = parse_lc_expr(rhs, iterable_context, symbol_table, i)?;
            Ok(Expression::Exp(Box::new(lhs), Box::new(rhs)))
        }
        Expression::ListFolding(lf_type) => {
            parse_list_folding(lf_type, expression, symbol_table, i)
        }
        _ => Ok(expression.clone()),
    }
}

/// Parses an identifier in a list comprehension expression.
///
/// # Errors
/// - Returns an error if the iterable is an identifier and that identifier does not correspond to
///   a vector.
/// - Returns an error if the iterable is an identifier but is not of a type in set:
///   { TraceColumns, IntegrityVariable, PublicInput, RandomValuesBinding }.
/// - Returns an error if the iterable is a slice and that identifier does not correspond to
///   a vector.
/// - Returns an error if the iterable is an identifier but is not of a type in set:
///   { TraceColumns, IntegrityVariable, PublicInput, RandomValuesBinding }.
fn parse_elem(
    ident: &Identifier,
    iterable_context: &IterableContext,
    symbol_table: &SymbolTable,
    i: usize,
) -> Result<Expression, SemanticError> {
    let iterable = iterable_context.get(ident);
    match iterable {
        // if the corresponding iterable is not present in the iterable context that means the
        // identifier is not part of the list comprehension and we just return it as it is.
        None => Ok(Expression::Elem(ident.clone())),
        Some(iterable_type) => match iterable_type {
            Iterable::Identifier(ident) => {
                let ident_type = symbol_table.get_type(ident.name())?;
                build_ident_expression(ident, ident_type, i)
            }
            Iterable::Range(range) => Ok(Expression::Const((range.start() + i) as u64)),
            Iterable::Slice(ident, range) => {
                let ident_type = symbol_table.get_type(ident.name())?;
                build_slice_ident_expression(ident, ident_type, range.start(), i)
            }
        },
    }
}

/// Parses a named trace access in a list comprehension expression.
///
/// # Errors
/// - Returns an error if the iterable is an identifier and that identifier does not correspond to
///   a trace column.
/// - Returns an error if the iterable is a range.
/// - Returns an error if the iterable is a slice and that identifier does not correspond to a
///   trace column.
fn parse_named_trace_access(
    named_trace_access: &NamedTraceAccess,
    iterable_context: &IterableContext,
    symbol_table: &SymbolTable,
    i: usize,
) -> Result<Expression, SemanticError> {
    let iterable = iterable_context.get(&Identifier(named_trace_access.name().to_string()));
    match iterable {
        // if the corresponding iterable is not present in the iterable context that means the
        // trace column is not part of the list comprehension and we just return it as it is.
        None => Ok(Expression::NamedTraceAccess(named_trace_access.clone())),
        Some(iterable_type) => match iterable_type {
            Iterable::Identifier(ident) => {
                let ident_type = symbol_table.get_type(ident.name())?;
                match ident_type {
                    IdentifierType::TraceColumns(size) => {
                        validate_access(i, size.size())?;
                        Ok(Expression::NamedTraceAccess(NamedTraceAccess::new(
                            ident.clone(),
                            i,
                            named_trace_access.row_offset(),
                        )))
                    }
                    _ => Err(SemanticError::InvalidListComprehension(format!(
                        "Iterable {ident} should contain trace columns"
                    )))?,
                }
            }
            Iterable::Range(_) => Err(SemanticError::InvalidListComprehension(format!(
                "Iterable cannot be of type Range for named trace access {}",
                named_trace_access.name()
            ))),
            Iterable::Slice(ident, range) => {
                let ident_type = symbol_table.get_type(ident.name())?;
                match ident_type {
                    IdentifierType::TraceColumns(trace_columns) => {
                        validate_access(i, trace_columns.size())?;
                        Ok(Expression::NamedTraceAccess(NamedTraceAccess::new(
                            ident.clone(),
                            range.start() + i,
                            named_trace_access.row_offset(),
                        )))
                    }
                    _ => Err(SemanticError::InvalidListComprehension(format!(
                        "Iterable {ident} should contain trace columns"
                    )))?,
                }
            }
        },
    }
}

/// Parses a list folding expression inside a list comprehension expression.
///
/// # Errors
/// - Returns an error if there is an error while unfolding the list comprehension.
fn parse_list_folding(
    lf_type: &ListFoldingType,
    expression: &Expression,
    symbol_table: &SymbolTable,
    i: usize,
) -> Result<Expression, SemanticError> {
    match lf_type {
        ListFoldingType::Sum(lc) | ListFoldingType::Prod(lc) => {
            let iterable_context = build_iterable_context(lc)?;
            let list = unfold_lc(lc, symbol_table)?;
            let mut acc = parse_lc_expr(expression, &iterable_context, symbol_table, i)?;
            for elem in list.iter().skip(1) {
                let expr = parse_lc_expr(elem, &iterable_context, symbol_table, i)?;
                acc = match lf_type {
                    ListFoldingType::Sum(_) => Expression::Add(Box::new(acc), Box::new(expr)),
                    ListFoldingType::Prod(_) => Expression::Mul(Box::new(acc), Box::new(expr)),
                };
            }
            Ok(acc)
        }
    }
}

/// Validates and returns the length of a list comprehension. Checks that the length of all iterables
/// in the list comprehension is the same.
///
/// # Errors
/// - Returns an error if the length of any of the iterables in the list comprehension is not the
///   same.
fn get_num_iterations(
    lc: &ListComprehension,
    symbol_table: &SymbolTable,
) -> Result<usize, SemanticError> {
    let lc_len = get_iterable_len(symbol_table, &lc.context()[0].1)?;
    for (_, iterable) in lc.context().iter().skip(1) {
        let iterable_len = get_iterable_len(symbol_table, iterable)?;
        if iterable_len != lc_len {
            return Err(SemanticError::InvalidListComprehension(
                "All iterables in a list comprehension must have the same length".to_string(),
            ));
        }
    }
    Ok(lc_len)
}

/// Returns the length of an iterable.
///
/// # Errors
/// - Returns an error if the iterable identifier is anything other than a vector in the symbol
///   table if it's a variable.
/// - Returns an error if the iterable is not of type in set:
///   { IntegrityVariable, PublicInput, TraceColumns }
fn get_iterable_len(
    symbol_table: &SymbolTable,
    iterable: &Iterable,
) -> Result<usize, SemanticError> {
    match iterable {
        Iterable::Identifier(ident) => {
            let ident_type = symbol_table.get_type(ident.name())?;
            match ident_type {
                IdentifierType::Variable(_, var_type) => match var_type.value() {
                    VariableType::Vector(vector) => Ok(vector.len()),
                    _ => Err(SemanticError::InvalidListComprehension(format!(
                        "Variable {} should be a vector for a valid list comprehension.",
                        ident.name()
                    ))),
                },
                IdentifierType::PublicInput(size) => Ok(*size),
                IdentifierType::TraceColumns(trace_columns) => Ok(trace_columns.size()),
                _ => Err(SemanticError::InvalidListComprehension(format!(
                    "IdentifierType {ident_type} not supported for list comprehensions"
                ))),
            }
        }
        Iterable::Range(range) | Iterable::Slice(_, range) => Ok(range.end() - range.start()),
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
            "Invalid access index {i} used in list comprehension"
        )))
    }
}

/// Builds an [IterableContext] from a given list comprehension.
///
/// # Errors
/// - Returns an error if there are duplicate members in the list comprehension.
fn build_iterable_context(lc: &ListComprehension) -> Result<IterableContext, SemanticError> {
    let mut iterable_context = IterableContext::new();
    for (member, iterable) in lc.context() {
        if iterable_context
            .insert(member.clone(), iterable.clone())
            .is_some()
        {
            return Err(SemanticError::InvalidListComprehension(format!(
                "Duplicate member {member} in list comprehension"
            )));
        }
    }
    Ok(iterable_context)
}

/// Builds an [Expression] from a given identifier and the index i at which it is being accessed.
///
/// # Errors
/// - Returns an error if the identifier is not of type in set:
///  { IntegrityVariable, PublicInput, TraceColumns, RandomValuesBinding }
/// - Returns an error if the access index is greater than the size of the vector.
/// - Returns an error if the identifier is not a vector in the symbol table if it's a variable.
fn build_ident_expression(
    ident: &Identifier,
    ident_type: &IdentifierType,
    i: usize,
) -> Result<Expression, SemanticError> {
    match ident_type {
        IdentifierType::TraceColumns(trace_columns) => {
            validate_access(i, trace_columns.size())?;
            let trace_segment = trace_columns.trace_segment();
            Ok(Expression::IndexedTraceAccess(IndexedTraceAccess::new(
                trace_segment,
                trace_columns.offset() + i,
                CURRENT_ROW,
            )))
        }
        IdentifierType::Variable(_, var_type) => {
            match var_type.value() {
                VariableType::Vector(vector) => {
                    validate_access(i, vector.len())?;
                    Ok(vector[i].clone())
                }
                // TODO: Handle matrix access
                _ => Err(SemanticError::InvalidListComprehension(format!(
                    "Iterable {ident} should be a vector"
                )))?,
            }
        }
        IdentifierType::PublicInput(size) => {
            validate_access(i, *size)?;
            Ok(Expression::VectorAccess(VectorAccess::new(
                ident.clone(),
                i,
            )))
        }
        IdentifierType::RandomValuesBinding(_, size) => {
            validate_access(i, *size)?;
            Ok(Expression::VectorAccess(VectorAccess::new(
                ident.clone(),
                i,
            )))
        }
        _ => Err(SemanticError::InvalidListComprehension(
            "{ident_type} is an invalid type for a vector".to_string(),
        ))?,
    }
}

/// Builds an [Expression] from a given identifier and a range of the iterable slice.
///
/// # Errors
/// - Returns an error if the identifier is not of type in set:
/// { IntegrityVariable, PublicInput, TraceColumns, RandomValuesBinding }
/// - Returns an error if the access index is greater than the size of the vector.
/// - Returns an error if the identifier is not a vector in the symbol table if it's a variable.
fn build_slice_ident_expression(
    ident: &Identifier,
    ident_type: &IdentifierType,
    range_start: usize,
    i: usize,
) -> Result<Expression, SemanticError> {
    match ident_type {
        IdentifierType::TraceColumns(trace_columns) => {
            validate_access(i, trace_columns.size())?;
            Ok(Expression::NamedTraceAccess(NamedTraceAccess::new(
                ident.clone(),
                range_start + i,
                CURRENT_ROW,
            )))
        }
        IdentifierType::Variable(_, var_type) => {
            match var_type.value() {
                VariableType::Vector(vector) => {
                    validate_access(i, vector.len())?;
                    Ok(vector[range_start + i].clone())
                }
                // TODO: Handle matrix access
                _ => Err(SemanticError::InvalidListComprehension(format!(
                    "Variable {ident} should be a vector for a valid list comprehension"
                )))?,
            }
        }
        IdentifierType::PublicInput(size) => {
            validate_access(i, *size)?;
            Ok(Expression::VectorAccess(VectorAccess::new(
                ident.clone(),
                range_start + i,
            )))
        }
        IdentifierType::RandomValuesBinding(_, size) => {
            validate_access(i, *size)?;
            Ok(Expression::VectorAccess(VectorAccess::new(
                ident.clone(),
                range_start + i,
            )))
        }
        _ => Err(SemanticError::InvalidListComprehension(
            "{ident_type} is an invalid type for a vector".to_string(),
        ))?,
    }
}
