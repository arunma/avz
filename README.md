# avz

A blistering-fast Avro CLI tool — a modern replacement for Java's `avro-tools` and Python's `fastavro`.

Supports **local files**, **glob patterns**, and **S3 URIs**.

## Install

```bash
cargo install avz
```

Or build from source:

```bash
git clone https://github.com/arunma/avz.git
cd avz
cargo build --release
# binary at target/release/avz
```

## Quick Start

```bash
# peek at the first 5 records
avz head -n 5 data.avro

# pretty-print with syntax highlighting
avz cat --pretty data.avro

# search for a record by regex
avz grep "user_id.*12345" data.avro

# search for a literal string (no regex)
avz grep -F "300376641*2967" data.avro

# count records across files using a glob
avz count "logs/*.avro"

# works with S3 too
avz cat "s3://my-bucket/events/dt=2026-03-16/*.avro" --pretty
```

> **Note:** Quote glob patterns and S3 URIs to prevent your shell from expanding them.

## Commands

| Command       | Description                                              |
|---------------|----------------------------------------------------------|
| `cat`         | Print all records as JSON (`--pretty` for color output)  |
| `head`        | Print the first N records (default 10)                   |
| `schema`      | Print the Avro schema (colorized)                        |
| `count`       | Count records in one or more files                       |
| `meta`        | Print file metadata (codec, sync marker, user metadata)  |
| `grep`        | Search records by regex or literal string (`-F`)         |
| `fromjson`    | Convert newline-delimited JSON to an Avro file           |
| `concat`      | Concatenate multiple Avro files into one                 |
| `recodec`     | Re-encode with a different codec                         |
| `fingerprint` | Print schema fingerprint (CRC-64-AVRO, MD5, SHA-256)    |
| `validate`    | Validate file integrity or check schema compatibility    |
| `random`      | Generate random records from a schema                    |

## Usage

### Reading files

```bash
# single file
avz cat data.avro

# glob pattern
avz count "events/*.avro"

# multiple files
avz concat a.avro b.avro c.avro --output merged.avro

# S3 URI (uses default AWS credentials)
avz schema "s3://bucket/prefix/file.avro"

# S3 glob
avz count "s3://bucket/events/dt=2026-03-16/*.avro"
```

### Grep

Search through records and print the entire matching record as JSON:

```bash
# regex search
avz grep "error.*timeout" events.avro

# literal string (-F), useful when pattern has special chars
avz grep -F "amount=100.00" transactions.avro

# case-insensitive
avz grep -i "failed" events.avro

# invert match (show non-matching records)
avz grep -v "SUCCESS" events.avro

# count matches only
avz grep -c "PARTY" events.avro

# pretty-print matches
avz grep --pretty "entity_id" events.avro
```

### Writing and converting

```bash
# JSON to Avro
avz fromjson --schema schema.json --output data.avro input.json

# from stdin
cat records.jsonl | avz fromjson --schema schema.json --output data.avro

# with compression
avz fromjson --schema schema.json --output data.avro --codec snappy input.json

# re-encode existing file with a different codec
avz recodec data.avro --codec zstandard --output data-zstd.avro
```

### Inspection

```bash
# schema
avz schema data.avro

# metadata (codec, sync marker)
avz meta data.avro

# schema fingerprints
avz fingerprint data.avro
avz fingerprint --algorithm sha256 data.avro

# validate file integrity
avz validate data.avro

# check schema compatibility
avz validate data.avro --reader-schema new_schema.json
```

### Generate test data

```bash
# random JSON records from a schema
avz random --schema schema.json -n 100

# reproducible output with a seed
avz random --schema schema.json -n 10 --seed 42 --pretty

# write directly to Avro
avz random --schema schema.json -n 1000 --format avro --output test.avro
```

## Supported Codecs

`null`, `deflate`, `snappy`, `zstandard`, `bzip2`, `xz`

## License

MIT OR Apache-2.0
