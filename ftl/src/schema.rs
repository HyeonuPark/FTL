use std::any::Any;
use std::cmp::{Eq, Ord};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use openapiv3 as oa;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

pub trait Schema: Any + Serialize + DeserializeOwned {
    fn schema() -> oa::Schema;
}

#[cfg(test)]
pub fn parse_example<T: Schema + serde::de::DeserializeOwned>() {
    let example = T::schema().schema_data.example.unwrap();
    let _: T = serde_json::from_value(example).unwrap();
}

impl<T: Schema> Schema for Box<T> {
    fn schema() -> oa::Schema {
        T::schema()
    }
}

impl<T: Schema> Schema for Option<T> {
    fn schema() -> oa::Schema {
        let mut schema = T::schema();
        schema.schema_data.nullable = true;
        schema
    }
}

#[test]
fn parse_example_vec_u32() {
    parse_example::<Vec<u32>>()
}

impl<T: Schema> Schema for Vec<T> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("Vec".into()),
                description: Some("Vec".into()),
                example: Some(json!([])),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Array(oa::ArrayType {
                items: oa::ReferenceOr::Item(Box::new(T::schema())),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        }
    }
}

#[test]
fn parse_example_hashset_u32() {
    parse_example::<HashSet<u32>>()
}

impl<T: Schema + Eq + Hash> Schema for HashSet<T> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("HashSet".into()),
                description: Some("HashSet".into()),
                example: Some(json!([])),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Array(oa::ArrayType {
                items: oa::ReferenceOr::Item(Box::new(T::schema())),
                min_items: None,
                max_items: None,
                unique_items: true,
            })),
        }
    }
}

#[test]
fn parse_example_btreeset_u32() {
    parse_example::<BTreeSet<u32>>()
}

impl<T: Schema + Ord> Schema for BTreeSet<T> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("BTreeSet".into()),
                description: Some("BTreeSet".into()),
                example: Some(json!([])),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Array(oa::ArrayType {
                items: oa::ReferenceOr::Item(Box::new(T::schema())),
                min_items: None,
                max_items: None,
                unique_items: true,
            })),
        }
    }
}

#[test]
fn parse_example_hashmap_u32() {
    parse_example::<HashMap<String, u32>>()
}

impl<T: Schema> Schema for HashMap<String, T> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("HashMap".into()),
                description: Some("HashMap".into()),
                example: Some(json!({})),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Object(oa::ObjectType {
                additional_properties: Some(oa::AdditionalProperties::Schema(Box::new(
                    oa::ReferenceOr::Item(T::schema()),
                ))),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_btreemap_u32() {
    parse_example::<BTreeMap<String, u32>>()
}

impl<T: Schema> Schema for BTreeMap<String, T> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("BTreeMap".into()),
                description: Some("BTreeMap".into()),
                example: Some(json!({})),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Object(oa::ObjectType {
                additional_properties: Some(oa::AdditionalProperties::Schema(Box::new(
                    oa::ReferenceOr::Item(T::schema()),
                ))),
                ..Default::default()
            })),
        }
    }
}

impl Schema for Value {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                nullable: true,
                title: Some("Value".into()),
                description: Some("Value".into()),
                example: Some(json!({})),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Any(Default::default()),
        }
    }
}

#[test]
fn parse_example_json_map() {
    parse_example::<Map<String, Value>>()
}

impl Schema for Map<String, Value> {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("Map".into()),
                description: Some("Map".into()),
                example: Some(json!({})),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Object(oa::ObjectType {
                additional_properties: Some(oa::AdditionalProperties::Schema(Box::new(
                    oa::ReferenceOr::Item(Value::schema()),
                ))),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_bool() {
    parse_example::<bool>()
}

impl Schema for bool {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("bool".into()),
                description: Some("bool".into()),
                example: Some(json!(true)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Boolean {}),
        }
    }
}

#[test]
fn parse_example_u8() {
    parse_example::<u8>()
}

impl Schema for u8 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("u8".into()),
                description: Some("u8".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                minimum: Some(0),
                maximum: Some(u8::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_u16() {
    parse_example::<u16>()
}

impl Schema for u16 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("u16".into()),
                description: Some("u16".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                minimum: Some(0),
                maximum: Some(u16::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_u32() {
    parse_example::<u32>()
}

impl Schema for u32 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("u32".into()),
                description: Some("u32".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int64),
                minimum: Some(0),
                maximum: Some(u32::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_i8() {
    parse_example::<i8>()
}

impl Schema for i8 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("i8".into()),
                description: Some("i8".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                minimum: Some(i8::min_value() as _),
                maximum: Some(i8::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_i16() {
    parse_example::<i16>()
}

impl Schema for i16 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("i16".into()),
                description: Some("i16".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                minimum: Some(i16::min_value() as _),
                maximum: Some(i16::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_i32() {
    parse_example::<i32>()
}

impl Schema for i32 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("i32".into()),
                description: Some("i32".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                minimum: Some(i32::min_value() as _),
                maximum: Some(i32::max_value() as _),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_i64() {
    parse_example::<i64>()
}

impl Schema for i64 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("i64".into()),
                description: Some("i64".into()),
                example: Some(json!(1)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int64),
                minimum: Some(i64::min_value()),
                maximum: Some(i64::max_value()),
                ..Default::default()
            })),
        }
    }
}

#[test]
fn parse_example_f32() {
    parse_example::<f32>()
}

impl Schema for f32 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("f32".into()),
                description: Some("f32".into()),
                example: Some(json!(1.0)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Number(Default::default())),
        }
    }
}

#[test]
fn parse_example_f64() {
    parse_example::<f64>()
}

impl Schema for f64 {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("f64".into()),
                description: Some("f64".into()),
                example: Some(json!(1.0)),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Number(Default::default())),
        }
    }
}

#[test]
fn parse_example_string() {
    parse_example::<String>()
}

impl Schema for String {
    fn schema() -> oa::Schema {
        oa::Schema {
            schema_data: oa::SchemaData {
                title: Some("String".into()),
                description: Some("String".into()),
                example: Some(json!("foobar")),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::String(Default::default())),
        }
    }
}
