# recutil-tools

A set of tools around [GNU recutils](https://www.gnu.org/software/recutils/).

## Crates

### [`recutils-rs`](./recutils-rs)

Rust FFI bindings to `librec`, plus a small safe wrapper. Designed to be
published independently to crates.io.

- Raw bindings under `recutils_rs::ffi` (generated via `bindgen`)
- Safe types at the crate root: `Db`, `Rset`, `Record`, `RecordRef`,
  `Fields`, `FieldRef`, `Sex`, `Error`
- Lazy `rec_init` via `std::sync::Once`
- Optional `arrow` feature: exposes `recutils_rs::arrow::rec_to_record_batch`
  for rec → Apache Arrow `RecordBatch` conversion. Off by default.

### [`rec2parquet`](./rec2parquet)

CLI binary that converts `.rec` files to Apache Parquet, modelled on
[`csv2parquet`](https://github.com/domoritz/arrow-tools/tree/main/crates/csv2parquet).

```bash
rec2parquet <INPUT> <OUTPUT> -t <TYPE> [-c <COMPRESSION>] [--max-row-group-size N] [-p] [-n]
```

Honors `%type:` declarations from the rec descriptor; untyped fields fall
back to `Utf8`. Repeated field names within a single record are errors
(future `--list-repeated` mode TBD).

### [`recsql`](./recsql)

Library + CLI binary that exposes a `.rec` file as a SQL-queryable table
via [Apache DataFusion](https://datafusion.apache.org/). The library
provides `RecTableProvider` (a custom DataFusion `TableProvider`); the
binary wires it up to a `SessionContext` and prints results.

```bash
recsql <INPUT> -t <TYPE> -q '<SQL>'
```

```bash
recsql books.rec -t Book -q 'SELECT "Title", "Year" FROM Book WHERE "Year" >= 2000'
```

The rec file is parsed eagerly when the provider is opened; subsequent
queries hit the cached `RecordBatch`. Note that SQL identifiers are
case-folded by default (ANSI SQL behavior) — quote field names that use
mixed case (e.g. `"Year"`) or the table name (e.g. `"Book"`) to preserve
casing.

Filter pushdown to recutils' Sex engine is planned but not yet wired up;
today all filtering happens above the provider in DataFusion.

## Build

```bash
cargo build                                    # build all crates
cargo build --workspace --all-targets          # include examples + bins
cargo test                                     # run recutils-rs unit tests
cargo run -p rec2parquet -- --type Book in.rec out.parquet
cargo run -p recsql -- in.rec -t Book -q 'SELECT * FROM Book LIMIT 5'
cargo run -p recutils-rs --example query_records -- file.rec Book "Year > 2000"
```

## librec discovery

`recutils-rs/build.rs` searches for librec in this order:

1. `RECUTILS_PREFIX` (expects `include/` and `lib/` underneath)
2. `RECUTILS_INCLUDE_DIR` and/or `RECUTILS_LIB_DIR`
3. `brew --prefix recutils` (macOS Homebrew)
4. Compiler defaults

`bindgen` requires `libclang` (Xcode CLT on macOS, `libclang-dev` on Linux).
On Linux: `apt install recutils libgnurec-dev`.

## License

`GPL-3.0-or-later` — matches upstream recutils.
