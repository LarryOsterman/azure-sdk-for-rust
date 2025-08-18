// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod attributes;
mod provider;
mod span;
mod tracer;

// Re-export the main types for convenience
pub use provider::TracingTracerProvider;
