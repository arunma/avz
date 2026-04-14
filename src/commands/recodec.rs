use apache_avro::{Reader, Writer};
use aws_sdk_s3::Client as S3Client;
use std::fs;

use crate::commands::fromjson::parse_codec;
use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    codec_name: &str,
    output_path: &str,
) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    if paths.len() != 1 {
        return Err(AvzError::User("recodec expects exactly one input file".into()));
    }

    let path = &paths[0];
    let codec = parse_codec(codec_name)?;

    let input = io::open_input(path, s3_client).await?;
    let (schema, records) = match input {
        AvroInput::Local(f) => read_all(Reader::new(f), path)?,
        AvroInput::Memory(c) => read_all(Reader::new(c), path)?,
    };

    let output_file = fs::File::create(output_path)
        .map_err(|e| AvzError::User(format!("Cannot create output file {}: {}", output_path, e)))?;
    let mut writer = Writer::with_codec(&schema, output_file, codec);

    let count = records.len();
    for val in records {
        writer.append(val)?;
    }
    writer.flush()?;

    eprintln!("Re-encoded {} records with codec '{}' to {}", count, codec_name, output_path);
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
