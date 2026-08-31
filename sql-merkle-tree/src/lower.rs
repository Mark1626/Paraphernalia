//! AST -> generic tree (spec §4.3, "tidy route") with normalization (§5) applied
//! inline. Every rule is behind a flag in `Norm`.

use std::collections::HashMap;

use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, GroupByExpr, Ident, Join,
    JoinConstraint, JoinOperator, ObjectName, OrderBy, OrderByKind, Query, Select, SelectItem,
    SetExpr, Statement, TableFactor, TableWithJoins, Value,
};

use crate::node::Node;

#[derive(Clone, Debug)]
pub struct Norm {
    /// literals -> typed placeholders, `IN (..)` list -> `<list>`
    pub literals: bool,
    /// table aliases / CTE names resolved to base relations
    pub aliases: bool,
    /// nested binary AND/OR -> n-ary
    pub flatten: bool,
    /// canonical child order for commutative nodes (applied during hashing)
    pub commute: bool,
    /// drop ORDER BY when there is no LIMIT
    pub orderby: bool,
    /// case-fold identifiers
    pub casefold: bool,
}

impl Default for Norm {
    fn default() -> Self {
        Norm {
            literals: true,
            aliases: true,
            flatten: true,
            commute: true,
            orderby: true,
            casefold: true,
        }
    }
}

impl Norm {
    /// The rules left on, for the report header.
    pub fn describe(&self) -> String {
        let on: Vec<&str> = [
            ("literals", self.literals),
            ("aliases", self.aliases),
            ("flatten", self.flatten),
            ("commute", self.commute),
            ("orderby", self.orderby),
            ("casefold", self.casefold),
        ]
        .iter()
        .filter(|(_, v)| *v)
        .map(|(n, _)| *n)
        .collect();
        if on.is_empty() {
            "none".to_string()
        } else {
            on.join(", ")
        }
    }
}

/// Names declared inside the query, and therefore resolvable without a catalog.
#[derive(Clone, Default)]
pub struct Scope {
    /// alias (or table name) -> base relation
    alias: HashMap<String, String>,
    /// relations in this FROM, in order; length 1 means unqualified columns resolve
    tables: Vec<String>,
    /// CTE names visible to nested scopes
    ctes: HashMap<String, String>,
}

impl Scope {
    /// Scope for a nested query: CTEs stay visible, FROM relations do not.
    fn inner(&self) -> Scope {
        Scope {
            alias: self.ctes.clone(),
            tables: Vec::new(),
            ctes: self.ctes.clone(),
        }
    }

    fn resolve(&self, parts: &[String]) -> String {
        if parts.len() >= 2 {
            let qual = &parts[parts.len() - 2];
            let col = &parts[parts.len() - 1];
            let table = self.alias.get(qual).cloned().unwrap_or_else(|| qual.clone());
            format!("{table}.{col}")
        } else if self.tables.len() == 1 {
            format!("{}.{}", self.tables[0], parts[0])
        } else {
            // §5: unqualified column in a multi-table join is genuinely
            // unresolvable without a catalog -- say so rather than guess.
            format!("<unresolved>.{}", parts[0])
        }
    }
}

pub struct Lowerer {
    pub norm: Norm,
}

/// Debug output starts with the variant name; good enough for the tail of the
/// AST we do not lower structurally.
fn variant_name<T: std::fmt::Debug>(x: &T) -> String {
    let s = format!("{x:?}");
    let name = s
        .split(|c| c == '(' || c == '{' || c == ' ' || c == '[')
        .next()
        .unwrap_or("node");
    name.to_lowercase()
}

impl Lowerer {
    pub fn new(norm: Norm) -> Self {
        Lowerer { norm }
    }

    fn ident(&self, i: &Ident) -> String {
        if self.norm.casefold {
            i.value.to_lowercase()
        } else {
            i.value.clone()
        }
    }

    fn object_name_last(&self, n: &ObjectName) -> String {
        n.0.last()
            .and_then(|p| p.as_ident())
            .map(|i| self.ident(i))
            .unwrap_or_else(|| n.to_string())
    }

