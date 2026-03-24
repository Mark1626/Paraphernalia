# Iceberg Hidden Partition Test Dataset

A standalone Rust program that generates a small Iceberg table with **hidden partitions** using [iceberg-rust](https://github.com/apache/iceberg-rust).

## What Are Hidden Partitions?

Unlike Hive-style partitioning where users must know and specify partition columns in queries, Iceberg's hidden partitions derive partition values automatically from source columns using **transforms**. Users query the original columns and the engine applies partition pruning transparently.

## Table Schema

| Column     | Type         | Nullable | Description          |
|------------|--------------|----------|----------------------|
| `event_id` | int          | No       | Unique event ID      |
| `event_ts` | timestamptz  | No       | Event timestamp      |
| `user_id`  | long         | No       | User identifier      |
| `city`     | string       | No       | City name            |
| `payload`  | string       | Yes      | Event payload        |

## Partition Spec

| Partition Field      | Source Column | Transform      | Description                         |
|----------------------|---------------|----------------|-------------------------------------|
| `event_id_identity`  | `event_id`    | `identity`     | Exact value of event_id             |
| `event_ts_day`       | `event_ts`    | `day`          | Extracts date from timestamp        |
| `user_id_bucket`     | `user_id`     | `bucket[4]`    | Murmur3 hash into 4 buckets         |
| `city_truncate`      | `city`        | `truncate[3]`  | First 3 characters of city          |

## Sample Data

| event_id | event_ts                | user_id | city          | payload  |
|----------|-------------------------|---------|---------------|----------|
| 1        | 2024-01-15 10:00:00 UTC | 100     | San Francisco | login    |
| 2        | 2024-01-15 14:30:00 UTC | 200     | New York      | purchase |
| 3        | 2024-01-16 09:00:00 UTC | 100     | San Francisco | logout   |
| 4        | 2024-01-16 18:00:00 UTC | 300     | Chicago       | (null)   |
| 5        | 2024-02-01 12:00:00 UTC | 200     | New York      | purchase |
| 6        | 2024-02-01 15:00:00 UTC | 400     | Seattle       | signup   |
| 7        | 2024-03-10 08:00:00 UTC | 100     | San Francisco | login    |
| 8        | 2024-03-10 20:00:00 UTC | 300     | Chicago       | purchase |

## Usage

```bash
# Build and generate the dataset
cargo run

# Output is written to ./warehouse/test_db/events/
```

Re-running will clean up and regenerate the dataset from scratch.

## Example SQL Queries

These queries demonstrate how hidden partitions enable transparent partition pruning.
Users write queries against the source columns -- the engine maps predicates to
partition transforms and skips irrelevant data files automatically.

### Day Transform Pruning (`event_ts` -> `event_ts_day`)

```sql
-- Filter by date: engine converts this to event_ts_day = '2024-01-15'
-- and only scans 2 of 8 data files (event_id 1 and 2)
SELECT * FROM events
WHERE event_ts >= TIMESTAMP '2024-01-15 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-01-16 00:00:00 UTC';

-- Filter by month: engine prunes to event_ts_day values in January
-- scanning 4 of 8 files (event_id 1-4)
SELECT event_id, city, payload FROM events
WHERE event_ts >= TIMESTAMP '2024-01-01 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-02-01 00:00:00 UTC';

-- Aggregate per day: partition pruning reduces scan for date range
SELECT CAST(event_ts AS DATE) AS event_date, COUNT(*) AS event_count
FROM events
WHERE event_ts >= TIMESTAMP '2024-01-15 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-01-17 00:00:00 UTC'
GROUP BY CAST(event_ts AS DATE);
-- Returns: 2024-01-15 -> 2, 2024-01-16 -> 2
```

### Truncate Transform Pruning (`city` -> `city_truncate`)

```sql
-- Filter by city prefix: engine maps city = 'San Francisco'
-- to city_truncate = 'San', scanning only 3 of 8 files
SELECT event_id, event_ts, payload FROM events
WHERE city = 'San Francisco';

-- LIKE with matching prefix also benefits from truncate pruning
-- city_truncate = 'New' narrows scan to 2 files
SELECT * FROM events
WHERE city LIKE 'New%';

-- Multiple cities: engine prunes to city_truncate IN ('San', 'Chi')
-- scanning 5 of 8 files instead of all 8
SELECT city, COUNT(*) as cnt FROM events
WHERE city IN ('San Francisco', 'Chicago')
GROUP BY city;
-- Returns: San Francisco -> 3, Chicago -> 2
```

### Bucket Transform Pruning (`user_id` -> `user_id_bucket`)

```sql
-- Filter by user_id: engine computes bucket hash and reads only
-- files in that bucket. user_id=100 hashes to bucket 0.
SELECT event_id, event_ts, city FROM events
WHERE user_id = 100;
-- Returns events 1, 3, 7 (all San Francisco)

-- Equality on user_id enables exact bucket pruning
-- user_id=200 hashes to bucket 3
SELECT payload, COUNT(*) FROM events
WHERE user_id = 200
GROUP BY payload;
-- Returns: purchase -> 2

-- Note: range predicates on user_id do NOT benefit from bucket pruning
-- because hash(x) has no ordering relationship with x.
SELECT * FROM events WHERE user_id > 200;
-- This scans ALL buckets (no pruning possible)
```

### Identity Transform Pruning (`event_id` -> `event_id_identity`)

```sql
-- Exact match: scans only 1 data file
SELECT * FROM events WHERE event_id = 5;

-- IN list: scans only the matching files (2 of 8)
SELECT event_id, city, payload FROM events
WHERE event_id IN (1, 8);

-- Range: identity transform supports range pruning
-- scans files for event_id 3, 4, 5 (3 of 8 files)
SELECT * FROM events
WHERE event_id BETWEEN 3 AND 5;
```

### Combined Partition Pruning (multiple transforms)

```sql
-- Day + City: engine applies BOTH transforms to minimize scan
-- event_ts_day = '2024-01-15' AND city_truncate = 'San'
-- narrows to just 1 file (event_id = 1)
SELECT * FROM events
WHERE event_ts >= TIMESTAMP '2024-01-15 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-01-16 00:00:00 UTC'
  AND city = 'San Francisco';

-- Day + Bucket: prunes on both day and user_id hash
-- event_ts_day = '2024-03-10' AND user_id_bucket = bucket(100)
SELECT * FROM events
WHERE event_ts >= TIMESTAMP '2024-03-10 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-03-11 00:00:00 UTC'
  AND user_id = 100;
-- Returns only event 7

-- All four transforms combined: maximum pruning
-- Targets exactly 1 file out of 8
SELECT payload FROM events
WHERE event_id = 2
  AND event_ts >= TIMESTAMP '2024-01-15 00:00:00 UTC'
  AND event_ts <  TIMESTAMP '2024-01-16 00:00:00 UTC'
  AND user_id = 200
  AND city = 'New York';
-- Returns: purchase
```

### Queries That Do NOT Benefit From Hidden Partitions

```sql
-- Full table scan: no partition predicates at all
SELECT COUNT(*) FROM events;

-- Predicate on non-partitioned column: no pruning possible
SELECT * FROM events WHERE payload = 'login';

-- Range on bucket column: bucket transform only prunes on equality
SELECT * FROM events WHERE user_id > 100 AND user_id < 400;

-- Function on partition source that doesn't match the transform:
-- HOUR(event_ts) doesn't align with the DAY transform
SELECT * FROM events WHERE EXTRACT(HOUR FROM event_ts) = 10;
```

## Output Structure

```
warehouse/test_db/events/
  metadata/
    00000-<uuid>.metadata.json   # Initial table creation
    00001-<uuid>.metadata.json   # After data commit
    snap-<id>-<uuid>.avro        # Manifest list
    <uuid>-m0.avro               # Manifest file
  data/
    event_id_identity=1/event_ts_day=2024-01-15/user_id_bucket=0/city_truncate=San/
      data-00000.parquet
    event_id_identity=2/event_ts_day=2024-01-15/user_id_bucket=3/city_truncate=New/
      data-00001.parquet
    ...
```
