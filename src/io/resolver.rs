use aws_sdk_s3::Client as S3Client;
use glob::glob;

use crate::io::s3;

pub async fn resolve_files(patterns: &[String], s3_client: &Option<S3Client>) -> Vec<String> {
    let mut paths = Vec::new();
    for pattern in patterns {
        if pattern.starts_with("s3://") {
            if let Some(client) = s3_client {
                match s3::list_s3_objects(client, pattern).await {
                    Ok(mut s3_paths) => paths.append(&mut s3_paths),
                    Err(e) => {
                        eprintln!("Warning: {}", e);
                        paths.push(pattern.clone());
                    }
                }
            }
        } else {
            match glob(pattern) {
                Ok(entries) => {
                    let mut matched = false;
                    for entry in entries.flatten() {
                        matched = true;
                        paths.push(entry.to_string_lossy().to_string());
                    }
                    if !matched {
                        paths.push(pattern.clone());
                    }
                }
                Err(_) => paths.push(pattern.clone()),
            }
        }
    }
    paths
}
