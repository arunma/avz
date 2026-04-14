use apache_avro::Reader;
use aws_sdk_s3::Client as S3Client;

use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(files: &[String], s3_client: &Option<S3Client>) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;

    for path in &paths {
        if multi {
            eprintln!("==> {} <==", path);
        }
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => process(Reader::new(f), path)?,
            AvroInput::Memory(c) => process(Reader::new(c), path)?,
        }
    }
    Ok(())
}

fn process(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
) -> Result<()> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;

    let canonical = reader.writer_schema().canonical_form();
    let parsed: serde_json::Value = serde_json::from_str(&canonical)?;
    println!("{}", colored_json::to_colored_json_auto(&parsed)?);
    Ok(())
}
