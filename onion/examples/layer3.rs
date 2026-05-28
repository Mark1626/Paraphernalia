use std::error::Error;
use std::fs;

const KEY_LEN: usize = 32;

// The puzzle's layer-header format is known plaintext (this is structure, not the
// hidden content). It pins key bytes 0..15 and seeds enough correct text for the
// word heuristic below to get traction.
const KNOWN_HEADER: &[u8] = b"==[ Layer 4/6: ";

// Top of the Zipf distribution for English. The correct key turns the description
// into real prose, so the decryption containing the most of these words wins.
const COMMON_WORDS: [&str; 10] = [
    "the", "of", "and", "to", "in", "is", "that", "it", "as", "for",
];

// All the prose lives at the start of the payload; the ASCII85 block after it has
// no real words, so scoring a prefix is both faster and less noisy.
const SCORE_LEN: usize = 4096;

/// Per-character English likelihood. Reward space and the most common letters,
/// penalize non-printable bytes. This is the per-column signal that tells `e` from
/// `f` (key ^ 0x03) and lowercase from UPPERCASE (key ^ 0x20), so each column can
/// be pinned on its own instead of deadlocking a whole shifted block.
fn char_score(b: u8) -> i32 {
    match b {
        b' ' => 8, // space: by far the most common character in English
        b'e' | b't' | b'a' | b'o' | b'i' | b'n' | b's' | b'h' | b'r' => 4, // top letters
        b'a'..=b'z' => 2,                          // other lowercase
        b'A'..=b'Z' => 1,                          // uppercase: rare in prose
        b'\n' | b'\t' | 0x20..=0x7e => 1,          // other printable
        _ => -8,                                   // control / >=0x80 => wrong key
    }
}

/// How English a candidate decryption reads: per-character frequency score, then
/// improved with a bonus per common whole word (case-sensitive, since the word list
/// is lowercase). Words straddle columns, so this scores the full text, not a column.
fn english_score(data: &[u8], key: &[u8; KEY_LEN]) -> i32 {
    let n = data.len().min(SCORE_LEN);
    let text: Vec<u8> = (0..n).map(|i| data[i] ^ key[i % KEY_LEN]).collect();

    let mut s: i32 = text.iter().map(|&b| char_score(b)).sum();
    s += 20 * text
        .split(|&b| !b.is_ascii_alphabetic())
        .filter(|w| COMMON_WORDS.iter().any(|cw| *w == cw.as_bytes()))
        .count() as i32;
    s
}

fn find_key(data: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];

    // Seed the columns the header covers; these are exact.
    for i in 0..KEY_LEN {
        if let Some(&plain) = KNOWN_HEADER.get(i) {
            key[i] = data[i] ^ plain;
        }
    }

    // Coordinate ascent over the unknown columns: set each column to the byte that
    // maximizes the whole-text word score. A column is pinned by words that overlap
    // already-correct columns, so we sweep both directions each round -- otherwise a
    // word-free block (e.g. the title) can only be unwound from one side and stalls.
    let start = KNOWN_HEADER.len();
    let forward: Vec<usize> = (start..KEY_LEN).collect();
    let backward: Vec<usize> = (start..KEY_LEN).rev().collect();
    for _ in 0..8 {
        let mut changed = false;
        for &i in forward.iter().chain(backward.iter()) {
            let best = (0u8..=255)
                .max_by_key(|&b| {
                    let mut trial = key;
                    trial[i] = b;
                    english_score(data, &trial)
                })
                .unwrap();
            if best != key[i] {
                key[i] = best;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    key
}

fn xor_decode(data: &[u8], key: &[u8; KEY_LEN]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % KEY_LEN])
        .collect()
}

fn decode_layer3(data: Vec<u8>) -> Vec<u8> {
    let key = find_key(&data);
    println!("Key {key:02x?}");
    xor_decode(&data, &key)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("payload/layer3.data")?;
    let ascii85_decoded = ascii85::decode(&input)?;

    // This could also be done by mutating the decoded ascii85 array
    let decoded = decode_layer3(ascii85_decoded);

    fs::write("layer3_out.data", &decoded)?;

    Ok(())
}
