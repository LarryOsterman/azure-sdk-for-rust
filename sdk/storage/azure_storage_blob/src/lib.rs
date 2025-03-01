// Copyright (c) Microsoft Corporation. All rights reserved.
//
// Licensed under the MIT License. See License.txt in the project root for license information.
// Code generated by Microsoft (R) Rust Code Generator. DO NOT EDIT.

mod clients;
mod generated;

pub(crate) mod pipeline;

pub use clients::*;

pub mod models;

pub use crate::generated::clients::*;
pub(crate) use blob_client::BlobClient as GeneratedBlobClient;

pub use crate::generated::clients::blob_blob_client::BlobBlobClientGetPropertiesOptions;
pub use crate::generated::clients::blob_client::BlobClientOptions;
