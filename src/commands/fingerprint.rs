use apache_avro::Reader;
use apache_avro::rabin::Rabin;
use aws_sdk_s3::Client as S3Client;
use md5::Md5;
use sha2::Sha256;

use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    algorithm: &str,
) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;

    for path in &paths {
        if multi {
            eprintln!("==> {} <==", path);
        }
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => process(Reader::new(f), path, algorithm)?,
            AvroInput::Memory(c) => process(Reader::new(c), path, algorithm)?,
        }
    }
    Ok(())
}

fn process(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
    algorithm: &str,
) -> Result<()> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;
    let schema = reader.writer_schema();

    match algorithm.to_lowercase().as_str() {
        "all" => {
            let rabin = schema.fingerprint::<Rabin>();
            let md5 = schema.fingerprint::<Md5>();
            let sha256 = schema.fingerprint::<Sha256>();
            println!("CRC-64-AVRO\t{}", hex::encode(&rabin.bytes));
            println!("MD5\t{}", hex::encode(&md5.bytes));
            println!("SHA-256\t{}", hex::encode(&sha256.bytes));
        }
        "rabin" | "crc-64-avro" | "crc64" => {
            let fp = schema.fingerprint::<Rabin>();
            println!("{}", hex::encode(&fp.bytes));
        }
        "md5" => {
            let fp = schema.fingerprint::<Md5>();
            println!("{}", hex::encode(&fp.bytes));
        }
        "sha256" | "sha-256" => {
            let fp = schema.fingerprint::<Sha256>();
            println!("{}", hex::encode(&fp.bytes));
        }
        _ => {
            return Err(AvzError::User(format!(
                "Unknown algorithm: {}. Supported: rabin, md5, sha256, all",
                algorithm
            )));
        }
    }
    Ok(())
}