    fn object_name(&self, n: &ObjectName) -> String {
        let parts: Vec<String> = n
            .0
            .iter()
            .map(|p| match p.as_ident() {
                Some(i) => self.ident(i),
                None => p.to_string(),
            })
            .collect();
        parts.join(".")
    }

    pub fn statement(&self, s: &Statement) -> Node {
        match s {
            Statement::Query(q) => self.query(q, &Scope::default()),
            other => Node::leaf(format!("stmt:{}", variant_name(other))),
        }
    }

    fn query(&self, q: &Query, outer: &Scope) -> Node {
        let mut scope = outer.clone();
        let mut kids = Vec::new();
        if let Some(with) = &q.with {
            for cte in &with.cte_tables {
                let name = self.ident(&cte.alias.name);
                let body = self.query(&cte.query, &scope);
                kids.push(Node::new("cte", vec![Node::leaf(format!("table:{name}")), body]));
                scope.alias.insert(name.clone(), name.clone());
                scope.ctes.insert(name.clone(), name);
            }
        }
        kids.push(self.set_expr(&q.body, &scope));
        let limited = q.limit_clause.is_some() || q.fetch.is_some();
        if let Some(ob) = &q.order_by {
            if limited || !self.norm.orderby {
                kids.push(self.order_by(ob, &scope));
            }
        }
        if limited {
            kids.push(Node::leaf("limit"));
        }
        Node::new("query", kids).with_sql(q.to_string())
    }

    fn order_by(&self, ob: &OrderBy, scope: &Scope) -> Node {
        let kids = match &ob.kind {
            OrderByKind::Expressions(v) => v.iter().map(|e| self.expr(&e.expr, scope)).collect(),
            OrderByKind::All(_) => vec![Node::leaf("all")],
        };
        Node::new("order_by", kids)
    }

    fn set_expr(&self, se: &SetExpr, scope: &Scope) -> Node {
        match se {
            SetExpr::Select(s) => self.select(s, scope),
            SetExpr::Query(q) => self.query(q, scope),
            SetExpr::SetOperation {
                left, op, right, ..
            } => Node::new(
                format!("setop:{}", variant_name(op)),
                vec![self.set_expr(left, scope), self.set_expr(right, scope)],
            ),
            other => Node::leaf(format!("setexpr:{}", variant_name(other))),
        }
    }

    fn select(&self, s: &Select, parent: &Scope) -> Node {
        let mut scope = parent.inner();
        for twj in &s.from {
            self.collect_scope(twj, &mut scope);
        }

        let mut kids = Vec::new();

        let proj: Vec<Node> = s
            .projection
            .iter()
            .map(|item| match item {
                SelectItem::UnnamedExpr(e) => self.expr(e, &scope),
                SelectItem::ExprWithAlias { expr, .. } => self.expr(expr, &scope),
                SelectItem::ExprWithAliases { expr, .. } => self.expr(expr, &scope),
                SelectItem::Wildcard(_) => Node::leaf("col:*"),
                SelectItem::QualifiedWildcard(_, _) => Node::leaf("col:*"),
            })
            .collect();
        let proj_sql: Vec<String> = s.projection.iter().map(|p| p.to_string()).collect();
        kids.push(Node::new("projection", proj).with_sql(format!("SELECT {}", proj_sql.join(", "))));

        if !s.from.is_empty() {
            let mut rels = Vec::new();
            for twj in &s.from {
                self.emit_from(twj, &scope, &mut rels);
            }
            let from_sql: Vec<String> = s.from.iter().map(|t| t.to_string()).collect();
            kids.push(Node::new("from", rels).with_sql(format!("FROM {}", from_sql.join(", "))));
        }
        if s.distinct.is_some() {
            kids.push(Node::leaf("distinct"));
        }
        if let Some(w) = &s.selection {
            kids.push(Node::new("where", vec![self.expr(w, &scope)]).with_sql(format!("WHERE {w}")));
        }
        match &s.group_by {
            GroupByExpr::Expressions(v, _) if !v.is_empty() => {
                let sql: Vec<String> = v.iter().map(|e| e.to_string()).collect();
                kids.push(
                    Node::new("group_by", v.iter().map(|e| self.expr(e, &scope)).collect())
                        .with_sql(format!("GROUP BY {}", sql.join(", "))),
                )
            }
            GroupByExpr::All(_) => kids.push(Node::leaf("group_by:all")),
            _ => {}
        }
        if let Some(h) = &s.having {
            kids.push(Node::new("having", vec![self.expr(h, &scope)]).with_sql(format!("HAVING {h}")));
        }
        Node::new("select", kids).with_sql(s.to_string())
    }

