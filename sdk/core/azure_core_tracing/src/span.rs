// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

// cspell:ignore callsite valueset

//! Tracing implementation of typespec_client_core tracing traits.

use azure_core::tracing::{AsAny, AttributeValue, Span, SpanGuard, SpanKind, SpanStatus};
use std::{error::Error as StdError, sync::Arc};
use tracing::{
    callsite, field::FieldSet, metadata::Kind as TracingKind, valueset, Level as TracingLevel,
    Metadata as TracingMetadata, Span as TracingCrateSpan, Value as TracingValue,
};

/// newtype for Azure Core SpanKind to enable conversion to OpenTelemetry SpanKind
pub(crate) struct TracingSpanKind(pub azure_core::tracing::SpanKind);

// Distributed tracing span attribute names. Defined in
// [OpenTelemetrySpans](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/http/http-spans.md)
// and [Azure conventions for open telemetry spans](https://github.com/Azure/azure-sdk/blob/main/docs/tracing/distributed-tracing-conventions.md)
const AZ_NAMESPACE_ATTRIBUTE: &str = "az.namespace";
const AZ_CLIENT_REQUEST_ID_ATTRIBUTE: &str = "az.client_request_id";
const ERROR_TYPE_ATTRIBUTE: &str = "error.type";
const AZ_SERVICE_REQUEST_ID_ATTRIBUTE: &str = "az.service_request.id";
const HTTP_REQUEST_RESEND_COUNT_ATTRIBUTE: &str = "http.request.resend_count";
const HTTP_RESPONSE_STATUS_CODE_ATTRIBUTE: &str = "http.response.status_code";
const HTTP_REQUEST_METHOD_ATTRIBUTE: &str = "http.request.method";
const SERVER_ADDRESS_ATTRIBUTE: &str = "server.address";
const SERVER_PORT_ATTRIBUTE: &str = "server.port";
const URL_FULL_ATTRIBUTE: &str = "url.full";

static FIELD_NAMES: &[&str] = &[
    AZ_CLIENT_REQUEST_ID_ATTRIBUTE,
    AZ_NAMESPACE_ATTRIBUTE,
    ERROR_TYPE_ATTRIBUTE,
    AZ_SERVICE_REQUEST_ID_ATTRIBUTE,
    HTTP_REQUEST_RESEND_COUNT_ATTRIBUTE,
    HTTP_RESPONSE_STATUS_CODE_ATTRIBUTE,
    HTTP_REQUEST_METHOD_ATTRIBUTE,
    SERVER_ADDRESS_ATTRIBUTE,
    SERVER_PORT_ATTRIBUTE,
    URL_FULL_ATTRIBUTE,
];

/// OpenTelemetry implementation of Span
pub(super) struct TracingSpan {
    span: TracingCrateSpan,
}

