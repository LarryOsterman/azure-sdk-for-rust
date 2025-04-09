// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::connection::{AmqpConnectionApis, AmqpConnectionOptions};
use crate::value::{AmqpOrderedMap, AmqpSymbol, AmqpValue};
use async_trait::async_trait;
use azure_core::{http::Url, Result};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct MockAmqpConnection {
    is_open: Arc<Mutex<bool>>,
    name: Arc<Mutex<Option<String>>>,
    url: Arc<Mutex<Option<Url>>>,
    options: Arc<Mutex<Option<AmqpConnectionOptions>>>,
}

impl MockAmqpConnection {
    pub fn new() -> Self {
        Self {
            is_open: Arc::new(Mutex::new(false)),
            name: Arc::new(Mutex::new(None)),
            url: Arc::new(Mutex::new(None)),
            options: Arc::new(Mutex::new(None)),
        }
    }

    // Helper methods for testing
    pub fn is_open(&self) -> bool {
        *self.is_open.lock().unwrap()
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.lock().unwrap().clone()
    }

    pub fn get_url(&self) -> Option<Url> {
        self.url.lock().unwrap().clone()
    }

    pub fn get_options(&self) -> Option<AmqpConnectionOptions> {
        self.options.lock().unwrap().clone()
    }
}

#[async_trait]
impl AmqpConnectionApis for MockAmqpConnection {
    async fn open(
        &self,
        name: String,
        url: Url,
        options: Option<AmqpConnectionOptions>,
    ) -> Result<()> {
        *self.is_open.lock().unwrap() = true;
        *self.name.lock().unwrap() = Some(name);
        *self.url.lock().unwrap() = Some(url);
        *self.options.lock().unwrap() = options;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        *self.is_open.lock().unwrap() = false;
        Ok(())
    }

    async fn close_with_error(
        &self,
        _condition: AmqpSymbol,
        _description: Option<String>,
        _info: Option<AmqpOrderedMap<AmqpSymbol, AmqpValue>>,
    ) -> Result<()> {
        *self.is_open.lock().unwrap() = false;
        Ok(())
    }
}