    fn collect_scope(&self, twj: &TableWithJoins, scope: &mut Scope) {
        self.collect_factor(&twj.relation, scope);
        for j in &twj.joins {
            self.collect_factor(&j.relation, scope);
        }
    }

    fn collect_factor(&self, tf: &TableFactor, scope: &mut Scope) {
        if !self.norm.aliases {
            return;
        }
        match tf {
            TableFactor::Table { name, alias, .. } => {
                let base = self.object_name_last(name);
                if let Some(a) = alias {
                    scope.alias.insert(self.ident(&a.name), base.clone());
                }
                scope.alias.insert(base.clone(), base.clone());
                scope.tables.push(base);
            }
            TableFactor::Derived { alias, .. } => {
                let name = alias
                    .as_ref()
                    .map(|a| self.ident(&a.name))
                    .unwrap_or_else(|| "<derived>".to_string());
                scope.alias.insert(name.clone(), name.clone());
                scope.tables.push(name);
            }
            _ => {}
        }
    }

    fn emit_from(&self, twj: &TableWithJoins, scope: &Scope, out: &mut Vec<Node>) {
        out.push(self.table_factor(&twj.relation, scope));
        for j in &twj.joins {
            out.push(self.table_factor(&j.relation, scope));
            out.push(self.join_node(j, scope));
        }
    }

    fn join_node(&self, j: &Join, scope: &Scope) -> Node {
        use JoinOperator::*;
        let (kind, constraint) = match &j.join_operator {
            Join(c) | Inner(c) | StraightJoin(c) => ("inner", Some(c)),
            Left(c) | LeftOuter(c) => ("left", Some(c)),
            Right(c) | RightOuter(c) => ("right", Some(c)),
            FullOuter(c) => ("full", Some(c)),
            CrossJoin(c) => ("cross", Some(c)),
            Semi(c) | LeftSemi(c) => ("semi", Some(c)),
            RightSemi(c) => ("right_semi", Some(c)),
            Anti(c) | LeftAnti(c) => ("anti", Some(c)),
            RightAnti(c) => ("right_anti", Some(c)),
            other => (join_other(other), None),
        };
        match constraint {
            Some(JoinConstraint::On(e)) => {
                Node::new(format!("join_on:{kind}"), vec![self.expr(e, scope)])
                    .with_sql(format!("{} JOIN ... ON {e}", kind.to_uppercase()))
            }
            Some(JoinConstraint::Using(cols)) => Node::new(
                format!("join_using:{kind}"),
                cols.iter()
                    .map(|c| Node::leaf(format!("col:{}", self.object_name(c))))
                    .collect(),
            ),
            _ => Node::leaf(format!("join:{kind}")),
        }
    }

    fn table_factor(&self, tf: &TableFactor, scope: &Scope) -> Node {
        match tf {
            TableFactor::Table { name, .. } => {
                Node::leaf(format!("table:{}", self.object_name_last(name)))
                    .with_sql(name.to_string())
            }
            TableFactor::Derived { subquery, .. } => {
                Node::new("derived", vec![self.query(subquery, &scope.inner())])
            }
            other => Node::leaf(format!("rel:{}", variant_name(other))),
        }
    }

