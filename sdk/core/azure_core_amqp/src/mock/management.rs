// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::management::AmqpManagementApis;
use crate::session::AmqpSession;
use crate::value::{AmqpOrderedMap, AmqpValue};
use async_trait::async_trait;
use azure_core::{credentials::AccessToken, Result};
use std::sync::{Arc, Mutex};

type CallRecord = (String, AmqpOrderedMap<String, AmqpValue>);

pub struct MockAmqpManagement {
    is_attached: Arc<Mutex<bool>>,
    _session: Arc<Mutex<AmqpSession>>,
    _client_node_name: Arc<Mutex<String>>,
    _access_token: Arc<Mutex<AccessToken>>,
    calls: Arc<Mutex<Vec<CallRecord>>>,
    response_map: Arc<Mutex<AmqpOrderedMap<String, AmqpValue>>>,
}

impl MockAmqpManagement {
    pub fn new(
        session: AmqpSession,
        client_node_name: String,
        access_token: AccessToken,
    ) -> Result<Self> {
        Ok(Self {
            is_attached: Arc::new(Mutex::new(false)),
            _session: Arc::new(Mutex::new(session)),
            _client_node_name: Arc::new(Mutex::new(client_node_name)),
            _access_token: Arc::new(Mutex::new(access_token)),
            calls: Arc::new(Mutex::new(Vec::new())),
            response_map: Arc::new(Mutex::new(AmqpOrderedMap::new())),
        })
    }

    // Helper methods for testing
    pub fn is_attached(&self) -> bool {
        *self.is_attached.lock().unwrap()
    }
    pub fn get_calls(&self) -> Vec<CallRecord> {
        self.calls.lock().unwrap().clone()
    }

    pub fn set_response(&self, response: AmqpOrderedMap<String, AmqpValue>) {
        *self.response_map.lock().unwrap() = response;
    }
}

#[async_trait]
impl AmqpManagementApis for MockAmqpManagement {
    async fn attach(&self) -> Result<()> {
        *self.is_attached.lock().unwrap() = true;
        Ok(())
    }

    async fn detach(self) -> Result<()> {
        *self.is_attached.lock().unwrap() = false;
        Ok(())
    }

    async fn call(
        &self,
        operation_type: String,
        application_properties: AmqpOrderedMap<String, AmqpValue>,
    ) -> Result<AmqpOrderedMap<String, AmqpValue>> {
        self.calls
            .lock()
            .unwrap()
            .push((operation_type, application_properties));
        Ok(self.response_map.lock().unwrap().clone())
    }
}
