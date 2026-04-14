use apache_avro::{Reader, Writer};
use aws_sdk_s3::Client as S3Client;
use std::fs;

use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    output_path: &str,
) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    if paths.is_empty() {
        return Err(AvzError::User("No input files".into()));
    }

    // Read schema from first file
    let first_input = io::open_input(&paths[0], s3_client).await?;
    let (first_schema, first_records) = match first_input {
        AvroInput::Local(f) => read_all(Reader::new(f), &paths[0])?,
        AvroInput::Memory(c) => read_all(Reader::new(c), &paths[0])?,
    };

    let output_file = fs::File::create(output_path)
        .map_err(|e| AvzError::User(format!("Cannot create output file {}: {}", output_path, e)))?;
    let mut writer = Writer::new(&first_schema, output_file);

    let mut total = 0;
    for val in first_records {
        writer.append(val)?;
        total += 1;
    }

    for path in &paths[1..] {
        let input = io::open_input(path, s3_client).await?;
        let (schema, records) = match input {
            AvroInput::Local(f) => read_all(Reader::new(f), path)?,
            AvroInput::Memory(c) => read_all(Reader::new(c), path)?,
        };

        if schema.canonical_form() != first_schema.canonical_form() {
            return Err(AvzError::User(format!(
                "Schema mismatch: {} has a different schema than {}",
                path, paths[0]
            )));
        }

        for val in records {
            writer.append(val)?;
            total += 1;
        }
    }

    writer.flush()?;
    eprintln!("Concatenated {} records from {} files into {}", total, paths.len(), output_path);
    Ok(())
}

fn read_all(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
) -> Result<(apache_avro::Schema, Vec<apache_avro::types::Value>)> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;
    let schema = reader.writer_schema().clone();
    let records: std::result::Result<Vec<_>, _> = reader.collect();
    let records = records?;
    Ok((schema, records))
}
