// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Distributed tracing trait definitions
//!
use std::sync::Arc;

use crate::http::Context;

/// Overall architecture for distributed tracing in the SDK.
///
/// This module defines the traits that are used to implement distributed tracing functionality.
///
/// Notes: There are three major traits defined here:
/// - TracerProvider: This trait is responsible for providing tracers - this is the
///   entrypoint for distributed tracing in the SDK.
/// - Tracer: This trait is responsible for creating spans and managing the active span.
/// - Span: This trait represents a single unit of work in the distributed tracing system.
///
pub mod attributes;
pub mod with_context;
pub use with_context::{FutureExt, WithContext};

/// The TracerProvider trait is the entrypoint for distributed tracing in the SDK.
///
/// It provides a method to get a tracer for a specific name and package version.
pub trait TracerProvider {
    /// Returns a tracer for the given name.
    ///
    /// Arguments:
    /// - `package_name`: The name of the package for which the tracer is requested.
    /// - `package_version`: The version of the package for which the tracer is requested.
    fn get_tracer(
        &self,
        package_name: String,
        package_version: String,
    ) -> Box<dyn Tracer + Send + Sync>;
}

pub trait Tracer {
    /// Starts a new span with the given name and type.
    fn start_span(&self, name: String, kind: SpanKind) -> Arc<dyn Span + Send + Sync>;

    /// Starts a new child with the given name, type, and parent span.
    fn start_span_with_parent(
        &self,
        name: String,
        kind: SpanKind,
        parent: Arc<dyn Span + Send + Sync>,
    ) -> Arc<dyn Span + Send + Sync>;
}
pub enum SpanStatus {
    Unset,
    Ok,
    Error { description: String },
}

#[derive(Debug, Default)]
pub enum SpanKind {
    #[default]
    Internal,
    Client,
    Server,
    Producer,
    Consumer,
}

pub trait SpanGuard {
    /// Ends the span when dropped.
    fn end(self) -> crate::Result<()>;
}

pub trait Span: AsAny {
    /// Ends the current span.
    fn end(&self) -> crate::Result<()>;

    /// Adds an event to the current span.
    fn add_event(
        &self,
        name: String,
        attributes: Option<Vec<attributes::KeyValue>>,
    ) -> crate::Result<()>;

    fn set_attribute(&self, key: String, value: attributes::AttributeValue) -> crate::Result<()>;

    fn record_error(&self, error: &dyn std::error::Error) -> crate::Result<()>;

    fn set_status(&self, status: SpanStatus) -> crate::Result<()>;

    fn set_current(&self, context: &Context) -> crate::Result<Box<dyn SpanGuard>>;
}

pub trait AsAny {
    /// Returns a reference to the current object as a trait object.
    fn as_any(&self) -> &dyn std::any::Any;
}
