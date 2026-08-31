//! Fuzzy similarity search over SQL queries via Merkle-hashed ASTs.
//! Minimal end-to-end prototype of sql-merkle-similarity-spec.md.

mod lower;
mod node;
mod sim;

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use lower::{Lowerer, Norm};
use node::Node;
use sim::{Bag, Corpus, FULL, Key};

struct Args {
    path: String,
    k: usize,
    top: usize,
    min_size: u32,
    matrix: Option<String>,
    report: Option<String>,
    explain: Option<(usize, usize)>,
    dump: Option<usize>,
    norm: Norm,
}

const USAGE: &str = "\
usage: sql-merkle <queries.sql> [options]

  --k N          WL depth cap (default 3)
  --top N        pairs / subexpressions to print (default 15)
  --min-size N   min subtree size for the shared-subexpression report (default 5)
  --matrix PATH  write the full similarity matrix as CSV
  --report PATH  write the full report as markdown -- duplicates, every ranked pair,
                 every shared subexpression, SQL untruncated. With --explain,
                 reports on that pair instead.
  --explain A B  print the shared subtrees behind one pair of query indices
  --dump N       print the normalized tree for query N

normalization toggles (all on by default, §5):
  --no-literals --no-aliases --no-flatten --no-commute --no-orderby --no-casefold
";

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        path: String::new(),
        k: 3,
        top: 15,
        min_size: 5,
        matrix: None,
        report: None,
        explain: None,
        dump: None,
        norm: Norm::default(),
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    let num = |s: &Option<&String>| -> Result<usize, String> {
        s.ok_or("missing value".to_string())?
            .parse()
            .map_err(|_| "expected a number".to_string())
    };
    while i < argv.len() {
        let arg = argv[i].as_str();
        match arg {
            "--k" => {
                a.k = num(&argv.get(i + 1))?;
                i += 1;
            }
            "--top" => {
                a.top = num(&argv.get(i + 1))?;
                i += 1;
            }
            "--min-size" => {
                a.min_size = num(&argv.get(i + 1))? as u32;
                i += 1;
            }
            "--matrix" => {
                a.matrix = Some(argv.get(i + 1).ok_or("--matrix needs a path")?.clone());
                i += 1;
            }
            "--report" => {
                a.report = Some(argv.get(i + 1).ok_or("--report needs a path")?.clone());
                i += 1;
            }
            "--dump" => {
                a.dump = Some(num(&argv.get(i + 1))?);
                i += 1;
            }
            "--explain" => {
                let x = num(&argv.get(i + 1))?;
                let y = num(&argv.get(i + 2))?;
                a.explain = Some((x, y));
                i += 2;
            }
            "--no-literals" => a.norm.literals = false,
            "--no-aliases" => a.norm.aliases = false,
            "--no-flatten" => a.norm.flatten = false,
            "--no-commute" => a.norm.commute = false,
            "--no-orderby" => a.norm.orderby = false,
            "--no-casefold" => a.norm.casefold = false,
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown flag {other}")),
            other => a.path = other.to_string(),
        }
        i += 1;
    }
    if a.path.is_empty() {
        return Err(USAGE.to_string());
    }
    Ok(a)
}

struct Raw {
    text: String,
    line: usize,
}

/// Split on `;` outside strings and comments, so one unparseable statement does
/// not take the file down with it (§10).
fn split_statements(sql: &str) -> Vec<Raw> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut line = 1usize;
    let mut start = 1usize;
    let mut in_str = false;
    let mut in_dq = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = sql.chars().peekable();

    fn push_ch(cur: &mut String, c: char, line: usize, start: &mut usize) {
        if cur.trim().is_empty() {
            *start = line;
        }
        cur.push(c);
    }

    while let Some(c) = chars.next() {
        if c == '\n' {
            line += 1;
        }
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if in_str {
            cur.push(c);
            if c == '\'' {
                in_str = false;
            }
            continue;
        }
        if in_dq {
            cur.push(c);
            if c == '"' {
                in_dq = false;
            }
            continue;
        }
        match c {
            '\'' => {
                in_str = true;
                push_ch(&mut cur, c, line, &mut start);
            }
            '"' => {
                in_dq = true;
                push_ch(&mut cur, c, line, &mut start);
            }
            '-' if chars.peek() == Some(&'-') => in_line_comment = true,
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                in_block_comment = true;
            }
            ';' => {
                if !cur.trim().is_empty() {
                    out.push(Raw {
                        text: cur.trim().to_string(),
                        line: start,
                    });
                }
                cur.clear();
            }
            _ => push_ch(&mut cur, c, line, &mut start),
        }
    }
    if !cur.trim().is_empty() {
        out.push(Raw {
            text: cur.trim().to_string(),
            line: start,
        });
    }
    out
}

