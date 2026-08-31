# Fuzzy sql search with merkle-hashed normalized AST

Prototype fuzzy similarity search on SQL queries using Merkle-hashed normalized AST

```
cargo run -- examples/workload.sql
cargo run -- examples/workload.sql --explain 2 4
cargo run -- examples/workload.sql --matrix sim.csv --top 20
cargo run -- examples/workload.sql --report report.md              # the whole run, untruncated
cargo run -- examples/workload.sql --explain 2 4 --report pair.md  # just that pair's subtrees
cargo run -- examples/workload.sql --no-literals --no-commute      # rules off, to compare
```

## Pipeline

| step             | where                        | notes |
|------------------|------------------------------|---------|
| split statements | `main.rs::split_statements`  | on `;` outside strings/comments, so one bad query doesn't sink the file (§10) |
| parse            | `sqlparser` `GenericDialect` | per statement; failures are reported, not dropped |
| lower to `Node`  | `lower.rs`                   | hand-written for `Query`/`Select`/`SetExpr`/`TableWithJoins`/`Expr`/`Function`; everything else becomes a labelled leaf (§4.3 "tidy route") |
| normalize        | `lower.rs`                   | §5, one flag per rule |
| Merkle hash      | `node.rs::merkle`            | bottom-up, canonically orders commutative children (§6) |
| WL hashes        | `node.rs::wl_hashes`         | `h_k` for every node and every `k` in `0..=K`, `K=3` (§7) |
| IDF + N²         | `sim.rs`, `main.rs`          | full matrix; corpus is small (§2, §8) |

Hashing follows §6's correctness rules: children concatenated in sorted order (never
XOR'd), arity mixed in, label delimited. 64-bit FNV-1a.

## Normalization rules, all on by default

| flag to disable | rule |
|---|---|
| `--no-literals` | literals → `<int>`/`<str>`/`<bool>`/…; `IN (1,2,3)` → one `<list>` child |
| `--no-aliases`  | table aliases and CTE names resolved (`FROM orders o … o.id` → `orders.id`); unqualified columns in a multi-table FROM become `<unresolved>.id` rather than a guess |
| `--no-flatten`  | nested binary `AND`/`OR` → n-ary |
| `--no-commute`  | canonical child order for commutative nodes; `b < a` → `a > b` by operand hash |
| `--no-orderby`  | `ORDER BY` dropped when there is no `LIMIT` |
| `--no-casefold` | identifiers lower-cased, quoting stripped |

`Expr::Nested` (parens) is always stripped, and projection aliases are dropped.

## Output

1. Parse failures, then a query index (`q0`, `q1`, … with line numbers).
2. Exact duplicates after normalization — root-hash equality, the cheap first pass of §7.
3. Most similar pairs. Headline score is §8's baseline: Jaccard over the feature multiset
   with **both** refinements — each feature weighted by its subtree size, then by corpus
   IDF. Cosine over the same vectors and both containment coefficients
   (`|A∩B|/|A|`, §8) are printed alongside.
4. Shared subexpressions (§9): all nodes grouped by full Merkle hash, kept when
   `size >= --min-size` in ≥2 distinct queries, ranked by `size × #queries`, rendered as
   the original SQL text of one occurrence. The CLI listing is capped at `--top` and cut
   to one line per group.
5. `--matrix PATH` writes the full N×N matrix as CSV.
6. `--report PATH` writes 2–4 to markdown, uncapped and untruncated: the duplicate
   groups with full SQL, a table of *every* scored pair, the shared-subexpression index
   table followed by every group with its queries, line numbers and full SQL, and an
   appendix of every query so the `q`-ids stand on their own. Combined with
   `--explain A B` it reports on that one pair instead — both query texts, then every
   shared subtree. Groups whose node was rewritten by normalization (flattened
   `AND`/`OR`, dropped `ORDER BY`) have no single source fragment and print as
   s-expressions, flagged as such.

`--explain A B` prints the shared subtrees and top-weighted shared features behind one
pair's score; `--dump N` prints the normalized tree with per-node size and hash.

## LIMITATIONS

- Correlated references inside a subquery do not see the outer FROM (only CTE names are
  carried in), so they resolve to `<unresolved>.col`.
- Non-`SELECT` statements lower to a single leaf.
