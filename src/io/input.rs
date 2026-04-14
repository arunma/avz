use aws_sdk_s3::Client as S3Client;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};

use crate::error::{AvzError, Result};
use crate::io::s3;

pub enum AvroInput {
    Local(File),
    Memory(Cursor<Vec<u8>>),
}

pub async fn open_input(path: &str, s3_client: &Option<S3Client>) -> Result<AvroInput> {
    if path.starts_with("s3://") {
        let client = s3_client
            .as_ref()
            .ok_or_else(|| AvzError::S3("S3 client not initialized".into()))?;
        let bytes = s3::read_s3_bytes(client, path).await?;
        Ok(AvroInput::Memory(Cursor::new(bytes)))
    } else {
        let f = File::open(path)
            .map_err(|e| AvzError::Io(std::io::Error::new(e.kind(), format!("Cannot open {}: {}", path, e))))?;
        Ok(AvroInput::Local(f))
    }
}

/// Parsed Avro file header containing metadata and sync marker.
pub struct AvroHeader {
    pub metadata: HashMap<String, Vec<u8>>,
    pub sync_marker: [u8; 16],
}

impl AvroHeader {
    pub fn codec(&self) -> &str {
        self.metadata
            .get("avro.codec")
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("null")
    }

    pub fn schema_json(&self) -> Option<&str> {
        self.metadata
            .get("avro.schema")
            .and_then(|v| std::str::from_utf8(v).ok())
    }
}

/// Read the Avro container header from raw bytes.
/// Parses magic bytes, metadata map, and 16-byte sync marker.
pub fn read_avro_header(reader: &mut impl Read) -> Result<AvroHeader> {
    // Read and verify magic bytes: 'O', 'b', 'j', 1
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != [b'O', b'b', b'j', 1] {
        return Err(AvzError::User("Not a valid Avro file (bad magic bytes)".into()));
    }

    // Read metadata map (encoded as Avro long-prefixed blocks)
    let mut metadata = HashMap::new();
    loop {
        let block_count = read_varint_long(reader)?;
        if block_count == 0 {
            break;
        }
        let count = block_count.unsigned_abs() as usize;
        if block_count < 0 {
            // Negative count means the next long is the byte size of the block; skip it
            let _byte_size = read_varint_long(reader)?;
        }
        for _ in 0..count {
            let key = read_avro_string(reader)?;
            let value = read_avro_bytes(reader)?;
            metadata.insert(key, value);
        }
    }

    // Read 16-byte sync marker
    let mut sync_marker = [0u8; 16];
    reader.read_exact(&mut sync_marker)?;

    Ok(AvroHeader {
        metadata,
        sync_marker,
    })
}

fn read_varint_long(reader: &mut impl Read) -> Result<i64> {
    let mut val: u64 = 0;
    let mut shift = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        val |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(AvzError::User("Varint too long".into()));
        }
    }
    // Zigzag decode
    Ok(((val >> 1) as i64) ^ -((val & 1) as i64))
}

fn read_avro_string(reader: &mut impl Read) -> Result<String> {
    let len = read_varint_long(reader)? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| AvzError::User(format!("Invalid UTF-8 in metadata key: {}", e)))
}

fn read_avro_bytes(reader: &mut impl Read) -> Result<Vec<u8>> {
    let len = read_varint_long(reader)? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apache_avro::{Schema, Writer, types::Value};

    #[test]
    fn test_read_avro_header() {
        // Create a minimal Avro file in memory
        let schema = Schema::parse_str(r#"{"type": "record", "name": "Test", "fields": [{"name": "x", "type": "int"}]}"#).unwrap();
        let mut writer = Writer::new(&schema, Vec::new());
        writer.append(Value::Record(vec![("x".into(), Value::Int(1))])).unwrap();
        let bytes = writer.into_inner().unwrap();

        let mut cursor = Cursor::new(bytes);
        let header = read_avro_header(&mut cursor).unwrap();

        assert!(header.schema_json().is_some());
        assert_eq!(header.codec(), "null");
        assert_eq!(header.sync_marker.len(), 16);
    }

    #[test]
    fn test_read_avro_header_invalid() {
        let mut cursor = Cursor::new(vec![0, 1, 2, 3]);
        assert!(read_avro_header(&mut cursor).is_err());
    }
}
