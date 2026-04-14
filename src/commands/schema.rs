use apache_avro::Reader;
use aws_sdk_s3::Client as S3Client;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{AvzError, Result};
use crate::io::{self, AvroInput};

pub async fn execute(files: &[String], s3_client: &Option<S3Client>) -> Result<()> {
    let paths = io::resolve_files(files, s3_client).await;
    let multi = paths.len() > 1;

    let mut output = String::new();

    for path in &paths {
        if multi {
            output.push_str(&format!("==> {} <==\n", path));
        }
        let input = io::open_input(path, s3_client).await?;
        match input {
            AvroInput::Local(f) => format_schema(Reader::new(f), path, &mut output)?,
            AvroInput::Memory(c) => format_schema(Reader::new(c), path, &mut output)?,
        }
    }

    // If stdout is a TTY and output is large, pipe through a pager
    if atty::is(atty::Stream::Stdout) && output.lines().count() > 40 {
        let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        let mut args = vec![];
        if pager == "less" {
            args.push("-R".to_string()); // pass through ANSI colors
        }
        if let Ok(mut child) = Command::new(&pager).args(&args).stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(output.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }
    }

    print!("{}", output);
    Ok(())
}

fn format_schema(
    reader_result: std::result::Result<Reader<impl std::io::Read>, apache_avro::Error>,
    path: &str,
    output: &mut String,
) -> Result<()> {
    let reader = reader_result.map_err(|e| {
        AvzError::User(format!("Not a valid Avro file {}: {}", path, e))
    })?;

    let canonical = reader.writer_schema().canonical_form();
    let parsed: serde_json::Value = serde_json::from_str(&canonical)?;
    output.push_str(&colored_json::to_colored_json_auto(&parsed)?);
    output.push('\n');
    Ok(())
}
