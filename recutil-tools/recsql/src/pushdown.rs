//! Translate a subset of DataFusion `Expr`s into recutils selection
//! expression syntax for filter pushdown into the librec layer.
//!
//! Returning `None` from [`expr_to_selection_expression`] means "this
//! predicate does not have a known selection-expression equivalent" — the
//! caller should leave it for DataFusion to evaluate above the provider.
//!
//! Supported today:
//! - `Column <op> Literal` and `Literal <op> Column` for `=`, `!=`, `<`,
//!   `<=`, `>`, `>=`. Operand types: `Int64`, `Float64`, `Utf8`.
//! - Logical `AND`, `OR`, `NOT` of supported sub-expressions.
//! - `Cast(Column, _)` is unwrapped — DataFusion often inserts a cast
//!   around a column when it gets coerced to match the literal's type.

use arrow::datatypes::{DataType, Schema};
use datafusion::common::{Column, ScalarValue};
use datafusion::logical_expr::{BinaryExpr, Cast, Expr, Operator};

pub(crate) fn expr_to_selection_expression(expr: &Expr, schema: &Schema) -> Option<String> {
    match expr {
        Expr::BinaryExpr(BinaryExpr { left, op, right }) => match op {
            Operator::And => {
                let l = expr_to_selection_expression(left, schema)?;
                let r = expr_to_selection_expression(right, schema)?;
                Some(format!("({l}) && ({r})"))
            }
            Operator::Or => {
                let l = expr_to_selection_expression(left, schema)?;
                let r = expr_to_selection_expression(right, schema)?;
                Some(format!("({l}) || ({r})"))
            }
            Operator::Eq
            | Operator::NotEq
            | Operator::Lt
            | Operator::LtEq
            | Operator::Gt
            | Operator::GtEq => binary_compare(left, *op, right, schema),
            _ => None,
        },
        Expr::Not(inner) => {
            let s = expr_to_selection_expression(inner, schema)?;
            Some(format!("!({s})"))
        }
        _ => None,
    }
}

fn binary_compare(
    left: &Expr,
    op: Operator,
    right: &Expr,
    schema: &Schema,
) -> Option<String> {
    let (col, scalar, swap) = if let (Some(c), Some(s)) =
        (column_under_cast(left), as_literal(right))
    {
        (c, s, false)
    } else if let (Some(s), Some(c)) = (as_literal(left), column_under_cast(right)) {
        (c, s, true)
    } else {
        return None;
    };
    let field = schema.field_with_name(&col.name).ok()?;
    let expr_value = literal_to_selection_value(scalar, field.data_type())?;
    let selection_op = selection_op_str(op, swap)?;
    Some(format!("{} {} {}", col.name, selection_op, expr_value))
}

fn column_under_cast(e: &Expr) -> Option<&Column> {
    let mut cur = e;
    loop {
        match cur {
            Expr::Column(c) => return Some(c),
            Expr::Cast(Cast { expr, .. }) => cur = expr,
            _ => return None,
        }
    }
}

fn as_literal(e: &Expr) -> Option<&ScalarValue> {
    match e {
        Expr::Literal(s, _) => Some(s),
        _ => None,
    }
}

fn selection_op_str(op: Operator, swap: bool) -> Option<&'static str> {
    Some(match (op, swap) {
        (Operator::Eq, _) => "=",
        (Operator::NotEq, _) => "!=",
        (Operator::Lt, false) | (Operator::Gt, true) => "<",
        (Operator::LtEq, false) | (Operator::GtEq, true) => "<=",
        (Operator::Gt, false) | (Operator::Lt, true) => ">",
        (Operator::GtEq, false) | (Operator::LtEq, true) => ">=",
        _ => return None,
    })
}

fn literal_to_selection_value(value: &ScalarValue, dt: &DataType) -> Option<String> {
    match dt {
        DataType::Int64 => match value {
            ScalarValue::Int64(Some(n)) => Some(n.to_string()),
            ScalarValue::Int32(Some(n)) => Some(n.to_string()),
            _ => None,
        },
        DataType::Float64 => match value {
            ScalarValue::Float64(Some(f)) => Some(format_float(*f)),
            ScalarValue::Float32(Some(f)) => Some(format_float(*f as f64)),
            _ => None,
        },
        DataType::Utf8 => match value {
            ScalarValue::Utf8(Some(s)) | ScalarValue::LargeUtf8(Some(s)) => {
                Some(format!("\"{}\"", escape_selection_string(s)))
            }
            _ => None,
        },
        _ => None,
    }
}

fn format_float(f: f64) -> String {
    if f.is_finite() && f.fract() == 0.0 {
        format!("{f:.1}")
    } else {
        f.to_string()
    }
}

fn escape_selection_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => {
                out.push('\\');
                out.push('"');
            }
            '\\' => {
                out.push('\\');
                out.push('\\');
            }
            other => out.push(other),
        }
    }
    out
}
