//! Feature bags, corpus IDF and the similarity metric (spec §8).

use std::collections::HashMap;

use crate::node::Node;

/// (depth cap, hash). `FULL` is the k = infinity layer (the plain Merkle hash).
pub type Key = (u8, u64);
pub const FULL: u8 = 255;

/// Multiset of features for one query: key -> occurrence count.
pub type Bag = HashMap<Key, u32>;

pub struct Corpus {
    /// key -> subtree size, the per-occurrence weight (§8: weight by subtree size)
    pub weights: HashMap<Key, u32>,
    /// key -> number of queries containing it
    pub df: HashMap<Key, u32>,
    pub n: usize,
}

pub fn features(root: &Node, kmax: usize, weights: &mut HashMap<Key, u32>) -> Bag {
    let mut bag = Bag::new();
    root.walk(&mut |n: &Node| {
        for k in 0..=kmax {
            let key = (k as u8, n.wl[k]);
            *bag.entry(key).or_insert(0) += 1;
            weights.insert(key, n.wl_size[k]);
        }
        let key = (FULL, n.hash);
        *bag.entry(key).or_insert(0) += 1;
        weights.insert(key, n.size);
    });
    bag
}

impl Corpus {
    pub fn build(bags: &[Bag], weights: HashMap<Key, u32>) -> Corpus {
        let mut df: HashMap<Key, u32> = HashMap::new();
        for b in bags {
            for key in b.keys() {
                *df.entry(*key).or_insert(0) += 1;
            }
        }
        Corpus {
            weights,
            df,
            n: bags.len(),
        }
    }

    pub fn idf(&self, key: &Key) -> f64 {
        let df = *self.df.get(key).unwrap_or(&0) as f64;
        ((1.0 + self.n as f64) / (1.0 + df)).ln() + 1.0
    }

    /// TF-IDF vector, with TF weighted by subtree size.
    pub fn vector(&self, bag: &Bag) -> HashMap<Key, f64> {
        bag.iter()
            .map(|(k, c)| {
                let w = *self.weights.get(k).unwrap_or(&1) as f64;
                (*k, *c as f64 * w * self.idf(k))
            })
            .collect()
    }
}

pub fn cosine(a: &HashMap<Key, f64>, b: &HashMap<Key, f64>) -> f64 {
    let (small, large) = if a.len() < b.len() { (a, b) } else { (b, a) };
    let mut dot = 0.0;
    for (k, v) in small {
        if let Some(w) = large.get(k) {
            dot += v * w;
        }
    }
    let na: f64 = a.values().map(|v| v * v).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|v| v * v).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// §8: |A n B| / |A| -- "is A essentially a fragment of B".
pub fn containment(a: &HashMap<Key, f64>, b: &HashMap<Key, f64>) -> f64 {
    let mut inter = 0.0;
    let mut total = 0.0;
    for (k, v) in a {
        total += v;
        if let Some(w) = b.get(k) {
            inter += v.min(*w);
        }
    }
    if total == 0.0 {
        0.0
    } else {
        inter / total
    }
}

/// §8 baseline: Jaccard, with both refinements applied (size weighting + IDF).
/// Reads as "what fraction of these two queries is structurally shared".
pub fn jaccard(a: &HashMap<Key, f64>, b: &HashMap<Key, f64>) -> f64 {
    let mut inter = 0.0;
    let mut union = 0.0;
    for (k, v) in a {
        match b.get(k) {
            Some(w) => {
                inter += v.min(*w);
                union += v.max(*w);
            }
            None => union += v,
        }
    }
    for (k, v) in b {
        if !a.contains_key(k) {
            union += v;
        }
    }
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}
