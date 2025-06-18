// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Distributed tracing trait definitions

use std::future::Future;

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
/// Tracing is designed to be transparent and
pub mod attributes;

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
    /// Starts a new span with the given name.
    fn start_span(&self, name: String) -> Box<dyn Span + Send + Sync>;
}

pub enum SpanStatus {
    Unset,
    Ok,
    Error { description: String },
}

pub trait Span {
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
}

/// The WithSpan trait enables the creation of a `with_span` method which returns a wrapped future that
/// sets the current span for the duration of the future's execution.
///
/// see also: [opentelemetry::trace::FutureExt](https://docs.rs/opentelemetry/latest/opentelemetry/trace/trait.FutureExt.html) or
/// [tracing::instrument](https://docs.rs/tracing/latest/tracing/trait.Instrument.html) for similar functionality in other tracing libraries.
pub trait WithSpan: Sized {
    fn with_span(
        self,
        span: Box<dyn Span + Send + Sync>,
    ) -> Box<dyn Future<Output = Self> + Send + Sync>;
}
