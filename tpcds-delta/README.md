# Build

```
cargo build --release
```

# Generate SF=1 (1 GB) into ./tpcds\_delta

```
cargo run --release -- --scale-factor 1 --output ./tpcds_delta
```

# Generate SF=10 (10 GB)

```
cargo run --release -- -s 10 -o ./tpcds_delta_10
```

