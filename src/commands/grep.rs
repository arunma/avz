use apache_avro::Reader;
use aws_sdk_s3::Client as S3Client;
use regex::RegexBuilder;

use crate::convert::avro_to_json;
use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(
    pattern: &str,
    files: &[String],
    s3_client: &Option<S3Client>,
    pretty: bool,
    ignore_case: bool,
    invert: bool,
    count_only: bool,
    fixed_string: bool,
) -> Result<()> {
    let escaped = if fixed_string { regex::escape(pattern) } else { pattern.to_string() };
    let re = RegexBuilder::new(&escaped)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| AvzError::User(format!("Invalid regex '{}': {}", pattern, e)))?;

    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;
    let mut total_matches = 0u64;

    for path in &paths {
        let input = io::open_input(path, s3_client).await?;
        let file_matches = match input {
            AvroInput::Local(f) => process(Reader::new(f), path, &re, pretty, invert, count_only, multi)?,
            AvroInput::Memory(c) => process(Reader::new(c), path, &re, pretty, invert, count_only, multi)?,
        };
        total_matches += file_matches;
    }

    if count_only && multi {
        println!("total: {}", total_matches);
    }

    if total_matches == 0 && !count_only {
        std::process::exit(1);
    }

    Ok(())
}

fn process(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
    re: &regex::Regex,
    pretty: bool,
    invert: bool,
    count_only: bool,
    multi: bool,
) -> Result<u64> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;

    let mut matches = 0u64;

    for record in reader {
        let val = record?;
        let json_val = avro_to_json(&val);
        let json_str = serde_json::to_string(&json_val)?;
        let found = re.is_match(&json_str);

        if found != invert {
            matches += 1;
            if !count_only {
                if multi {
                    eprintln!("==> {} <==", path);
                }
                if pretty {
                    println!("{}", colored_json::to_colored_json_auto(&json_val)?);
                } else {
                    println!("{}", json_str);
                }
            }
        }
    }

    if count_only {
        if multi {
            println!("{}: {}", path, matches);
        } else {
            println!("{}", matches);
        }
    }

    Ok(matches)
}
