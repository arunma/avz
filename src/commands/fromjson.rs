use apache_avro::{Codec, Schema, Writer};
use std::fs;
use std::io::{self, BufRead, BufReader};

use crate::convert::json_to_avro;
use crate::error::{AvzError, Result};

pub async fn execute(
    schema_path: &str,
    output_path: &str,
    codec_name: &str,
    input_path: Option<&str>,
) -> Result<()> {
    let schema_str = fs::read_to_string(schema_path)
        .map_err(|e| AvzError::User(format!("Cannot read schema file {}: {}", schema_path, e)))?;
    let schema = Schema::parse_str(&schema_str)?;
    let codec = parse_codec(codec_name)?;

    let output_file = fs::File::create(output_path)
        .map_err(|e| AvzError::User(format!("Cannot create output file {}: {}", output_path, e)))?;
    let mut writer = Writer::with_codec(&schema, output_file, codec);

    let reader: Box<dyn BufRead> = match input_path {
        Some(path) => {
            let f = fs::File::open(path)
                .map_err(|e| AvzError::User(format!("Cannot open input file {}: {}", path, e)))?;
            Box::new(BufReader::new(f))
        }
        None => Box::new(BufReader::new(io::stdin())),
    };

    let mut line_num = 0;
    for line in reader.lines() {
        line_num += 1;
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let json: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|e| AvzError::User(format!("Invalid JSON at line {}: {}", line_num, e)))?;
        let avro_value = json_to_avro(&json, &schema)
            .map_err(|e| AvzError::User(format!("Conversion error at line {}: {}", line_num, e)))?;
        writer.append(avro_value)?;
    }

    writer.flush()?;
    eprintln!("Wrote {} records to {}", line_num, output_path);
    Ok(())
}

pub fn parse_codec(name: &str) -> Result<Codec> {
    match name.to_lowercase().as_str() {
        "null" => Ok(Codec::Null),
        "deflate" => Ok(Codec::Deflate),
        "snappy" => Ok(Codec::Snappy),
        "zstandard" | "zstd" => Ok(Codec::Zstandard),
        "bzip2" | "bzip" => Ok(Codec::Bzip2),
        "xz" => Ok(Codec::Xz),
        _ => Err(AvzError::User(format!(
            "Unknown codec: {}. Supported: null, deflate, snappy, zstandard, bzip2, xz",
            name
        ))),
    }
}
