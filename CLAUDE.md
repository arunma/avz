# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is avz?

`avz` is a fast Avro CLI tool written in Rust — a replacement for Java's avro-tools and Python's fastavro. It supports local files, glob patterns, and S3 URIs.

## Build & Test Commands

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo test                   # run all unit tests
cargo test <test_name>       # run a single test by name
cargo run -- <subcommand>    # run the CLI (e.g., cargo run -- cat file.avro)
```

No integration tests exist yet (no `tests/` directory); all tests are inline `#[cfg(test)]` modules.

## Architecture

**Binary entry point:** `src/main.rs` — parses CLI args via clap, lazily initializes an S3 client only when S3 URIs are detected, then dispatches to the matching command module.

**Modules:**

- `src/cli.rs` — Clap-derived CLI definition. `Commands` enum defines all subcommands. `FileArgs` is a shared struct for file path arguments (reused across most commands via `#[command(flatten)]`).
- `src/commands/` — One module per subcommand (`cat`, `head`, `schema`, `count`, `meta`, `fromjson`, `concat`, `recodec`, `fingerprint`, `validate`, `random`). Each exports an `execute()` async function.
- `src/io/` — I/O abstraction layer:
  - `resolver.rs` — Expands glob patterns and S3 prefix listings into concrete file paths.
  - `input.rs` — `AvroInput` enum (local `File` or in-memory `Cursor<Vec<u8>>` for S3) plus raw Avro header parsing (magic bytes, metadata map, sync marker).
  - `s3.rs` — S3 URI parsing, object download, and paginated listing with glob matching.
- `src/convert.rs` — Bidirectional Avro↔JSON value conversion (`avro_to_json`, `json_to_avro`). Handles all Avro types including unions, records, logical types.
- `src/error.rs` — `AvzError` enum (via thiserror) and `Result<T>` type alias.

**Key design patterns:**
- Every command takes `&Option<S3Client>` — S3 is only initialized when needed.
- File resolution (glob + S3 listing) happens through `io::resolve_files()` before command logic runs.
- The `apache-avro` crate (with snappy/zstd/bzip2/xz features) handles all Avro serialization/deserialization. The custom header parser in `input.rs` is used only by the `meta` command for raw metadata access.