struct Query {
    line: usize,
    text: String,
    tree: Node,
    bag: Bag,
}

fn snippet(text: &str, width: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= width {
        one
    } else {
        let cut: String = one.chars().take(width - 1).collect();
        format!("{cut}...")
    }
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let text = match std::fs::read_to_string(&args.path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {e}", args.path);
            std::process::exit(2);
        }
    };

    let dialect = GenericDialect {};
    let lowerer = Lowerer::new(args.norm.clone());
    let mut queries: Vec<Query> = Vec::new();
    let mut failures: Vec<(usize, String, String)> = Vec::new();
    let mut weights: HashMap<Key, u32> = HashMap::new();

    for raw in split_statements(&text) {
        match Parser::parse_sql(&dialect, &raw.text) {
            Ok(stmts) => {
                for st in &stmts {
                    let mut tree = lowerer.statement(st);
                    tree.merkle(args.norm.commute);
                    tree.wl_hashes(args.k);
                    let bag = sim::features(&tree, args.k, &mut weights);
                    queries.push(Query {
                        line: raw.line,
                        text: raw.text.clone(),
                        tree,
                        bag,
                    });
                }
            }
            // §10: parse failures are reported, never silently dropped.
            Err(e) => failures.push((raw.line, snippet(&raw.text, 60), e.to_string())),
        }
    }

    println!(
        "parsed {} queries from {} ({} failed), K={}",
        queries.len(),
        args.path,
        failures.len(),
        args.k
    );
    if !failures.is_empty() {
        println!("\n== parse failures ==");
        for (line, snip, err) in &failures {
            println!("  line {line}: {snip}\n    {err}");
        }
    }
    if queries.is_empty() {
        return;
    }

    for (i, q) in queries.iter().enumerate() {
        println!(
            "  q{i:<3} line {:<5} {} nodes  {}",
            q.line,
            q.tree.size,
            snippet(&q.text, 70)
        );
    }

    if let Some(d) = args.dump {
        if let Some(q) = queries.get(d) {
            println!("\n== normalized tree for q{d} ==");
            print_tree(&q.tree, 0);
        }
        return;
    }

    let bags: Vec<Bag> = queries.iter().map(|q| q.bag.clone()).collect();
    let corpus = Corpus::build(&bags, weights);
    let vectors: Vec<HashMap<Key, f64>> = bags.iter().map(|b| corpus.vector(b)).collect();

    let dupes = duplicate_groups(&queries);
    print_duplicates(&dupes);

    if let Some((a, b)) = args.explain {
        explain(&queries, &vectors, &corpus, a, b, &args);
        return;
    }

    let pairs = rank_pairs(&vectors);
    print_pairs(&pairs, args.top);

    let groups = group_by_hash(&queries, args.min_size);
    print_shared_subexpressions(&groups, args.min_size, args.top);
    if let Some(path) = &args.report {
        write_out(
            path,
            corpus_report(&queries, &dupes, &pairs, &groups, failures.len(), &args),
        );
    }

    if let Some(path) = &args.matrix {
        let mut out = String::new();
        out.push_str("query");
        for i in 0..queries.len() {
            let _ = write!(out, ",q{i}");
        }
        out.push('\n');
        for i in 0..queries.len() {
            let _ = write!(out, "q{i}");
            for j in 0..queries.len() {
                let s = if i == j {
                    1.0
                } else {
                    sim::jaccard(&vectors[i], &vectors[j])
                };
                let _ = write!(out, ",{s:.4}");
            }
            out.push('\n');
        }
        match std::fs::write(path, out) {
            Ok(()) => println!("\nsimilarity matrix written to {path}"),
            Err(e) => eprintln!("\ncannot write {path}: {e}"),
        }
    }
}

