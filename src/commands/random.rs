use apache_avro::{Schema, Writer, types::Value};
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs;

use crate::convert::avro_to_json;
use crate::error::{AvzError, Result};

pub async fn execute(
    schema_path: &str,
    count: usize,
    format: &str,
    output_path: Option<&str>,
    seed: Option<u64>,
    pretty: bool,
) -> Result<()> {
    let schema_str = fs::read_to_string(schema_path)
        .map_err(|e| AvzError::User(format!("Cannot read schema file {}: {}", schema_path, e)))?;
    let schema = Schema::parse_str(&schema_str)?;

    let mut rng: Box<dyn rand::RngCore> = match seed {
        Some(s) => Box::new(StdRng::seed_from_u64(s)),
        None => Box::new(rand::thread_rng()),
    };

    match format.to_lowercase().as_str() {
        "json" => {
            for _ in 0..count {
                let val = generate_random(&schema, &mut rng)?;
                let json_val = avro_to_json(&val);
                if pretty {
                    println!("{}", colored_json::to_colored_json_auto(&json_val)?);
                } else {
                    println!("{}", serde_json::to_string(&json_val)?);
                }
            }
        }
        "avro" => {
            let path = output_path
                .ok_or_else(|| AvzError::User("--output is required for avro format".into()))?;
            let output_file = fs::File::create(path)
                .map_err(|e| AvzError::User(format!("Cannot create output file {}: {}", path, e)))?;
            let mut writer = Writer::new(&schema, output_file);
            for _ in 0..count {
                let val = generate_random(&schema, &mut rng)?;
                writer.append(val)?;
            }
            writer.flush()?;
            eprintln!("Wrote {} random records to {}", count, path);
        }
        _ => {
            return Err(AvzError::User(format!(
                "Unknown format: {}. Supported: json, avro",
                format
            )));
        }
    }
    Ok(())
}

fn generate_random(schema: &Schema, rng: &mut dyn rand::RngCore) -> Result<Value> {
    match schema {
        Schema::Null => Ok(Value::Null),
        Schema::Boolean => Ok(Value::Boolean(rng.gen())),
        Schema::Int => Ok(Value::Int(rng.gen_range(-1000..1000))),
        Schema::Long => Ok(Value::Long(rng.gen_range(-100000..100000))),
        Schema::Float => Ok(Value::Float(rng.gen_range(-1000.0f32..1000.0f32))),
        Schema::Double => Ok(Value::Double(rng.gen_range(-1000.0..1000.0))),
        Schema::Bytes => {
            let len = rng.gen_range(0..20);
            let mut bytes = vec![0u8; len];
            rng.fill_bytes(&mut bytes);
            Ok(Value::Bytes(bytes))
        }
        Schema::String => {
            let words = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
                        "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi"];
            let n = rng.gen_range(1..4);
            let s: Vec<&str> = (0..n).map(|_| words[rng.gen_range(0..words.len())]).collect();
            Ok(Value::String(s.join(" ")))
        }
        Schema::Array(inner) => {
            let len = rng.gen_range(0..5);
            let items: Result<Vec<Value>> = (0..len).map(|_| generate_random(&inner.items, rng)).collect();
            Ok(Value::Array(items?))
        }
        Schema::Map(inner) => {
            let len = rng.gen_range(0..4);
            let items: Result<Vec<(String, Value)>> = (0..len)
                .map(|i| {
                    let key = format!("key_{}", i);
                    let val = generate_random(&inner.types, rng)?;
                    Ok((key, val))
                })
                .collect();
            Ok(Value::Map(items?.into_iter().collect()))
        }
        Schema::Union(union_schema) => {
            let variants = union_schema.variants();
            let idx = rng.gen_range(0..variants.len());
            let val = generate_random(&variants[idx], rng)?;
            Ok(Value::Union(idx as u32, Box::new(val)))
        }
        Schema::Record(record_schema) => {
            let fields: Result<Vec<(String, Value)>> = record_schema
                .fields
                .iter()
                .map(|field| {
                    let val = generate_random(&field.schema, rng)?;
                    Ok((field.name.clone(), val))
                })
                .collect();
            Ok(Value::Record(fields?))
        }
        Schema::Enum(enum_schema) => {
            let idx = rng.gen_range(0..enum_schema.symbols.len());
            Ok(Value::Enum(idx as u32, enum_schema.symbols[idx].clone()))
        }
        Schema::Fixed(fixed_schema) => {
            let mut bytes = vec![0u8; fixed_schema.size];
            rng.fill_bytes(&mut bytes);
            Ok(Value::Fixed(fixed_schema.size, bytes))
        }
        _ => Err(AvzError::User(format!("Cannot generate random data for schema: {:?}", schema))),
    }
}
