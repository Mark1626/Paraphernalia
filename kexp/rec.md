# `rec.k` — a GNU recutils parser in K

This document explains `rec.k` line by line. The parser reads a
[GNU recutils](https://www.gnu.org/software/recutils/) file (a flat,
human-readable record database) and returns a **list of dictionaries**, one
dictionary per record.

---

## The recfile format in 60 seconds

A recfile is plain text:

```
a: 1
b: "abc"

a: 2
b: "def"
```

- A **field** is `Name: value` — name, a colon, then the value.
- A **record** is a run of consecutive field lines.
- Records are separated by a **blank line**.
- A field value can span multiple lines using **continuation lines** that
  start with `+`. The `+` and one optional following space are stripped and
  the rest is appended to the previous value, joined by a newline:

  ```
  Name: John
  + Smith
  ```
  → `Name` = `"John\nSmith"`.

The goal of `rec.k` is to turn that text into:

```
((,"a";,"b")!(,"1";"\"abc\"")     / record 0 as a dict
 (,"a";,"b")!(,"2";"\"def\""))    / record 1 as a dict
```

---

## A note on reading K

K evaluates **right to left** with no operator precedence — every verb takes
everything to its right as its argument. Parentheses override that. Most
symbols are *overloaded*: one glyph means one thing with one argument
(**monadic**) and another with two (**dyadic**). The cheat-sheet for the
symbols used in this file:

| Glyph | Monadic (1 arg) | Dyadic (2 args) |
|-------|-----------------|-----------------|
| `*`   | first element   | multiply |
| `#`   | count / length  | take / reshape |
| `!`   | enumerate `0..n-1` | **make dictionary** `keys!values` |
| `+`   | transpose (flip)| add |
| `_`   | floor           | **drop** / **cut** |
| `&`   | **where** (indices of 1s) | min / and |
| `<`   | grade-up (sort indices) | less-than |
| `=`   | group           | equals |
| `?`   | unique          | **find** (first index of) |
| `\|`  | reverse         | max / or |
| `~`   | not             | match |

Adverbs (modify a verb):

- `'`  **each** — apply to each element.
- `/`  **over** (fold) — `seed f/ list` reduces left-to-right.
- `\`  **scan** — like over, but keeps every intermediate result.
- `:`  forces the **monadic** form of the verb it follows (e.g. `#:` is
  always "count", never "take").

---

## Line by line

### `\d rec`  (line 1)

`\d` sets the current **directory** (namespace). Everything defined below lives
under `rec.`, so the entry point is called as `rec.recorig`. Line 17 (`\d .`)
switches back to the root namespace.

---

### Constants (lines 3–6)

```k
ESC:  "\""    / the double-quote character "
FLD:  ":"     / field separator (name : value)
ROW:  "\n"    / record/line separator (newline)
CONT: "+"     / continuation-line marker
```

`x: y` is assignment. Each value is a one-character string. `ESC` is used by
`sp` to ignore separators that sit **inside** quoted text.

---

### `tl` — trim leading spaces (line 8)

```k
tl:{(+/&\x=" ")_x}
```

`{ ... }` is a lambda; its implicit first argument is `x`. Read right-to-left:

1. `x=" "` — compare each char of `x` to a space → boolean vector
   (`1` where space).