fn print_tree(n: &Node, depth: usize) {
    println!(
        "{:indent$}{} [size={} h={:016x}]",
        "",
        n.label,
        n.size,
        n.hash,
        indent = depth * 2
    );
    for c in &n.children {
        print_tree(c, depth + 1);
    }
}

struct Group {
    size: u32,
    label: String,
    example: String,
    /// False when `example` is the s-expression fallback rather than real SQL.
    example_is_sql: bool,
    queries: BTreeSet<usize>,
    occurrences: u32,
}

/// §7: root-hash equality is a dedup test, run first because it is free.
fn duplicate_groups(queries: &[Query]) -> Vec<Vec<usize>> {
    let mut by_hash: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, q) in queries.iter().enumerate() {
        by_hash.entry(q.tree.hash).or_default().push(i);
    }
    let mut groups: Vec<Vec<usize>> = by_hash.into_values().filter(|v| v.len() > 1).collect();
    groups.sort_by_key(|v| v[0]);
    groups
}

fn print_duplicates(groups: &[Vec<usize>]) {
    if groups.is_empty() {
        return;
    }
    println!("\n== exact duplicates after normalization ==");
    for g in groups {
        let names: Vec<String> = g.iter().map(|i| format!("q{i}")).collect();
        println!("  {}", names.join(" = "));
    }
}

struct PairRow {
    a: usize,
    b: usize,
    jaccard: f64,
    cosine: f64,
    /// §8 containment: the fraction of a found in b, and the reverse.
    cab: f64,
    cba: f64,
}

/// §8: full N^2 pass; the corpus is small enough that nothing clever is needed.
fn rank_pairs(vectors: &[HashMap<Key, f64>]) -> Vec<PairRow> {
    let mut rows = Vec::new();
    for i in 0..vectors.len() {
        for j in (i + 1)..vectors.len() {
            rows.push(PairRow {
                a: i,
                b: j,
                jaccard: sim::jaccard(&vectors[i], &vectors[j]),
                cosine: sim::cosine(&vectors[i], &vectors[j]),
                cab: sim::containment(&vectors[i], &vectors[j]),
                cba: sim::containment(&vectors[j], &vectors[i]),
            });
        }
    }
    rows.sort_by(|x, y| y.jaccard.partial_cmp(&x.jaccard).unwrap());
    rows
}

fn print_pairs(rows: &[PairRow], top: usize) {
    println!("\n== most similar pairs (features weighted by subtree size x IDF) ==");
    println!("  pair       jaccard  cosine   A in B   B in A");
    for p in rows.iter().take(top) {
        let (i, j) = (p.a, p.b);
        println!(
            "  q{i:<3} q{j:<3}  {:.3}    {:.3}    {:.3}    {:.3}",
            p.jaccard, p.cosine, p.cab, p.cba
        );
    }
    if rows.len() > top {
        println!(
            "  ... {} more pair(s); --report PATH writes all of them",
            rows.len() - top
        );
    }
}

/// §9: group every node across every query by full Merkle hash.
fn group_by_hash(queries: &[Query], min_size: u32) -> Vec<Group> {
    let mut groups: HashMap<u64, Group> = HashMap::new();
    for (qi, q) in queries.iter().enumerate() {
        q.tree.walk(&mut |n: &Node| {
            if n.size < min_size {
                return;
            }
            let g = groups.entry(n.hash).or_insert_with(|| Group {
                size: n.size,
                label: n.label.clone(),
                example: n.render(),
                example_is_sql: n.has_sql(),
                queries: BTreeSet::new(),
                occurrences: 0,
            });
            g.queries.insert(qi);
            g.occurrences += 1;
        });
    }
    let mut v: Vec<Group> = groups
        .into_values()
        .filter(|g| g.queries.len() >= 2)
        .collect();
    v.sort_by_key(|g| std::cmp::Reverse(g.size as u64 * g.queries.len() as u64));
    v
}

