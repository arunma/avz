//! # avz
//!
//! A blistering-fast Avro CLI tool — a modern replacement for Java's `avro-tools` and Python's `fastavro`.
//!
//! ## Features
//!
//! - **Read & inspect**: `cat`, `head`, `schema`, `count`, `meta`, `fingerprint`, `validate`
//! - **Search**: `grep` with regex or fixed-string matching across entire Avro records
//! - **Transform**: `fromjson`, `concat`, `recodec`
//! - **Generate**: `random` test data from any Avro schema
//! - **I/O sources**: local files, glob patterns, and S3 URIs (with glob support)
//! - **Codecs**: null, deflate, snappy, zstandard, bzip2, xz
//! - **Output**: colorized JSON with automatic pager for large output
//!
//! ## Installation
//!
//! ```bash
//! cargo install avz
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # peek at records
//! avz head -n 5 data.avro --pretty
//!
//! # search for a record
//! avz grep "user_id.*12345" data.avro
//!
//! # count records across S3
//! avz count "s3://bucket/prefix/*.avro"
//!
//! # convert JSON to Avro
//! avz fromjson --schema schema.json --output data.avro input.jsonl
//! ```
//!
//! See the [README](https://github.com/arunma/avz) for full command reference.

pub mod cli;
pub mod commands;
pub mod convert;
pub mod error;
pub mod io;
