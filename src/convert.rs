use apache_avro::types::Value;
use apache_avro::Schema;
use serde_json::json;

use crate::error::{AvzError, Result};

pub fn avro_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => json!(null),
        Value::Boolean(b) => json!(b),
        Value::Int(i) => json!(i),
        Value::Long(l) => json!(l),
        Value::Float(f) => json!(f),
        Value::Double(d) => json!(d),
        Value::Bytes(b) => json!(format!("{:?}", b)),
        Value::String(s) => json!(s),
        Value::Fixed(_, b) => json!(format!("{:?}", b)),
        Value::Enum(_, s) => json!(s),
        Value::Union(_, v) => avro_to_json(v),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(avro_to_json).collect())
        }
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> =
                map.iter().map(|(k, v)| (k.clone(), avro_to_json(v))).collect();
            serde_json::Value::Object(obj)
        }
        Value::Record(fields) => {
            let obj: serde_json::Map<String, serde_json::Value> =
                fields.iter().map(|(k, v)| (k.clone(), avro_to_json(v))).collect();
            serde_json::Value::Object(obj)
        }
        Value::Date(d) => json!(d),
        Value::Decimal(d) => json!(format!("{:?}", d)),
        Value::TimeMillis(t) => json!(t),
        Value::TimeMicros(t) => json!(t),
        Value::TimestampMillis(t) => json!(t),
        Value::TimestampMicros(t) => json!(t),
        Value::Duration(d) => json!(format!("{:?}", d)),
        Value::Uuid(u) => json!(u.to_string()),
        _ => json!(format!("{:?}", value)),
    }
}

