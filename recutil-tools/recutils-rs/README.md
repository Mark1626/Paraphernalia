# recutils-rs

Rust FFI bindings to [GNU recutils](https://www.gnu.org/software/recutils/) (`librec`).

The crate ships two layers:

- A raw `unsafe` FFI under `recutils_rs::ffi`, generated at build time by
  `bindgen` from `<rec.h>` — covers the full `rec_*` / `REC_*` / `MSET_*`
  surface.
- A small safe wrapper at the crate root (`Db`, `Rset`, `Record`,
  `SelectionExpression`, …)
  that the bundled examples use. It only covers what those examples need; for
  anything else, drop down to `recutils_rs::ffi::*`.

## Status

Alpha. The raw FFI is complete; the safe layer is intentionally minimal —
parsing, record / field construction, record-set append, the writer, and the
selection-expression engine. Reach into `ffi` for everything else.

## Prerequisites

`librec` must be installed and findable at build time.

- **macOS (Homebrew):** `brew install recutils`. The build script discovers it
  automatically via `brew --prefix recutils`.
- **Debian / Ubuntu:** `apt install recutils libgnurec-dev` (header lands on
  the default include path).
- **Other:** install upstream and point the build at it (see below).

`bindgen` requires `libclang`. On macOS it ships with Xcode Command Line
Tools; on Linux install `libclang-dev` (or your distro's equivalent).

## Build

```sh
cargo build
cargo test
cargo build --examples
```

If `librec` lives somewhere non-standard, point the build at it with one of:

| Variable                  | Meaning                                       |
|---------------------------|-----------------------------------------------|
| `RECUTILS_PREFIX`         | Install prefix; expects `include/` and `lib/` |
| `RECUTILS_INCLUDE_DIR`    | Directory containing `rec.h`                  |
| `RECUTILS_LIB_DIR`        | Directory containing `librec.{so,dylib}`      |

Search order: `RECUTILS_PREFIX` → individual `_INCLUDE_DIR` / `_LIB_DIR` →
`brew --prefix recutils` → compiler defaults.

## Usage

```rust
use recutils_rs::{Db, Record, SelectionExpression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Db::parse_str("%rec: Book\n\nTitle: Refactoring\nAuthor: Martin Fowler\n")?;

    // Query with a recutils selection expression.
    let selection_expression =
        SelectionExpression::compile("Author = 'Martin Fowler'", false)?;
    let rset = db.rset_by_type("Book").unwrap();
    let hits = rset
        .records()
        .filter(|r| selection_expression.matches(r))
        .count();
    println!("matches: {hits}");

    // Build and append a new record, then re-serialize.
    let mut rec = Record::new();
    rec.append_field("Title", "Test-Driven Development")?;
    rec.append_field("Author", "Kent Beck")?;
    db.rset_by_type("Book").unwrap().append_record(rec)?;
    let updated = db.to_rec_string()?;
    println!("{updated}");
    Ok(())
}
```

`librec` is initialized lazily via `std::sync::Once` on first use of any safe
type — there is no manual init/fini to track. Drop handles cleanup for `Db`,
`Record` (when not transferred), `SelectionExpression`, and the iterators.

Reach into `recutils_rs::ffi` for the raw C API when the safe layer doesn't
yet cover what you need.

## Examples

Each example takes its inputs from CLI arguments. They share a small
`/tmp/sample.rec` between runs in the snippets below.

```sh
cat > /tmp/sample.rec <<'EOF'
%rec: Book

Title: Refactoring
Author: Martin Fowler

Title: Domain-Driven Design
Author: Eric Evans

%rec: Movie

Title: Heat
Year: 1995
EOF
```

### `count_records` — total records and record sets

```sh
cargo run --example count_records -- /tmp/sample.rec
# /tmp/sample.rec: 3 records across 2 record set(s)
```

### `append_record` — parse, append, write back

Parses the file, finds the named record set, builds a new record from
`Field=Value` pairs, appends it, serializes the whole DB with
`rec_writer_new_str`, and writes the result back to the same path.

```sh
cargo run --example append_record -- \
    /tmp/sample.rec Book \
    "Title=Test-Driven Development" "Author=Kent Beck"
# appended 1 record to rset "Book" in /tmp/sample.rec
```

### `modify_records` — alter or delete records (≈ `recset`)

Loads the file, picks records matching a selection expression, and either
sets a field on each match or deletes the matches outright. Re-serializes
and writes the file in place.

```sh
# update a field on every matching record
cargo run --example modify_records -- \
    /tmp/sample.rec Book "Author = 'Eric Evans'" set "Author=Evans, Eric"
# set Author=Evans, Eric on 1 record(s); 0 matching record(s) had no such field

# delete every matching record
cargo run --example modify_records -- \
    /tmp/sample.rec Movie "Year < 2000" delete
# deleted 1 record(s)
```

### `query_records` — selection expressions via `rec_sex_*`

Compiles the third argument as a recutils selection expression (the same
syntax `recsel -e` accepts) and prints every matching record from the named
record set.

```sh
cargo run --example query_records -- \
    /tmp/sample.rec Book "Author = 'Kent Beck'"
# Title: Test-Driven Development
# Author: Kent Beck
#
# 1 match(es)
```

## License

GPL-3.0-or-later, matching upstream recutils.
