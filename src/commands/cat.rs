use apache_avro::Reader;
use aws_sdk_s3::Client as S3Client;

use crate::convert::avro_to_json;
use crate::error::Result;
use crate::io::{self, AvroInput};

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    pretty: bool,
    head: Option<usize>,
) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;

    for path in &paths {
        if multi {
            eprintln!("==> {} <==", path);
        }
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => process(Reader::new(f), path, pretty, head)?,
            AvroInput::Memory(c) => process(Reader::new(c), path, pretty, head)?,
        }
    }
    Ok(())
}

fn process(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
    pretty: bool,
    head: Option<usize>,
) -> Result<()> {
    let reader = reader_result.map_err(|e| {
        crate::error::AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;

    let mut count = 0;
    for record in reader {
        let val = record?;
        let json_val = avro_to_json(&val);
        if pretty {
            println!("{}", colored_json::to_colored_json_auto(&json_val)?);
        } else {
            println!("{}", serde_json::to_string(&json_val)?);
        }
        count += 1;
        if let Some(limit) = head {
            if count >= limit {
                break;
            }
        }
    }
    Ok(())
}
