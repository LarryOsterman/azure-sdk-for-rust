// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

pub(crate) mod connection_manager;
pub(crate) mod management;
pub mod retry;
pub(crate) mod user_agent;

// Public API
pub(crate) use retry::{retry_azure_operation, RetryOptions};

pub(crate) use management::ManagementInstance;