impl TracingSpan {
    pub fn new(
        name: &'static str,
        kind: SpanKind,
        package_name: &'static str,
        package_version: Option<&'static str>,
        attributes: Vec<azure_core::tracing::Attribute>,
    ) -> Arc<Self> {
        // NOTE:
        // The `tracing` crate requires span fields to be declared at compile time. Since the
        // Azure tracing abstraction allows an arbitrary set of attributes, we create a span
        // with a fixed set of well-known Azure / HTTP semantic fields, and then immediately
        // record any provided attributes that match those field names. Attributes with keys
        // that are not declared as span fields cannot be recorded directly; a future
        // enhancement could log them as events or store them in extensions.

        let pkg_version = package_version.unwrap_or("");

        // Create the span with the known field set. (Values initially empty.)
        // NOTE: tracing::span! requires the span name to be a string literal so we use a
        // constant literal ("azure_core_span") and store the dynamic name in a span field.
        let span = tracing::span!(
            TracingLevel::INFO,
            "azure_core_span",
            span_name = name,
            az_client_request_id = tracing::field::Empty,
            az_namespace = package_name,
            error_type = tracing::field::Empty,
            az_service_request_id = tracing::field::Empty,
            http_request_resend_count = tracing::field::Empty,
            http_response_status_code = tracing::field::Empty,
            http_request_method = tracing::field::Empty,
            server_address = tracing::field::Empty,
            server_port = tracing::field::Empty,
            url_full = tracing::field::Empty,
            span_kind = ?kind,
            package_name = package_name,
            package_version = pkg_version,
        );

        // Record provided attributes for those keys we have declared.
        for attr in attributes.iter() {
            let key: &str = &attr.key;
            match key {
                AZ_CLIENT_REQUEST_ID_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(AZ_CLIENT_REQUEST_ID_ATTRIBUTE, &s.as_str());
                    }
                    _ => {
                        span.record(
                            AZ_CLIENT_REQUEST_ID_ATTRIBUTE,
                            &format!("{:?}", attr.value).as_str(),
                        );
                    }
                },
                AZ_NAMESPACE_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(AZ_NAMESPACE_ATTRIBUTE, &s.as_str());
                    }
                    _ => {
                        span.record(
                            AZ_NAMESPACE_ATTRIBUTE,
                            &format!("{:?}", attr.value).as_str(),
                        );
                    }
                },
                ERROR_TYPE_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(ERROR_TYPE_ATTRIBUTE, &s.as_str());
                    }
                    _ => {
                        span.record(ERROR_TYPE_ATTRIBUTE, &format!("{:?}", attr.value).as_str());
                    }
                },
                AZ_SERVICE_REQUEST_ID_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(AZ_SERVICE_REQUEST_ID_ATTRIBUTE, &s.as_str())
                    }
                    _ => span.record(
                        AZ_SERVICE_REQUEST_ID_ATTRIBUTE,
                        &format!("{:?}", attr.value).as_str(),
                    ),
                },
                HTTP_REQUEST_RESEND_COUNT_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::I64(v) => {
                        span.record(HTTP_REQUEST_RESEND_COUNT_ATTRIBUTE, v)
                    }
                    _ => span.record(
                        HTTP_REQUEST_RESEND_COUNT_ATTRIBUTE,
                        &format!("{:?}", attr.value).as_str(),
                    ),
                },
                HTTP_RESPONSE_STATUS_CODE_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::I64(v) => {
                        span.record(HTTP_RESPONSE_STATUS_CODE_ATTRIBUTE, v)
                    }
                    _ => span.record(
                        HTTP_RESPONSE_STATUS_CODE_ATTRIBUTE,
                        &format!("{:?}", attr.value).as_str(),
                    ),
                },
                HTTP_REQUEST_METHOD_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(HTTP_REQUEST_METHOD_ATTRIBUTE, &s.as_str())
                    }
                    _ => span.record(
                        HTTP_REQUEST_METHOD_ATTRIBUTE,
                        &format!("{:?}", attr.value).as_str(),
                    ),
                },
                SERVER_ADDRESS_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(SERVER_ADDRESS_ATTRIBUTE, &s.as_str())
                    }
                    _ => span.record(
                        SERVER_ADDRESS_ATTRIBUTE,
                        &format!("{:?}", attr.value).as_str(),
                    ),
                },
                SERVER_PORT_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::I64(v) => {
                        span.record(SERVER_PORT_ATTRIBUTE, v)
                    }
                    _ => span.record(SERVER_PORT_ATTRIBUTE, &format!("{:?}", attr.value).as_str()),
                },
                URL_FULL_ATTRIBUTE => match &attr.value {
                    azure_core::tracing::AttributeValue::String(s) => {
                        span.record(URL_FULL_ATTRIBUTE, &s.as_str())
                    }
                    _ => span.record(URL_FULL_ATTRIBUTE, &format!("{:?}", attr.value).as_str()),
                },
                // Unknown attribute keys cannot be recorded directly (field not declared).
                _ => {
                    tracing::debug!(target: package_name, attribute.key = %attr.key, attribute.value = ?attr.value, "attribute not recorded (field not declared)");
                }
            }
        }

        Arc::new(Self { span })
    }
}

impl Span for TracingSpan {
    fn is_recording(&self) -> bool {
        !self.span.is_disabled()
    }

    fn end(&self) {
        let _ = self.span.enter();
    }

    fn span_id(&self) -> [u8; 8] {
        self.span.id().unwrap().into_u64().to_le_bytes()
    }

    fn set_attribute(&self, _key: &'static str, _value: AttributeValue) {
        todo!();
    }

    fn record_error(&self, _error: &dyn StdError) {
        todo!();
    }

    fn set_status(&self, _status: SpanStatus) {
        todo!();
    }

    fn propagate_headers(&self, _request: &mut azure_core::http::Request) {
        todo!();
    }

    fn set_current(&self, _context: &azure_core::http::Context) -> Box<dyn SpanGuard> {
        todo!();
    }
}

impl AsAny for TracingSpan {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct TracingSpanGuard<'a> {
    _inner: tracing::span::Entered<'a>,
}

impl SpanGuard for TracingSpanGuard<'_> {
    fn end(self) {
        // The span is ended when the guard is dropped, so no action needed here.
    }
}

impl Drop for TracingSpanGuard<'_> {
    fn drop(&mut self) {
        // The OpenTelemetry context guard will automatically end the span when dropped.
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::provider::TracingTracerProvider;
    use azure_core::http::{Context as AzureContext, Url};
    use azure_core::tracing::{Attribute, AttributeValue, SpanKind, SpanStatus, TracerProvider};
    use std::io::{Error, ErrorKind};
    use std::sync::Arc;
    use tracing::trace;

    #[test]
    fn test_open_telemetry_span_new() {
        todo!();
    }

    // cspell: ignore traceparent tracestate
    #[test]
    fn test_open_telemetry_span_propagate() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_hierarchy() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_start_with_parent() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_start_with_current() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_set_attribute() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_record_error() {
        todo!();
    }

    #[test]
    fn test_open_telemetry_span_set_status() {
        todo!();
    }

    #[tokio::test]
    async fn test_open_telemetry_span_futures() {
        todo!();
    }
}