    fn placeholder(&self, v: &Value) -> String {
        match v {
            Value::Number(n, _) => {
                if n.contains('.') {
                    "<float>".into()
                } else {
                    "<int>".into()
                }
            }
            Value::SingleQuotedString(_)
            | Value::DoubleQuotedString(_)
            | Value::TripleSingleQuotedString(_)
            | Value::TripleDoubleQuotedString(_)
            | Value::EscapedStringLiteral(_)
            | Value::NationalStringLiteral(_)
            | Value::UnicodeStringLiteral(_) => "<str>".into(),
            Value::Boolean(_) => "<bool>".into(),
            Value::Null => "<null>".into(),
            Value::Placeholder(_) => "<param>".into(),
            _ => "<lit>".into(),
        }
    }

    fn expr(&self, e: &Expr, scope: &Scope) -> Node {
        let node = self.expr_inner(e, scope);
        match node.sql {
            Some(_) => node,
            None => node.with_sql(e.to_string()),
        }
    }

    fn expr_inner(&self, e: &Expr, scope: &Scope) -> Node {
        match e {
            Expr::Identifier(i) => {
                let parts = vec![self.ident(i)];
                let name = if self.norm.aliases {
                    scope.resolve(&parts)
                } else {
                    parts[0].clone()
                };
                Node::leaf(format!("col:{name}"))
            }
            Expr::CompoundIdentifier(v) => {
                let parts: Vec<String> = v.iter().map(|i| self.ident(i)).collect();
                let name = if self.norm.aliases {
                    scope.resolve(&parts)
                } else {
                    parts.join(".")
                };
                Node::leaf(format!("col:{name}"))
            }
            Expr::Value(v) => {
                if self.norm.literals {
                    Node::leaf(self.placeholder(&v.value))
                } else {
                    Node::leaf(format!("lit:{}", v.value))
                }
            }
            Expr::TypedString(t) => {
                if self.norm.literals {
                    Node::leaf(format!("<{}>", variant_name(&t.data_type)))
                } else {
                    Node::leaf(format!("lit:{}", e))
                }
            }
            Expr::Nested(inner) => self.expr_inner(inner, scope),
            Expr::BinaryOp { left, op, right } => {
                let label = format!("op:{}", variant_name(op));
                if self.norm.flatten && (label == "op:and" || label == "op:or") {
                    let flat_label = if label == "op:and" { "and" } else { "or" };
                    let mut kids = Vec::new();
                    self.flatten_bool(e, op, scope, &mut kids);
                    Node::new(flat_label, kids)
                } else {
                    Node::new(
                        label,
                        vec![self.expr(left, scope), self.expr(right, scope)],
                    )
                }
            }
            Expr::UnaryOp { op, expr } => Node::new(
                format!("unop:{}", variant_name(op)),
                vec![self.expr(expr, scope)],
            ),
            Expr::IsNull(x) => Node::new("is_null", vec![self.expr(x, scope)]),
            Expr::IsNotNull(x) => Node::new("is_not_null", vec![self.expr(x, scope)]),
            Expr::IsTrue(x) => Node::new("is_true", vec![self.expr(x, scope)]),
            Expr::IsFalse(x) => Node::new("is_false", vec![self.expr(x, scope)]),
            Expr::InList {
                expr,
                list,
                negated,
            } => {
                let label = if *negated { "not_in" } else { "in" };
                let mut kids = vec![self.expr(expr, scope)];
                if self.norm.literals {
                    // §5: collapse the list so its length does not dominate.
                    kids.push(Node::leaf("<list>"));
                } else {
                    kids.extend(list.iter().map(|x| self.expr(x, scope)));
                }
                Node::new(label, kids)
            }
            Expr::InSubquery {
                expr,
                subquery,
                negated,
            } => Node::new(
                if *negated { "not_in_subquery" } else { "in_subquery" },
                vec![self.expr(expr, scope), self.query(subquery, scope)],
            ),
            Expr::Between {
                expr,
                negated,
                low,
                high,
            } => Node::new(
                if *negated { "not_between" } else { "between" },
                vec![
                    self.expr(expr, scope),
                    self.expr(low, scope),
                    self.expr(high, scope),
                ],
            ),
            Expr::Like {
                negated,
                expr,
                pattern,
                ..
            }
            | Expr::ILike {
                negated,
                expr,
                pattern,
                ..
            } => Node::new(
                if *negated { "not_like" } else { "like" },
                vec![self.expr(expr, scope), self.expr(pattern, scope)],
            ),
            Expr::Cast {
                expr, data_type, ..
            } => Node::new(
                format!("cast:{}", variant_name(data_type)),
                vec![self.expr(expr, scope)],
            ),
            Expr::Extract { field, expr, .. } => Node::new(
                format!("extract:{}", variant_name(field)),
                vec![self.expr(expr, scope)],
            ),
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                let mut kids = Vec::new();
                if let Some(o) = operand {
                    kids.push(self.expr(o, scope));
                }
                for w in conditions {
                    kids.push(Node::new(
                        "when",
                        vec![self.expr(&w.condition, scope), self.expr(&w.result, scope)],
                    ));
                }
                if let Some(er) = else_result {
                    kids.push(Node::new("else", vec![self.expr(er, scope)]));
                }
                Node::new("case", kids)
            }
            Expr::Exists { subquery, negated } => Node::new(
                if *negated { "not_exists" } else { "exists" },
                vec![self.query(subquery, scope)],
            ),
            Expr::Subquery(q) => Node::new("subquery", vec![self.query(q, scope)]),
            Expr::Tuple(v) => {
                Node::new("tuple", v.iter().map(|x| self.expr(x, scope)).collect())
            }
            Expr::Function(f) => {
                let name = self.object_name(&f.name);
                let mut kids = Vec::new();
                match &f.args {
                    FunctionArguments::List(list) => {
                        if list.duplicate_treatment.is_some() {
                            kids.push(Node::leaf("distinct"));
                        }
                        for a in &list.args {
                            kids.push(self.function_arg(a, scope));
                        }
                    }
                    FunctionArguments::Subquery(q) => {
                        kids.push(self.query(q, scope));
                    }
                    FunctionArguments::None => {}
                }
                if f.over.is_some() {
                    kids.push(Node::leaf("over"));
                }
                Node::new(format!("func:{name}"), kids)
            }
            other => Node::leaf(format!("expr:{}", variant_name(other))),
        }
    }

    fn function_arg(&self, a: &FunctionArg, scope: &Scope) -> Node {
        let inner = match a {
            FunctionArg::Unnamed(x) => Some(x),
            FunctionArg::Named { arg, .. } => Some(arg),
            FunctionArg::ExprNamed { arg, .. } => Some(arg),
        };
        match inner {
            Some(FunctionArgExpr::Expr(e)) => self.expr(e, scope),
            Some(FunctionArgExpr::QualifiedWildcard(_)) | Some(FunctionArgExpr::Wildcard) => {
                Node::leaf("col:*")
            }
            _ => Node::leaf("arg"),
        }
    }

    /// §5: nested binary AND/OR -> n-ary.
    fn flatten_bool(
        &self,
        e: &Expr,
        op: &sqlparser::ast::BinaryOperator,
        scope: &Scope,
        out: &mut Vec<Node>,
    ) {
        match e {
            Expr::BinaryOp {
                left,
                op: o,
                right,
            } if o == op => {
                self.flatten_bool(left, op, scope, out);
                self.flatten_bool(right, op, scope, out);
            }
            Expr::Nested(inner) => self.flatten_bool(inner, op, scope, out),
            other => out.push(self.expr(other, scope)),
        }
    }
}

fn join_other(op: &JoinOperator) -> &'static str {
    match op {
        JoinOperator::CrossApply => "cross_apply",
        JoinOperator::OuterApply => "outer_apply",
        JoinOperator::AsOf { .. } => "asof",
        _ => "other",
    }
}
