// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Attribute conversions between typespec_client_core and Tracing.

use azure_core::tracing::{
    Attribute as AzureAttribute, AttributeArray as AzureAttributeArray,
    AttributeValue as AzureAttributeValue,
};
use tracing::{field::DebugValue, Value as TracingValue};

pub(super) struct AttributeArray(AzureAttributeArray);

pub(super) struct AttributeValue(pub AzureAttributeValue);

pub(super) struct TracingAttribute(pub AzureAttribute);

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        AttributeValue(AzureAttributeValue::Bool(value))
    }
}

impl From<i64> for AttributeValue {
    fn from(value: i64) -> Self {
        AttributeValue(AzureAttributeValue::I64(value))
    }
}

impl From<f64> for AttributeValue {
    fn from(value: f64) -> Self {
        AttributeValue(AzureAttributeValue::I64(value as i64))
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        AttributeValue(AzureAttributeValue::String(value))
    }
}

impl From<Vec<bool>> for AttributeArray {
    fn from(values: Vec<bool>) -> Self {
        AttributeArray(AzureAttributeArray::Bool(values))
    }
}

impl From<Vec<i64>> for AttributeArray {
    fn from(values: Vec<i64>) -> Self {
        AttributeArray(AzureAttributeArray::I64(values))
    }
}
impl From<Vec<f64>> for AttributeArray {
    fn from(values: Vec<f64>) -> Self {
        AttributeArray(AzureAttributeArray::F64(values))
    }
}

impl From<Vec<String>> for AttributeArray {
    fn from(values: Vec<String>) -> Self {
        AttributeArray(AzureAttributeArray::String(values))
    }
}
