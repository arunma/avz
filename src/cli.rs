use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(
    name = "avz",
    about = "Blistering-fast Avro CLI tool",
    long_about = "A modern, high-performance replacement for Java's avro-tools and Python's fastavro.\nSupports local files, glob patterns, and S3 URIs.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Clone)]
pub struct FileArgs {
    /// File path(s), glob pattern, or s3:// URI
    #[arg(required = true)]
    pub files: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print all records as JSON
    Cat {
        #[command(flatten)]
        files: FileArgs,

        /// Pretty-print JSON output
        #[arg(short, long)]
        pretty: bool,

        /// Print only the first N records
        #[arg(short = 'n', long)]
        head: Option<usize>,
    },

    /// Print the first N records (default 10)
    Head {
        #[command(flatten)]
        files: FileArgs,

        /// Number of records to show
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,
    },

    /// Print the Avro schema as JSON
    Schema {
        #[command(flatten)]
        files: FileArgs,
    },

    /// Count records in Avro files
    Count {
        #[command(flatten)]
        files: FileArgs,
    },

    /// Print file metadata (codec, sync marker, user metadata)
    Meta {
        #[command(flatten)]
        files: FileArgs,
    },

    /// Convert JSON records to an Avro file
    #[command(name = "fromjson")]
    FromJson {
        /// Path to the Avro schema JSON file
        #[arg(short, long, required = true)]
        schema: String,

        /// Output Avro file path
        #[arg(short, long, required = true)]
        output: String,

        /// Compression codec: null, deflate, snappy, zstandard, bzip2, xz
        #[arg(short, long, default_value = "null")]
        codec: String,

        /// Input JSON file (reads from stdin if omitted)
        input: Option<String>,
    },

    /// Concatenate Avro files into one
    Concat {
        #[command(flatten)]
        files: FileArgs,

        /// Output Avro file path
        #[arg(short, long, required = true)]
        output: String,
    },

    /// Re-encode an Avro file with a different codec
    Recodec {
        #[command(flatten)]
        files: FileArgs,

        /// Target compression codec: null, deflate, snappy, zstandard, bzip2, xz
        #[arg(short, long, required = true)]
        codec: String,

        /// Output Avro file path
        #[arg(short, long, required = true)]
        output: String,
    },

    /// Print schema fingerprint (CRC-64-AVRO, MD5, SHA-256)
    Fingerprint {
        #[command(flatten)]
        files: FileArgs,

        /// Algorithm: rabin, md5, sha256, or all
        #[arg(short, long, default_value = "all")]
        algorithm: String,
    },

    /// Validate an Avro file or check schema compatibility
    Validate {
        #[command(flatten)]
        files: FileArgs,

        /// Optional reader schema for compatibility check
        #[arg(long)]
        reader_schema: Option<String>,
    },

    /// Search records for a pattern, printing matching records as JSON
    Grep {
        /// Regex pattern to search for (use -F for literal string)
        pattern: String,

        #[command(flatten)]
        files: FileArgs,

        /// Pretty-print JSON output
        #[arg(short, long)]
        pretty: bool,

        /// Case-insensitive matching
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Invert match — show records that do NOT match
        #[arg(short = 'v', long)]
        invert: bool,

        /// Show only the count of matching records
        #[arg(short, long)]
        count: bool,

        /// Treat pattern as a fixed string, not a regex
        #[arg(short = 'F', long)]
        fixed_string: bool,
    },

    /// Generate random records from a schema
    Random {
        /// Path to the Avro schema JSON file
        #[arg(short, long, required = true)]
        schema: String,

        /// Number of records to generate
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,

        /// Output format: json or avro
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path (stdout if omitted, required for avro format)
        #[arg(short, long)]
        output: Option<String>,

        /// Random seed for reproducible output
        #[arg(long)]
        seed: Option<u64>,

        /// Pretty-print JSON output
        #[arg(short, long)]
        pretty: bool,
    },
}
