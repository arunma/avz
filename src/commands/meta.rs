use aws_sdk_s3::Client as S3Client;

use crate::error::Result;
use crate::io::{self, AvroInput, read_avro_header};

pub async fn execute(files: &[String], s3_client: &Option<S3Client>) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;

    for path in &paths {
        if multi {
            eprintln!("==> {} <==", path);
        }
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => process(f, path)?,
            AvroInput::Memory(c) => process(c, path)?,
        }
    }
    Ok(())
}

fn process(mut reader: impl std::io::Read, path: &str) -> Result<()> {
    let header = read_avro_header(&mut reader).map_err(|e| {
        crate::error::AvzError::User(format!("Failed to read header from {}: {}", path, e))
    })?;

    // Print schema (pretty-printed if valid JSON)
    if let Some(schema_json) = header.schema_json() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(schema_json) {
            println!("avro.schema\t{}", serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| schema_json.to_string()));
        } else {
            println!("avro.schema\t{}", schema_json);
        }
    }

    // Print codec
    println!("avro.codec\t{}", header.codec());

    // Print sync marker
    println!("sync\t{}", hex::encode(header.sync_marker));

    // Print any user metadata (non-avro.* keys)
    let mut user_keys: Vec<&String> = header
        .metadata
        .keys()
        .filter(|k| !k.starts_with("avro."))
        .collect();
    user_keys.sort();
    for key in user_keys {
        if let Some(val) = header.metadata.get(key) {
            match std::str::from_utf8(val) {
                Ok(s) => println!("{}\t{}", key, s),
                Err(_) => println!("{}\t{}", key, hex::encode(val)),
            }
        }
    }

    Ok(())
}