fn print_shared_subexpressions(groups: &[Group], min_size: u32, top: usize) {
    println!("\n== shared subexpressions (rank = size x #queries) ==");
    if groups.is_empty() {
        println!("  none with size >= {min_size} in 2+ queries");
        return;
    }
    for g in groups.iter().take(top) {
        let names: Vec<String> = g.queries.iter().map(|i| format!("q{i}")).collect();
        println!(
            "  [{}] size {:<3} {} queries ({})",
            g.label,
            g.size,
            g.queries.len(),
            names.join(" ")
        );
        println!("      {}", snippet(&g.example, 110));
    }
    if groups.len() > top {
        println!(
            "  ... {} more group(s); --report PATH writes all of them, untruncated",
            groups.len() - top
        );
    }
}

/// One subtree shared by two queries, or by a whole group.
struct Shared {
    size: u32,
    label: String,
    sql: String,
    /// False when `sql` is the s-expression fallback rather than real SQL.
    is_sql: bool,
}

fn explain(
    queries: &[Query],
    vectors: &[HashMap<Key, f64>],
    corpus: &Corpus,
    a: usize,
    b: usize,
    args: &Args,
) {
    if a >= queries.len() || b >= queries.len() {
        eprintln!("query index out of range");
        return;
    }
    let min_size = args.min_size;
    let s = sim::jaccard(&vectors[a], &vectors[b]);
    println!(
        "\n== q{a} vs q{b}: jaccard {s:.3}, cosine {:.3} ==",
        sim::cosine(&vectors[a], &vectors[b])
    );
    println!("  q{a}: {}", snippet(&queries[a].text, 100));
    println!("  q{b}: {}", snippet(&queries[b].text, 100));
    println!(
        "  containment q{a} in q{b} = {:.3}, q{b} in q{a} = {:.3}",
        sim::containment(&vectors[a], &vectors[b]),
        sim::containment(&vectors[b], &vectors[a])
    );

    let shared = shared_subtrees(&queries[a], &queries[b], min_size);
    println!("\n  shared subtrees (size >= {min_size}):");
    if shared.is_empty() {
        println!("    none");
    }
    for sh in shared.iter().take(20) {
        println!("    size {:<3} {}", sh.size, snippet(&sh.sql, 100));
    }
    if shared.len() > 20 {
        println!(
            "    ... {} more; --report PATH writes all of them, untruncated",
            shared.len() - 20
        );
    }

    // Top-weighted shared features, including the depth-capped ones.
    let mut feats: Vec<(f64, Key)> = vectors[a]
        .iter()
        .filter_map(|(k, v)| vectors[b].get(k).map(|w| (v.min(*w), *k)))
        .collect();
    feats.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());
    println!("\n  top shared features (k, weight, idf):");
    for (v, key) in feats.iter().take(10) {
        let kind = if key.0 == FULL {
            "full".to_string()
        } else {
            format!("h_{}", key.0)
        };
        println!(
            "    {kind:<5} weight {v:8.1}  idf {:.2}  size {}",
            corpus.idf(key),
            corpus.weights.get(key).unwrap_or(&0)
        );
    }

    if let Some(path) = &args.report {
        write_out(path, pair_report(&shared, queries, vectors, a, b, args));
    }
}

/// Full-Merkle subtrees present in both queries, i.e. exactly identical
/// normalized fragments, largest first.
fn shared_subtrees(a: &Query, b: &Query, min_size: u32) -> Vec<Shared> {
    let mut mine: HashMap<u64, &Node> = HashMap::new();
    a.tree.walk(&mut |n: &Node| {
        if n.size >= min_size {
            mine.insert(n.hash, n);
        }
    });
    let mut shared: Vec<Shared> = Vec::new();
    let mut seen = BTreeSet::new();
    b.tree.walk(&mut |n: &Node| {
        if let Some(m) = mine.get(&n.hash) {
            if seen.insert(n.hash) {
                shared.push(Shared {
                    size: m.size,
                    label: m.label.clone(),
                    sql: m.render(),
                    is_sql: m.has_sql(),
                });
            }
        }
    });
    shared.sort_by_key(|sh| std::cmp::Reverse(sh.size));
    shared
}