2. `&\` — **and-scan** (running minimum). It stays `1` from the left until the
   first `0`, then is `0` forever. So it marks the run of *leading* spaces.
   - `"  ab"="  "` → `1 1 0 0`, and `&\1 1 0 0` → `1 1 0 0`.
3. `+/` — **over with `+`** = sum. Counts the leading spaces.
4. `n _ x` — dyadic `_` is **drop**: drop the first `n` characters.

So `tl "   John"` → `"John"`. Used to strip the space after the colon in
`name: value`.

---

### `kv` — split one line into (key; value) (line 9)

```k
kv:{i:x?FLD; (i#x; tl (1+i)_x)}
```

- `i:x?FLD` — dyadic `?` is **find**: the index of the first `:` in the line.
  (If there is no `:`, find returns `#x`, i.e. "past the end".)
- `i#x` — dyadic `#` is **take**: the first `i` characters = the **key**
  (everything before the colon).
- `(1+i)_x` — **drop** `i+1` characters: skip the key *and* the colon, leaving
  the raw value.
- `tl ...` — strip the leading space from that value.
- `( key ; value )` — return the two as a 2-element list.

`kv "Name: John"` → `("Name"; "John")`.

Splitting on the **first** colon (via `?`) matters: values like
`URL: http://x` keep their internal colons.

---

### `ct` — strip a continuation marker (line 10)

```k
ct:{r:1_x; (" "=*r)_r}
```

Given a continuation line like `"+ Smith"`:

1. `r:1_x` — drop the leading `+` → `" Smith"`.
2. `*r` — **first** char of the remainder.
3. `" "=*r` — is it a space? → `1` or `0`.
4. `(...)_r` — **drop** that many (1 or 0) chars, i.e. drop one optional space.

`ct "+ Smith"` → `"Smith"`; `ct "+bar"` → `"bar"`. Exactly the recutils rule:
strip `+` and one optional blank.

---

### `mg` — merge continuation lines (line 11)

```k
mg:{$[CONT=*y; (-1_x),,(*|x),ROW,ct y; x,,y]}
```

This is the function folded over a record's lines. As a dyadic lambda, `x` is
the **accumulator** (the list of merged lines built so far) and `y` is the
**next line**.

`$[cond; then; else]` is `if/else`.

- **Condition** `CONT=*y` — does the next line start with `+`?
- **Then** (it's a continuation): `(-1_x),,(*|x),ROW,ct y`
  - `*|x` — `|x` reverses, `*` takes first → the **last** accumulated line.
  - `(*|x),ROW,ct y` — glue: last line + `"\n"` + stripped continuation.
  - `-1_x` — **drop** the last element from the accumulator.
  - `(-1_x),,Z` — `,Z` enlists `Z` (wraps it as a 1-element list), and the
    outer `,` concatenates. Net effect: replace the last line with the merged
    line.
- **Else** (a normal line): `x,,y` — append `y` to the accumulator as a new
  element.

Folded with seed `()` over `("Name: John"; "+ Smith")`:

```
() -> ("Name: John")                     after the normal line
   -> ("Name: John\nSmith")              after merging "+ Smith"
```

The embedded `\n` later ends up inside the *value*, because `kv` only splits on
the first `:`.

---

### `sp` — quote-aware split (line 13)

```k
sp:{[s;x](1&!#l)_'(l:0,&(2!+\x=ESC)<x=s)_x}
```

`{[s;x] ...}` declares an explicit two-argument lambda: `s` = separator char,
`x` = the string to split. This is the engine used for **both** splitting the
file into lines (separator `\n`) and could split a line into fields (separator
`:`) — though here field splitting is done by `kv`.

Work from the inside out.

**1. Find the real separators.**

```k
2!+\x=ESC
```
- `x=ESC` — `1` at every `"`.
- `+\` — **plus-scan** = running total of quotes seen so far.
- `2!` — dyadic `!` is **mod**: `mod 2` of that running count → the *parity*.
  It is `1` while we are **inside** an open quote, `0` while outside.

```k
(2!+\x=ESC) < x=s
```
- `x=s` — `1` where the char is the separator.
- dyadic `<` — "left is less than right", i.e. `1` only where parity is `0`
  **and** the char is a separator. So this is the mask of separators that are
  **not** inside quotes — the real split points.

**2. Turn the mask into cut indices.**

```k
l: 0, &(...)
```
- `&` — monadic **where**: the indices where the mask is `1`.
- `0,` — prepend `0` so the first chunk starts at the beginning of the string.

`l` is now the list of cut positions, e.g. for `"a,b,c"` split on `,`,
`l = 0 1 3`.

**3. Cut the string.**

```k
l _ x
```
Dyadic `_` with an **integer-vector** left argument is **cut**: it slices `x`
into pieces that begin at each index in `l`. Because the cut points land *on*
the separators, every piece after the first begins with the separator
character:

```
"a,b,c" cut at 0 1 3  ->  ("a"; ",b"; ",c")
```

**4. Drop the leading separator from each piece.**

```k
(1&!#l) _' (...)
```
- `#l` — count of cut points = number of pieces.
- `!#l` — `0 1 2 … (#l-1)`.
- `1&` — dyadic `&` is **min**: `min(1, each)` → `0 1 1 1 …`. Zero for the
  first piece, one for all the rest.
- `_'` — **drop-each**: drop `0` chars from the first piece (it has no leading
  separator) and `1` char (the separator) from every other piece.

```
("a"; ",b"; ",c")  ->  ("a"; "b"; "c")
```

**Why `1&!#l` and not the simpler `1&l`?** `1&l` derives the drop-counts from
the cut *indices*. That works only if no separator sits at position `0`. If the
string *starts* with a separator (e.g. a leading blank line → leading `\n`),
the first real separator's index is also `0`, it collides with the prepended
`0`, and `1&l` would compute "drop 0" for that piece — leaking a stray
separator character through as a phantom token. `1&!#l` derives the drop-counts
from each piece's **position** instead (always `0,1,1,1,…`), so it is correct
regardless of where separators fall. For inputs with no leading separator the
two are identical.

---

### `recorig` — the parser (line 15)

```k
recorig:{r:sp[ROW;x]; c:{x@&0<#'x}'(0,&0=#'r)_r; {(!).+x}'kv''{()mg/x}'c@&0<#'c}
```

`x` is the whole file as a string. Three statements separated by `;`; the last
is the return value. Left to right:

**1. Split into lines.**

```k
r: sp[ROW; x]
```
`sp` with separator `\n` → `r` is the list of lines. A blank line becomes an
empty string `""`. A trailing newline produces a trailing `""`.

**2. Group lines into records, dropping blank lines.**

```k
c: {x@&0<#'x}' (0, &0=#'r) _ r
```
- `#'r` — `#'` is **count-each**: the length of every line.
- `0=#'r` — `1` for every **blank** line (length 0).
- `&0=#'r` — **where**: the indices of the blank lines.
- `(0, ...)` — prepend `0` so the first record is included.
- `... _ r` — **cut** the list of lines at those indices → one chunk per
  record. Each chunk after the first begins with the blank line that preceded
  it.
- `{x@&0<#'x}'` — for **each** chunk, keep only the non-empty lines:
  - `0<#'x` — `1` for lines with length `> 0`.
  - `&` — their indices.
  - `x@...` — `@` is **index/select**: keep those lines, dropping the blank.

After this, `c` is a list of records, each a list of raw field lines. (Empty
chunks — from a trailing newline or doubled blank lines — survive here as `()`
and are filtered next.)

**3. Merge continuations, split fields, build dicts.**

The final expression is three `each`-mapped stages, again read right-to-left:

```k
{(!).+x}' kv'' {()mg/x}' c@&0<#'c
```

- `c@&0<#'c` — drop **empty records**: `0<#'c` is `1` for chunks with at least
  one line; `&` their indices; `c@...` selects them. This removes the phantom
  empty record a trailing newline would otherwise create.
- `{()mg/x}'` — for **each** record, fold `mg` over its lines with seed `()`,
  collapsing `+` continuation lines into the line above (see `mg`).
- `kv''` — `''` is **each-each**: apply `kv` to every line of every record,
  turning each `"name: value"` into a `(key; value)` pair.
- `{(!).+x}'` — for **each** record, turn its list of pairs into a dictionary:
  - `+x` — **transpose** the list of pairs `((k1;v1);(k2;v2);…)` into
    `((k1;k2;…); (v1;v2;…))` = `(keys; values)`.
  - `(!).` — `.` is **apply**: apply dyadic `!` (make-dictionary) to that
    2-element argument list → `keys!values`.

The result is a **list of dictionaries**, one per record.

---

### `\d .`  (line 17)

Return to the root namespace. Definitions above remain reachable as `rec.tl`,
`rec.sp`, `rec.recorig`, etc.

---

## Using it

```k
\l rec.k                 / load the file
x: 1:"i/test.rec"        / read the file as a string (1: = read bytes)
d: rec.recorig x         / parse -> list of dicts
```

**Field lookup gotcha.** The keys are strings (character vectors), so a bare
`d[0]"b"` makes K try to index by each character and errors with `'rank`.
Enlist the key:

```k
d[0]@,"b"        / -> "\"abc\""
d[0]@,"Name"     / -> "John\nSmith"
```

Indexing a dict with a *list* of keys returns a list of values, so a one-key
lookup returns a one-element list; prefix `*` if you want the bare value:

```k
*d[0]@,"b"       / -> the value itself
```

**Uniform records display as a table.** When every record has the same keys
(as in `test.rec`), ngn/k stores the list of identical-keyed dicts as a *table*
(`@d` is `` `M ``) and prints it transposed as `+(keys)!(columns)`. This is
still a list of dictionaries — `d[i]` returns the i-th row as a dict
(`@d[i]` is `` `m ``). When records have differing keys, `d` is a plain list of
dicts.

---

## Worked example

Input `i/test.rec`:

```
a: 1
b: "abc"

a: 2
b: "def"

a: 3
b: "ghi"
```

Pipeline:

1. `sp[ROW;x]` →
   `("a: 1";"b: \"abc\"";"";"a: 2";"b: \"def\"";"";"a: 3";"b: \"ghi\"";"")`
   (note the trailing `""` from the final newline).
2. blank indices `&0=#'r` → `2 5 8`; cut at `0 2 5 8`; drop blank lines per
   chunk → four chunks, the last one empty.
3. `c@&0<#'c` drops the empty chunk → three records.
4. `{()mg/x}'` — no `+` lines here, so records are unchanged.
5. `kv''` →
   `((("a";"1");("b";"\"abc\"")) (("a";"2");("b";"\"def\"")) (("a";"3");("b";"\"ghi\"")))`.
6. `{(!).+x}'` → three dicts:
   `(,"a";,"b")!(,"1";"\"abc\"")`, … etc.
