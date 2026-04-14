use apache_avro::{Reader, Schema, schema_compatibility::SchemaCompatibility};
use aws_sdk_s3::Client as S3Client;
use std::fs;

use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    reader_schema_path: Option<&str>,
) -> Result<()> {
    if let Some(rs_path) = reader_schema_path {
        return check_compatibility(files, s3_client, rs_path).await;
    }

    let paths = io::resolve_files(files, s3_client).await;
    for path in &paths {
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => validate_file(Reader::new(f), path)?,
            AvroInput::Memory(c) => validate_file(Reader::new(c), path)?,
        }
        println!("{}: OK", path);
    }
    Ok(())
}

fn validate_file(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
) -> Result<()> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;

    let mut count = 0;
    for record in reader {
        record.map_err(|e| {
            AvzError::User(format!("Invalid record #{} in {}: {}", count + 1, path, e))
        })?;
        count += 1;
    }
    eprintln!("Validated {} records in {}", count, path);
    Ok(())
}

async fn check_compatibility(
    files: &[String],
    s3_client: &Option<S3Client>,
    reader_schema_path: &str,
) -> Result<()> {
    let schema_str = fs::read_to_string(reader_schema_path)
        .map_err(|e| AvzError::User(format!("Cannot read schema file {}: {}", reader_schema_path, e)))?;
    let reader_schema = Schema::parse_str(&schema_str)?;

    let paths = io::resolve_files(files, s3_client).await;
    for path in &paths {
        let input = io::open_input(path, s3_client).await?;
        let writer_schema = match input {
            AvroInput::Local(f) => get_schema(Reader::new(f), path)?,
            AvroInput::Memory(c) => get_schema(Reader::new(c), path)?,
        };

        match SchemaCompatibility::can_read(&writer_schema, &reader_schema) {
            Ok(()) => println!("{}: COMPATIBLE", path),
            Err(e) => {
                println!("{}: INCOMPATIBLE", path);
                return Err(AvzError::User(format!(
                    "Reader schema is not compatible with writer schema in {}: {}",
                    path, e
                )));
            }
        }
    }
    Ok(())
}

fn get_schema(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
) -> Result<Schema> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;
    Ok(reader.writer_schema().clone())
}