fn write_out(path: &str, contents: String) {
    match std::fs::write(path, contents) {
        Ok(()) => println!("\nreport written to {path}"),
        Err(e) => eprintln!("\ncannot write {path}: {e}"),
    }
}

/// A fence long enough to survive a fragment that itself contains backticks.
fn fenced(out: &mut String, lang: &str, body: &str) {
    let (mut longest, mut run) = (0usize, 0usize);
    for c in body.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let f = "`".repeat(longest.max(2) + 1);
    let _ = writeln!(out, "{f}{lang}\n{body}\n{f}\n");
}

fn report_header(out: &mut String, title: &str, queries: &[Query], args: &Args) {
    let _ = writeln!(out, "# {title}\n");
    let _ = writeln!(out, "- source: `{}`", args.path);
    let _ = writeln!(out, "- queries: {}", queries.len());
    let _ = writeln!(out, "- min subtree size: {} (`--min-size`)", args.min_size);
    let _ = writeln!(out, "- WL depth cap K: {} (`--k`)", args.k);
    let _ = writeln!(out, "- normalization on: {}", args.norm.describe());
}

fn subtree_section(out: &mut String, heading: &str, queries: &str, sh: &Shared) {
    let _ = writeln!(out, "### {heading}\n");
    if !queries.is_empty() {
        let _ = writeln!(out, "{queries}\n");
    }
    if !sh.is_sql {
        let _ = writeln!(
            out,
            "_Rewritten by normalization, so it maps to no single source fragment; \
             shown as an s-expression._\n"
        );
    }
    fenced(out, if sh.is_sql { "sql" } else { "text" }, &sh.sql);
}

/// Everything the CLI prints about the corpus (§9/§11), uncapped and with the SQL
/// untruncated -- the listings are capped by `--top` and cut to one line each.
fn corpus_report(
    queries: &[Query],
    dupes: &[Vec<usize>],
    pairs: &[PairRow],
    groups: &[Group],
    failures: usize,
    args: &Args,
) -> String {
    let mut out = String::new();
    report_header(&mut out, "Query similarity report", queries, args);
    let _ = writeln!(out, "- parse failures: {failures}");

    let _ = writeln!(out, "\n## Exact duplicates after normalization\n");
    if dupes.is_empty() {
        let _ = writeln!(out, "None: every query has a distinct root hash.\n");
    } else {
        let _ = writeln!(
            out,
            "Root-hash equality (§7). Same normalized shape, so the text below is one \
             member of each group.\n"
        );
        for g in dupes {
            let names: Vec<String> = g
                .iter()
                .map(|i| format!("q{i} (line {})", queries[*i].line))
                .collect();
            let _ = writeln!(out, "### {}\n", names.join(" = "));
            fenced(&mut out, "sql", &queries[g[0]].text);
        }
    }

    let _ = writeln!(out, "## Most similar pairs\n");
    let _ = writeln!(
        out,
        "Jaccard over the feature multiset, weighted by subtree size then by corpus IDF \
         (§8); cosine and both containments over the same vectors.\n"
    );
    let scored: Vec<&PairRow> = pairs.iter().filter(|p| p.jaccard > 0.0).collect();
    if scored.is_empty() {
        let _ = writeln!(out, "No two queries share a feature.\n");
    } else {
        let _ = writeln!(out, "| pair | jaccard | cosine | A in B | B in A |");
        let _ = writeln!(out, "|---|---:|---:|---:|---:|");
        for p in &scored {
            let _ = writeln!(
                out,
                "| q{} / q{} | {:.3} | {:.3} | {:.3} | {:.3} |",
                p.a, p.b, p.jaccard, p.cosine, p.cab, p.cba
            );
        }
        let zero = pairs.len() - scored.len();
        if zero > 0 {
            let _ = writeln!(out, "\n{zero} further pair(s) score 0 and are omitted.");
        }
        out.push('\n');
    }

    let _ = writeln!(out, "## Shared subexpressions\n");
    let _ = writeln!(
        out,
        "Every node grouped by full Merkle hash, kept at size >= {} in 2+ queries, \
         ranked by size x #queries. {} group(s).\n",
        args.min_size,
        groups.len()
    );
    if groups.is_empty() {
        let _ = writeln!(
            out,
            "No subtree of size >= {} occurs in two or more queries.\n",
            args.min_size
        );
    } else {
        let _ = writeln!(out, "| # | node | size | queries | occurrences |");
        let _ = writeln!(out, "|---|---|---:|---:|---:|");
        for (i, g) in groups.iter().enumerate() {
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} | {} |",
                i + 1,
                g.label,
                g.size,
                g.queries.len(),
                g.occurrences
            );
        }
        let _ = writeln!(
            out,
            "\nThe SQL below is one occurrence per group; the others matched on \
             normalized shape, not necessarily on text.\n"
        );
        for (i, g) in groups.iter().enumerate() {
            let names: Vec<String> = g
                .queries
                .iter()
                .map(|q| format!("q{q} (line {})", queries[*q].line))
                .collect();
            subtree_section(
                &mut out,
                &format!(
                    "{}. `{}` -- size {} in {} queries ({} occurrences)",
                    i + 1,
                    g.label,
                    g.size,
                    g.queries.len(),
                    g.occurrences
                ),
                &names.join(", "),
                &Shared {
                    size: g.size,
                    label: g.label.clone(),
                    sql: g.example.clone(),
                    is_sql: g.example_is_sql,
                },
            );
        }
    }

    query_appendix(&mut out, queries);
    out
}

