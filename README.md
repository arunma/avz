# avz

A blistering-fast Avro CLI tool — a modern replacement for Java's `avro-tools` and Python's `fastavro`.

Supports **local files**, **glob patterns**, and **S3 URIs**.

## Install

### Homebrew (macOS)

```bash
brew tap arunma/tap
brew install avz
```

### Cargo

```bash
cargo install avz
```

### Debian / Ubuntu

Download the `.deb` from [Releases](https://github.com/arunma/avz/releases):

```bash
sudo dpkg -i avz-x86_64-unknown-linux-gnu.deb
```

### From source

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

```
Usage: avz <COMMAND>

Commands:
  cat          Print all records as JSON
  head         Print the first N records (default 10)
  schema       Print the Avro schema as JSON
  count        Count records in Avro files
  meta         Print file metadata (codec, sync marker, user metadata)
  fromjson     Convert JSON records to an Avro file
  concat       Concatenate Avro files into one
  recodec      Re-encode an Avro file with a different codec
  fingerprint  Print schema fingerprint (CRC-64-AVRO, MD5, SHA-256)
  validate     Validate an Avro file or check schema compatibility
  grep         Search records for a pattern, printing matching records as JSON
  random       Generate random records from a schema
```

---

## Command Reference

All examples below use this sample dataset — 8 employee records with fields for id, name, department (enum), salary, email (nullable), tags (array), and active (boolean).

### `cat` — Print records as JSON

Print all records, one JSON object per line:

```bash
$ avz cat employees.avro
```

```json
{"id":1,"name":"Alice Chen","department":"ENGINEERING","salary":125000.0,"email":"alice@example.com","tags":["rust","backend","senior"],"active":true}
{"id":2,"name":"Bob Smith","department":"SALES","salary":95000.0,"email":"bob@example.com","tags":["enterprise","closer"],"active":true}
{"id":3,"name":"Carol Davis","department":"ENGINEERING","salary":140000.0,"email":"carol@example.com","tags":["rust","systems","principal"],"active":true}
...
```

With `--pretty` for colorized, indented output:

```bash
$ avz cat --pretty employees.avro
```

```json
{
  "id": 1,
  "name": "Alice Chen",
  "department": "ENGINEERING",
  "salary": 125000.0,
  "email": "alice@example.com",
  "tags": [
    "rust",
    "backend",
    "senior"
  ],
  "active": true
}
```

Limit output with `-n`:

```bash
$ avz cat -n 2 employees.avro
```

---

### `head` — Print first N records

```bash
$ avz head -n 3 employees.avro
```

```json
{"id":1,"name":"Alice Chen","department":"ENGINEERING","salary":125000.0,"email":"alice@example.com","tags":["rust","backend","senior"],"active":true}
{"id":2,"name":"Bob Smith","department":"SALES","salary":95000.0,"email":"bob@example.com","tags":["enterprise","closer"],"active":true}
{"id":3,"name":"Carol Davis","department":"ENGINEERING","salary":140000.0,"email":"carol@example.com","tags":["rust","systems","principal"],"active":true}
```

With colorized output:

```bash
$ avz head -n 2 --pretty employees.avro
```

```json
{
  "id": 1,
  "name": "Alice Chen",
  "department": "ENGINEERING",
  "salary": 125000.0,
  "email": "alice@example.com",
  "tags": [
    "rust",
    "backend",
    "senior"
  ],
  "active": true
}
{
  "id": 2,
  "name": "Bob Smith",
  "department": "SALES",
  "salary": 95000.0,
  "email": "bob@example.com",
  "tags": [
    "enterprise",
    "closer"
  ],
  "active": true
}
```

Default is 10 records when `-n` is omitted.

---

### `schema` — Print the Avro schema

Outputs colorized JSON with automatic pager for large schemas:

```bash
$ avz schema employees.avro
```

```json
{
  "name": "com.example.hr.Employee",
  "type": "record",
  "fields": [
    {
      "name": "id",
      "type": "int"
    },
    {
      "name": "name",
      "type": "string"
    },
    {
      "name": "department",
      "type": {
        "name": "com.example.hr.Department",
        "type": "enum",
        "symbols": [
          "ENGINEERING",
          "SALES",
          "MARKETING",
          "HR",
          "FINANCE"
        ]
      }
    },
    {
      "name": "salary",
      "type": "double"
    },
    {
      "name": "email",
      "type": [
        "null",
        "string"
      ]
    },
    {
      "name": "tags",
      "type": {
        "type": "array",
        "items": "string"
      }
    },
    {
      "name": "active",
      "type": "boolean"
    }
  ]
}
```

Large schemas automatically pipe through `less -R` in interactive terminals.

---

### `count` — Count records

Single file:

```bash
$ avz count employees.avro
8
```

Multiple files show per-file counts and a total:

```bash
$ avz count employees.avro employees2.avro
/tmp/avz-docs/employees.avro: 8
/tmp/avz-docs/employees2.avro: 8
total: 16
```

Works with globs:

```bash
$ avz count "data/*.avro"
```

---

### `meta` — File metadata

Shows the raw schema, codec, sync marker, and any user-defined metadata:

```bash
$ avz meta employees.avro
```

```
avro.schema	{ ... }
avro.codec	null
sync	0be4e3b6562329dbba6c5f06aa43ee96
```

---

### `fingerprint` — Schema fingerprint

Print all fingerprints:

```bash
$ avz fingerprint employees.avro
CRC-64-AVRO	146d06fde15d172f
MD5	874856ac6f65f6eeced12661790a5ec2
SHA-256	3c9dd71e34662cb613aac0d4bdb7afa7309f2712ff97c1991a29028fccd607df
```

Or a specific algorithm:

```bash
$ avz fingerprint --algorithm sha256 employees.avro
3c9dd71e34662cb613aac0d4bdb7afa7309f2712ff97c1991a29028fccd607df
```

Supported: `rabin` (CRC-64-AVRO), `md5`, `sha256`, `all` (default).

---

### `validate` — Validate files and schema compatibility

Validate file integrity (reads every record):

```bash
$ avz validate employees.avro
Validated 8 records in employees.avro
employees.avro: OK
```

Check schema compatibility:

```bash
$ avz validate employees.avro --reader-schema new_schema.json
employees.avro: COMPATIBLE
```

---

### `grep` — Search records

Searches the JSON representation of each record and prints the **entire matching record**:

```bash
$ avz grep "ENGINEERING" employees.avro
```

```json
{"id":1,"name":"Alice Chen","department":"ENGINEERING","salary":125000.0,"email":"alice@example.com","tags":["rust","backend","senior"],"active":true}
{"id":3,"name":"Carol Davis","department":"ENGINEERING","salary":140000.0,"email":"carol@example.com","tags":["rust","systems","principal"],"active":true}
{"id":7,"name":"Grace Kim","department":"ENGINEERING","salary":135000.0,"email":"grace@example.com","tags":["frontend","react","senior"],"active":true}
```

Pretty-print matches:

```bash
$ avz grep --pretty "rust" employees.avro
```

```json
{
  "id": 1,
  "name": "Alice Chen",
  "department": "ENGINEERING",
  "salary": 125000.0,
  "email": "alice@example.com",
  "tags": [
    "rust",
    "backend",
    "senior"
  ],
  "active": true
}
...
```

Case-insensitive:

```bash
$ avz grep -i "alice" employees.avro
{"id":1,"name":"Alice Chen","department":"ENGINEERING","salary":125000.0,...}
```

Fixed string (no regex — useful when pattern has special chars like `*`, `.`, `(`):

```bash
$ avz grep -F "125000.0" employees.avro
{"id":1,"name":"Alice Chen","department":"ENGINEERING","salary":125000.0,...}
```

Count matches:

```bash
$ avz grep -c "ENGINEERING" employees.avro
3
```

Invert match (show non-matching records):

```bash
$ avz grep -v -c "ENGINEERING" employees.avro
5
```

| Flag | Description |
|------|-------------|
| `-i` | Case-insensitive matching |
| `-v` | Invert match (show records that do NOT match) |
| `-c` | Show only the count of matching records |
| `-F` | Treat pattern as a fixed string, not a regex |
| `--pretty` | Colorized pretty-print output |

---

### `fromjson` — Convert JSON to Avro

Convert newline-delimited JSON to an Avro file:

```bash
$ avz fromjson --schema schema.json --output employees.avro employees.jsonl
Wrote 8 records to employees.avro
```

```
Usage: avz fromjson [OPTIONS] --schema <SCHEMA> --output <OUTPUT> [INPUT]

Options:
  -s, --schema <SCHEMA>  Path to the Avro schema JSON file
  -o, --output <OUTPUT>  Output Avro file path
  -c, --codec <CODEC>    Compression codec [default: null]
  [INPUT]                Input JSON file (reads from stdin if omitted)
```

With compression:

```bash
$ avz fromjson --schema schema.json --output data.avro --codec snappy input.jsonl
```

From stdin:

```bash
$ cat records.jsonl | avz fromjson --schema schema.json --output data.avro
```

---

### `concat` — Concatenate Avro files

Merge multiple files into one:

```bash
$ avz concat employees.avro employees2.avro --output merged.avro
Concatenated 16 records from 2 files into merged.avro
```

---

### `recodec` — Re-encode with a different codec

Change the compression codec of an existing Avro file:

```bash
$ avz recodec employees.avro --codec zstandard --output employees-zstd.avro
Re-encoded 8 records with codec 'zstandard' to employees-zstd.avro
```

Verify the codec changed:

```bash
$ avz meta employees-zstd.avro | grep codec
avro.codec	zstandard
```

---

### `random` — Generate random test data

Generate random records from a schema:

```bash
$ avz random --schema schema.json -n 3 --seed 42
```

```json
{"id":-734,"name":"lambda eta","department":"ENGINEERING","salary":-170.08,"email":"lambda","tags":["iota","mu xi lambda"],"active":false}
{"id":-680,"name":"lambda iota","department":"ENGINEERING","salary":-297.97,"email":null,"tags":["lambda omicron","lambda eta"],"active":false}
{"id":199,"name":"theta","department":"ENGINEERING","salary":-691.60,"email":"kappa eta eta","tags":["lambda","pi theta","lambda","zeta"],"active":true}
```

Pretty-print:

```bash
$ avz random --schema schema.json -n 2 --seed 42 --pretty
```

```json
{
  "id": -734,
  "name": "lambda eta",
  "department": "ENGINEERING",
  "salary": -170.08,
  "email": "lambda",
  "tags": [
    "iota",
    "mu xi lambda"
  ],
  "active": false
}
```

Write directly to Avro format:

```bash
$ avz random --schema schema.json -n 1000 --format avro --output test.avro
```

| Flag | Description |
|------|-------------|
| `-s, --schema` | Path to Avro schema JSON file (required) |
| `-n, --count` | Number of records to generate (default: 10) |
| `--seed` | Random seed for reproducible output |
| `-f, --format` | Output format: `json` (default) or `avro` |
| `-o, --output` | Output file path (required for avro format) |
| `--pretty` | Colorized pretty-print (json format only) |

---

## S3 Support

All read commands work with S3 URIs. AWS credentials are loaded from the standard chain (env vars, `~/.aws/credentials`, IAM role, etc.).

```bash
# single file
avz head -n 5 "s3://my-bucket/data/events.avro"

# glob pattern on S3
avz count "s3://my-bucket/data/dt=2026-03-16/*.avro"

# grep across S3 files
avz grep -F "transaction_id" "s3://my-bucket/data/*.avro"
```

> S3 files are downloaded into memory. For very large individual files, consider downloading first with `aws s3 cp`.

## Supported Codecs

| Codec | Flag value |
|-------|-----------|
| None | `null` |
| Deflate | `deflate` |
| Snappy | `snappy` |
| Zstandard | `zstandard` or `zstd` |
| Bzip2 | `bzip2` or `bzip` |
| XZ | `xz` |

## License

MIT OR Apache-2.0
