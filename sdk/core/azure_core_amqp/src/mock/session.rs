// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::connection::AmqpConnection;
use crate::session::{AmqpSessionApis, AmqpSessionOptions};
use async_trait::async_trait;
use azure_core::Result;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MockAmqpSession {
    is_begun: Arc<Mutex<bool>>,
    options: Arc<Mutex<Option<AmqpSessionOptions>>>,
}

impl MockAmqpSession {
    pub fn new() -> Self {
        Self {
            is_begun: Arc::new(Mutex::new(false)),
            options: Arc::new(Mutex::new(None)),
        }
    }

    // Helper methods for testing
    pub fn is_begun(&self) -> bool {
        *self.is_begun.lock().unwrap()
    }

    pub fn get_options(&self) -> Option<AmqpSessionOptions> {
        self.options.lock().unwrap().clone()
    }
}

#[async_trait]
impl AmqpSessionApis for MockAmqpSession {
    async fn begin(
        &self,
        _connection: &AmqpConnection,
        options: Option<AmqpSessionOptions>,
    ) -> Result<()> {
        *self.is_begun.lock().unwrap() = true;
        *self.options.lock().unwrap() = options;
        Ok(())
    }

    async fn end(&self) -> Result<()> {
        *self.is_begun.lock().unwrap() = false;
        Ok(())
    }
}
