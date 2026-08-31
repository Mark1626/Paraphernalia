//! Generic tree node (spec §4.3) + Merkle (§6) and Weisfeiler-Lehman (§7) hashing.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(FNV_OFFSET)
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= *b as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
    fn write_u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }
}

/// §6: delimit the label and encode arity, concatenate children in (already
/// canonical) order. Never XOR -- that cancels duplicate children.
pub fn hash_node(label: &str, kids: &[u64]) -> u64 {
    let mut h = Fnv::new();
    h.write(label.as_bytes());
    h.write(&[0xff]);
    h.write_u64(kids.len() as u64);
    for k in kids {
        h.write_u64(*k);
    }
    h.0
}

/// Labels whose children are a set, not a sequence.
fn is_commutative(label: &str) -> bool {
    matches!(
        label,
        "and" | "or" | "projection" | "from" | "op:eq" | "op:noteq" | "op:plus" | "op:multiply"
    ) || label.starts_with("setop:union")
}

/// §5: canonical comparison direction. `b < a` -> `a > b`, orientation chosen
/// by operand hash so it is deterministic.
fn flipped(label: &str) -> Option<&'static str> {
    match label {
        "op:lt" => Some("op:gt"),
        "op:gt" => Some("op:lt"),
        "op:lteq" => Some("op:gteq"),
        "op:gteq" => Some("op:lteq"),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub label: String,
    pub children: Vec<Node>,
    /// Original SQL text of the fragment this node came from, when there was one.
    pub sql: Option<String>,
    /// Full Merkle hash (k = infinity).
    pub hash: u64,
    /// Node count of this subtree, inclusive.
    pub size: u32,
    /// wl[k] = hash of this subtree truncated to depth k.
    pub wl: Vec<u64>,
    /// wl_size[k] = node count of that truncated subtree (feature weight).
    pub wl_size: Vec<u32>,
}

impl Node {
    pub fn new(label: impl Into<String>, children: Vec<Node>) -> Node {
        Node {
            label: label.into(),
            children,
            sql: None,
            hash: 0,
            size: 0,
            wl: Vec::new(),
            wl_size: Vec::new(),
        }
    }

    pub fn leaf(label: impl Into<String>) -> Node {
        Node::new(label, Vec::new())
    }

    pub fn with_sql(mut self, sql: impl Into<String>) -> Node {
        self.sql = Some(sql.into());
        self
    }

    /// §6. Bottom-up, imposing canonical order on commutative nodes as it goes.
    pub fn merkle(&mut self, commute: bool) -> u64 {
        let mut kids: Vec<u64> = self.children.iter_mut().map(|c| c.merkle(commute)).collect();
        if commute && kids.len() > 1 {
            if is_commutative(&self.label) {
                let mut pairs: Vec<(u64, Node)> =
                    kids.iter().cloned().zip(self.children.drain(..)).collect();
                pairs.sort_by_key(|(h, _)| *h);
                kids = pairs.iter().map(|(h, _)| *h).collect();
                self.children = pairs.into_iter().map(|(_, c)| c).collect();
            } else if kids.len() == 2 && kids[0] > kids[1] {
                if let Some(f) = flipped(&self.label) {
                    self.children.swap(0, 1);
                    kids.swap(0, 1);
                    self.label = f.to_string();
                }
            }
        }
        self.size = 1 + self.children.iter().map(|c| c.size).sum::<u32>();
        self.hash = hash_node(&self.label, &kids);
        self.hash
    }

    /// §7. h_0 = label, h_k = hash(label || h_{k-1}(children)). Run after `merkle`
    /// so children are already in canonical order.
    pub fn wl_hashes(&mut self, kmax: usize) {
        for c in self.children.iter_mut() {
            c.wl_hashes(kmax);
        }
        self.wl = Vec::with_capacity(kmax + 1);
        self.wl_size = Vec::with_capacity(kmax + 1);
        self.wl.push(hash_node(&self.label, &[]));
        self.wl_size.push(1);
        for k in 1..=kmax {
            let kids: Vec<u64> = self.children.iter().map(|c| c.wl[k - 1]).collect();
            self.wl.push(hash_node(&self.label, &kids));
            self.wl_size
                .push(1 + self.children.iter().map(|c| c.wl_size[k - 1]).sum::<u32>());
        }
    }

    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a Node)) {
        f(self);
        for c in &self.children {
            c.walk(f);
        }
    }

    /// Fallback rendering for nodes with no original SQL text (§9 wants SQL;
    /// rewritten nodes such as flattened AND/OR no longer map to one fragment).
    pub fn sexpr(&self) -> String {
        if self.children.is_empty() {
            self.label.clone()
        } else {
            let kids: Vec<String> = self.children.iter().map(|c| c.sexpr()).collect();
            format!("({} {})", self.label, kids.join(" "))
        }
    }

    /// Original SQL text with its own line breaks intact -- the CLI collapses it
    /// with `snippet`, the report (§9) wants it whole.
    pub fn render(&self) -> String {
        match &self.sql {
            Some(s) => s.trim().to_string(),
            None => self.sexpr(),
        }
    }

    /// Whether `render` returns real SQL or the s-expression fallback.
    pub fn has_sql(&self) -> bool {
        self.sql.is_some()
    }
}
