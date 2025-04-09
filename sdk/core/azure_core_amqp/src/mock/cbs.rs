// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::cbs::AmqpClaimsBasedSecurityApis;
use crate::session::AmqpSession;
use async_trait::async_trait;
use azure_core::Result;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// Type alias for the complex authorized path type
type AuthorizedPath = (String, Option<String>, String, time::OffsetDateTime);

pub struct MockAmqpClaimsBasedSecurity<'a> {
    is_attached: Arc<Mutex<bool>>,
    session: PhantomData<&'a AmqpSession>,
    authorized_paths: Arc<Mutex<Vec<AuthorizedPath>>>,
}

impl<'a> MockAmqpClaimsBasedSecurity<'a> {
    pub fn new(_session: &'a AmqpSession) -> Result<Self> {
        Ok(Self {
            is_attached: Arc::new(Mutex::new(false)),
            session: PhantomData,
            authorized_paths: Arc::new(Mutex::new(Vec::new())),
        })
    }

    // Helper methods for testing
    pub fn is_attached(&self) -> bool {
        *self.is_attached.lock().unwrap()
    }
    pub fn get_authorized_paths(&self) -> Vec<AuthorizedPath> {
        self.authorized_paths.lock().unwrap().clone()
    }

    pub fn clear_authorized_paths(&self) {
        self.authorized_paths.lock().unwrap().clear();
    }
}

#[async_trait]
impl AmqpClaimsBasedSecurityApis for MockAmqpClaimsBasedSecurity<'_> {
    async fn attach(&self) -> Result<()> {
        *self.is_attached.lock().unwrap() = true;
        Ok(())
    }

    async fn detach(self) -> Result<()> {
        *self.is_attached.lock().unwrap() = false;
        Ok(())
    }

    async fn authorize_path(
        &self,
        path: String,
        token_type: Option<String>,
        secret: String,
        expires_on: time::OffsetDateTime,
    ) -> Result<()> {
        self.authorized_paths
            .lock()
            .unwrap()
            .push((path, token_type, secret, expires_on));
        Ok(())
    }
}
