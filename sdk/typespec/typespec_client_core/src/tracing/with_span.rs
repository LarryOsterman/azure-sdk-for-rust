// Copyright (C) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

// cspell: ignore cloneable

use super::Span;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::Context as TaskContext;
use std::task::Poll;

impl<T: Sized> FutureExt for T {}

impl<T: std::future::Future> std::future::Future for WithSpan<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.set_current();

        this.inner.poll(task_cx)
    }
}

trait CloneableSpan: Span + Clone {}

#[pin_project]
/// A future, stream, or sink that has an associated context.
#[derive(Clone, Debug)]
pub struct WithSpan<T> {
    #[pin]
    inner: T,
    span: Box<dyn CloneableSpan + Send + Sync>,
}

/// Extension trait allowing futures, streams, and sinks to be traced with a span.
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as current while it is being polled.
    ///
    /// [`Context`]: Context
    fn with_span(self, span: Box<dyn CloneableSpan + Send + Sync>) -> WithSpan<Self> {
        WithSpan { inner: self, span }
    }
}