pub fn json_to_avro(json: &serde_json::Value, schema: &Schema) -> Result<Value> {
    match schema {
        Schema::Null => Ok(Value::Null),
        Schema::Boolean => match json {
            serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
            _ => Err(AvzError::User(format!("Expected boolean, got {}", json))),
        },
        Schema::Int => match json {
            serde_json::Value::Number(n) => {
                Ok(Value::Int(n.as_i64().ok_or_else(|| AvzError::User(format!("Expected int, got {}", n)))? as i32))
            }
            _ => Err(AvzError::User(format!("Expected int, got {}", json))),
        },
        Schema::Long => match json {
            serde_json::Value::Number(n) => {
                Ok(Value::Long(n.as_i64().ok_or_else(|| AvzError::User(format!("Expected long, got {}", n)))?))
            }
            _ => Err(AvzError::User(format!("Expected long, got {}", json))),
        },
        Schema::Float => match json {
            serde_json::Value::Number(n) => {
                Ok(Value::Float(n.as_f64().ok_or_else(|| AvzError::User(format!("Expected float, got {}", n)))? as f32))
            }
            _ => Err(AvzError::User(format!("Expected float, got {}", json))),
        },
        Schema::Double => match json {
            serde_json::Value::Number(n) => {
                Ok(Value::Double(n.as_f64().ok_or_else(|| AvzError::User(format!("Expected double, got {}", n)))?))
            }
            _ => Err(AvzError::User(format!("Expected double, got {}", json))),
        },
        Schema::Bytes => match json {
            serde_json::Value::String(s) => Ok(Value::Bytes(s.as_bytes().to_vec())),
            _ => Err(AvzError::User(format!("Expected string for bytes, got {}", json))),
        },
        Schema::String => match json {
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            _ => Err(AvzError::User(format!("Expected string, got {}", json))),
        },
        Schema::Array(inner) => match json {
            serde_json::Value::Array(arr) => {
                let items: Result<Vec<Value>> = arr.iter().map(|v| json_to_avro(v, &inner.items)).collect();
                Ok(Value::Array(items?))
            }
            _ => Err(AvzError::User(format!("Expected array, got {}", json))),
        },
        Schema::Map(inner) => match json {
            serde_json::Value::Object(map) => {
                let items: Result<Vec<(String, Value)>> = map.iter()
                    .map(|(k, v)| Ok((k.clone(), json_to_avro(v, &inner.types)?)))
                    .collect();
                Ok(Value::Map(items?.into_iter().collect()))
            }
            _ => Err(AvzError::User(format!("Expected object for map, got {}", json))),
        },
        Schema::Union(union_schema) => {
            if json.is_null() {
                if union_schema.variants().iter().any(|s| matches!(s, Schema::Null)) {
                    return Ok(Value::Union(0, Box::new(Value::Null)));
                }
            }
            for (i, variant) in union_schema.variants().iter().enumerate() {
                if matches!(variant, Schema::Null) && json.is_null() {
                    return Ok(Value::Union(i as u32, Box::new(Value::Null)));
                }
                if !matches!(variant, Schema::Null) {
                    if let Ok(val) = json_to_avro(json, variant) {
                        return Ok(Value::Union(i as u32, Box::new(val)));
                    }
                }
            }
            Err(AvzError::User(format!("No matching union variant for {}", json)))
        },
        Schema::Record(record_schema) => match json {
            serde_json::Value::Object(map) => {
                let mut fields = Vec::new();
                for field in &record_schema.fields {
                    let val = if let Some(v) = map.get(&field.name) {
                        json_to_avro(v, &field.schema)?
                    } else if let Some(default) = &field.default {
                        json_to_avro(default, &field.schema)?
                    } else if matches!(field.schema, Schema::Union(_)) {
                        Value::Union(0, Box::new(Value::Null))
                    } else {
                        return Err(AvzError::User(format!("Missing required field: {}", field.name)));
                    };
                    fields.push((field.name.clone(), val));
                }
                Ok(Value::Record(fields))
            }
            _ => Err(AvzError::User(format!("Expected object for record, got {}", json))),
        },
        Schema::Enum(enum_schema) => match json {
            serde_json::Value::String(s) => {
                if let Some(idx) = enum_schema.symbols.iter().position(|sym| sym == s) {
                    Ok(Value::Enum(idx as u32, s.clone()))
                } else {
                    Err(AvzError::User(format!("Unknown enum symbol: {}", s)))
                }
            }
            _ => Err(AvzError::User(format!("Expected string for enum, got {}", json))),
        },
        Schema::Fixed(fixed_schema) => match json {
            serde_json::Value::String(s) => {
                let bytes = s.as_bytes().to_vec();
                if bytes.len() != fixed_schema.size {
                    return Err(AvzError::User(format!(
                        "Fixed size mismatch: expected {}, got {}",
                        fixed_schema.size,
                        bytes.len()
                    )));
                }
                Ok(Value::Fixed(fixed_schema.size, bytes))
            }
            _ => Err(AvzError::User(format!("Expected string for fixed, got {}", json))),
        },
        _ => Err(AvzError::User(format!("Unsupported schema type for conversion: {:?}", schema))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_avro_null_to_json() {
        assert_eq!(avro_to_json(&Value::Null), json!(null));
    }

    #[test]
    fn test_avro_primitives_to_json() {
        assert_eq!(avro_to_json(&Value::Boolean(true)), json!(true));
        assert_eq!(avro_to_json(&Value::Int(42)), json!(42));
        assert_eq!(avro_to_json(&Value::Long(123456789)), json!(123456789));
        assert_eq!(avro_to_json(&Value::Float(3.14)), json!(3.14f32));
        assert_eq!(avro_to_json(&Value::Double(2.718)), json!(2.718));
        assert_eq!(avro_to_json(&Value::String("hello".into())), json!("hello"));
    }

    #[test]
    fn test_avro_bytes_to_json() {
        let bytes = Value::Bytes(vec![1, 2, 3]);
        let result = avro_to_json(&bytes);
        assert!(result.is_string());
    }

    #[test]
    fn test_avro_record_to_json() {
        let record = Value::Record(vec![
            ("name".into(), Value::String("Alice".into())),
            ("age".into(), Value::Int(30)),
        ]);
        let json = avro_to_json(&record);
        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_avro_array_to_json() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(avro_to_json(&arr), json!([1, 2, 3]));
    }

    #[test]
    fn test_avro_map_to_json() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::String("val".into()));
        let result = avro_to_json(&Value::Map(map));
        assert_eq!(result["key"], "val");
    }

    #[test]
    fn test_avro_union_to_json() {
        let union_val = Value::Union(1, Box::new(Value::String("hello".into())));
        assert_eq!(avro_to_json(&union_val), json!("hello"));
    }

    #[test]
    fn test_avro_enum_to_json() {
        let enum_val = Value::Enum(0, "HEARTS".into());
        assert_eq!(avro_to_json(&enum_val), json!("HEARTS"));
    }

    #[test]
    fn test_avro_logical_types_to_json() {
        assert_eq!(avro_to_json(&Value::Date(18628)), json!(18628));
        assert_eq!(avro_to_json(&Value::TimestampMillis(1000000)), json!(1000000));
        assert_eq!(avro_to_json(&Value::TimestampMicros(1000000)), json!(1000000));
        let uuid = apache_avro::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            avro_to_json(&Value::Uuid(uuid)),
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_json_to_avro_primitives() {
        assert_eq!(json_to_avro(&json!(null), &Schema::Null).unwrap(), Value::Null);
        assert_eq!(json_to_avro(&json!(true), &Schema::Boolean).unwrap(), Value::Boolean(true));
        assert_eq!(json_to_avro(&json!(42), &Schema::Int).unwrap(), Value::Int(42));
        assert_eq!(json_to_avro(&json!(99), &Schema::Long).unwrap(), Value::Long(99));
        assert_eq!(json_to_avro(&json!("hello"), &Schema::String).unwrap(), Value::String("hello".into()));
    }

    #[test]
    fn test_json_to_avro_record() {
        let schema_str = r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"}
            ]
        }"#;
        let schema = Schema::parse_str(schema_str).unwrap();
        let json = json!({"name": "Alice", "age": 30});
        let avro = json_to_avro(&json, &schema).unwrap();
        match avro {
            Value::Record(fields) => {
                assert_eq!(fields[0], ("name".to_string(), Value::String("Alice".into())));
                assert_eq!(fields[1], ("age".to_string(), Value::Int(30)));
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_json_to_avro_nested() {
        let schema_str = r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "tags", "type": {"type": "array", "items": "string"}}
            ]
        }"#;
        let schema = Schema::parse_str(schema_str).unwrap();
        let json = json!({"tags": ["a", "b", "c"]});
        let avro = json_to_avro(&json, &schema).unwrap();
        match avro {
            Value::Record(fields) => {
                match &fields[0].1 {
                    Value::Array(items) => assert_eq!(items.len(), 3),
                    _ => panic!("Expected Array"),
                }
            }
            _ => panic!("Expected Record"),
        }
    }
}
