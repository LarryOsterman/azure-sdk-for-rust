// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Type conversions between Azure attributes and OpenTelemetry types.

use crate::{Attribute, AttributeArray, AttributeValue};
use opentelemetry::{Key, KeyValue, Value};

impl From<AttributeArray> for Value {
    fn from(array: AttributeArray) -> Self {
        match array {
            AttributeArray::Bool(values) => {
                Value::Array(values.into_iter().map(Value::Bool).collect())
            }
            AttributeArray::I64(values) => {
                Value::Array(values.into_iter().map(Value::I64).collect())
            }
            AttributeArray::U64(values) => {
                Value::Array(values.into_iter().map(|v| Value::I64(v as i64)).collect())
            }
            AttributeArray::String(values) => {
                Value::Array(values.into_iter().map(Value::String).collect())
            }
        }
    }
}

impl From<AttributeValue> for Value {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Bool(v) => Value::Bool(v),
            AttributeValue::I64(v) => Value::I64(v),
            AttributeValue::U64(v) => Value::I64(v as i64),
            AttributeValue::String(v) => Value::String(v.into()),
            AttributeValue::Array(array) => array.into(),
        }
    }
}

impl From<Attribute> for KeyValue {
    fn from(attr: Attribute) -> Self {
        KeyValue::new(Key::from(attr.key), attr.value.into())
    }
}

/// Converts a collection of attributes to OpenTelemetry KeyValue pairs.
pub fn attributes_to_keyvalues(attributes: Vec<Attribute>) -> Vec<KeyValue> {
    attributes.into_iter().map(KeyValue::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_value_converts_to_otel_value() {
        let bool_val = AttributeValue::Bool(true);
        let otel_val: Value = bool_val.into();
        matches!(otel_val, Value::Bool(true));

        let string_val = AttributeValue::String("test".to_string());
        let otel_val: Value = string_val.into();
        matches!(otel_val, Value::String(_));
    }

    #[test]
    fn attribute_converts_to_keyvalue() {
        let attr = Attribute {
            key: "test.key".to_string(),
            value: AttributeValue::String("test.value".to_string()),
        };
        let kv: KeyValue = attr.into();
        assert_eq!(kv.key.as_str(), "test.key");
    }

    #[test]
    fn attribute_array_converts_correctly() {
        let array = AttributeArray::String(vec!["a".to_string(), "b".to_string()]);
        let value: Value = array.into();
        matches!(value, Value::Array(_));
    }

    #[test]
    fn attributes_to_keyvalues_converts_collection() {
        let attributes = vec![
            Attribute {
                key: "service.name".to_string(),
                value: AttributeValue::String("test".to_string()),
            },
            Attribute {
                key: "success".to_string(),
                value: AttributeValue::Bool(true),
            },
        ];

        let keyvalues = attributes_to_keyvalues(attributes);
        assert_eq!(keyvalues.len(), 2);
    }
}