/// The q-ids the rest of the report is written in terms of, so it stands alone.
fn query_appendix(out: &mut String, queries: &[Query]) {
    let _ = writeln!(out, "## Queries\n");
    for (i, q) in queries.iter().enumerate() {
        let _ = writeln!(out, "### q{i} -- line {}, {} nodes\n", q.line, q.tree.size);
        fenced(out, "sql", &q.text);
    }
}

/// `--explain A B --report PATH`: the same shared subtrees, untruncated.
fn pair_report(
    shared: &[Shared],
    queries: &[Query],
    vectors: &[HashMap<Key, f64>],
    a: usize,
    b: usize,
    args: &Args,
) -> String {
    let mut out = String::new();
    report_header(&mut out, &format!("Shared subtrees: q{a} vs q{b}"), queries, args);
    let _ = writeln!(
        out,
        "- jaccard: {:.3}, cosine: {:.3}",
        sim::jaccard(&vectors[a], &vectors[b]),
        sim::cosine(&vectors[a], &vectors[b])
    );
    let _ = writeln!(
        out,
        "- containment: q{a} in q{b} = {:.3}, q{b} in q{a} = {:.3}",
        sim::containment(&vectors[a], &vectors[b]),
        sim::containment(&vectors[b], &vectors[a])
    );
    let _ = writeln!(out, "- shared subtrees: {}\n", shared.len());

    let _ = writeln!(out, "## Queries\n");
    for i in [a, b] {
        let _ = writeln!(out, "### q{i} -- line {}, {} nodes\n", queries[i].line, queries[i].tree.size);
        fenced(&mut out, "sql", &queries[i].text);
    }

    let _ = writeln!(out, "## Shared subtrees\n");
    if shared.is_empty() {
        let _ = writeln!(
            out,
            "No subtree of size >= {} is shared by both queries.",
            args.min_size
        );
        return out;
    }
    for (i, sh) in shared.iter().enumerate() {
        subtree_section(
            &mut out,
            &format!("{}. `{}` -- size {}", i + 1, sh.label, sh.size),
            "",
            sh,
        );
    }
    out
}
