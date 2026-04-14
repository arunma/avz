use aws_sdk_s3::Client as S3Client;

use crate::error::{AvzError, Result};

pub fn parse_s3_uri(uri: &str) -> Result<(&str, &str)> {
    let without_scheme = &uri[5..]; // strip "s3://"
    let slash = without_scheme
        .find('/')
        .ok_or_else(|| AvzError::User(format!("Invalid S3 URI — expected s3://bucket/key: {}", uri)))?;
    let bucket = &without_scheme[..slash];
    let key = &without_scheme[slash + 1..];
    Ok((bucket, key))
}

pub async fn read_s3_bytes(client: &S3Client, uri: &str) -> Result<Vec<u8>> {
    let (bucket, key) = parse_s3_uri(uri)?;
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| AvzError::S3(format!("Failed to read {}: {}", uri, e)))?;
    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| AvzError::S3(format!("Failed to download {}: {}", uri, e)))?
        .into_bytes()
        .to_vec();
    Ok(bytes)
}

pub async fn list_s3_objects(client: &S3Client, uri: &str) -> Result<Vec<String>> {
    let (bucket, key) = parse_s3_uri(uri)?;

    let prefix = if let Some(pos) = key.find('*') {
        &key[..key[..pos].rfind('/').map(|i| i + 1).unwrap_or(0)]
    } else {
        return Ok(vec![uri.to_string()]);
    };

    let pattern = glob::Pattern::new(key)
        .map_err(|e| AvzError::User(format!("Invalid glob pattern '{}': {}", key, e)))?;

    let mut uris = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AvzError::S3(format!("Failed to list s3://{}/{}: {}", bucket, prefix, e)))?;

        for obj in resp.contents() {
            if let Some(obj_key) = obj.key() {
                if pattern.matches(obj_key) {
                    uris.push(format!("s3://{}/{}", bucket, obj_key));
                }
            }
        }

        if resp.is_truncated() == Some(true) {
            continuation_token = resp.next_continuation_token().map(|s| s.to_string());
        } else {
            break;
        }
    }

    uris.sort();
    Ok(uris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_s3_uri() {
        let (bucket, key) = parse_s3_uri("s3://my-bucket/path/to/file.avro").unwrap();
        assert_eq!(bucket, "my-bucket");
        assert_eq!(key, "path/to/file.avro");
    }

    #[test]
    fn test_parse_s3_uri_deep_path() {
        let (bucket, key) = parse_s3_uri("s3://b/a/b/c/d/e.avro").unwrap();
        assert_eq!(bucket, "b");
        assert_eq!(key, "a/b/c/d/e.avro");
    }

    #[test]
    fn test_parse_s3_uri_no_key() {
        assert!(parse_s3_uri("s3://bucket").is_err());
    }
}
