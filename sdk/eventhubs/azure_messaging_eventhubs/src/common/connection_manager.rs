// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use crate::common::user_agent::get_package_version;
use crate::{
    common::user_agent::{get_package_name, get_platform_info, get_user_agent},
    error::{ErrorKind, EventHubsError},
    models::AmqpValue,
};
use async_lock::{Mutex, OnceCell};
use azure_core::{credentials::AccessToken, http::Url, Result, Uuid};
use azure_core_amqp::{
    AmqpClaimsBasedSecurity, AmqpClaimsBasedSecurityApis as _, AmqpConnection, AmqpConnectionApis,
    AmqpConnectionOptions, AmqpSession, AmqpSessionApis as _, AmqpSymbol,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace};

/// The connection manager is responsible for managing the connection to the Event Hubs service.
/// It also handles authorization and connection recovery.
///
/// Currently the connection manager only handles a *single* connection, eventually it will manage
/// a pool of connections to the service.
pub(crate) struct ConnectionManager {
    url: Url,
    application_id: Option<String>,
    custom_endpoint: Option<Url>,
    connections: OnceCell<Arc<dyn AmqpConnectionApis + Send + Sync>>,
    authorization_scopes: Mutex<HashMap<Url, AccessToken>>,
    connection_name: String,
}

#[cfg(test)]
use azure_core_amqp::mocks::MockAmqpConnection;

impl ConnectionManager {
    pub fn new(url: Url, application_id: Option<String>, custom_endpoint: Option<Url>) -> Self {
        let connection_name = application_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Self {
            url,
            application_id,
            connection_name,
            custom_endpoint,
            connections: OnceCell::new(),
            authorization_scopes: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    async fn ensure_mock_connection(&self, connection: Arc<MockAmqpConnection>) -> Result<()> {
        trace!("Ensuring Mocked connection for {}.", self.url);

        connection
            .open(
                self.connection_name.clone(),
                self.url.clone(),
                Some(AmqpConnectionOptions {
                    properties: Some(
                        vec![
                            ("user-agent", get_user_agent(&self.application_id)),
                            ("version", get_package_version()),
                            ("platform", get_platform_info()),
                            ("product", get_package_name()),
                        ]
                        .into_iter()
                        .map(|(k, v)| (AmqpSymbol::from(k), AmqpValue::from(v)))
                        .collect(),
                    ),
                    custom_endpoint: self.custom_endpoint.clone(),
                    ..Default::default()
                }),
            )
            .await?;
        self.connections
            .get_or_try_init(|| async move {
                Ok::<_, azure_core::error::Error>(
                    connection.clone() as Arc<dyn AmqpConnectionApis + Send + Sync>
                )
            })
            .await?;
        Ok(())
    }

    async fn create_connection(&self) -> Result<Arc<dyn AmqpConnectionApis + Send + Sync>> {
        trace!("Creating connection for {}.", self.url);
        let connection = Arc::new(AmqpConnection::new());

        connection
            .open(
                self.connection_name.clone(),
                self.url.clone(),
                Some(AmqpConnectionOptions {
                    properties: Some(
                        vec![
                            ("user-agent", get_user_agent(&self.application_id)),
                            ("version", get_package_version()),
                            ("platform", get_platform_info()),
                            ("product", get_package_name()),
                        ]
                        .into_iter()
                        .map(|(k, v)| (AmqpSymbol::from(k), AmqpValue::from(v)))
                        .collect(),
                    ),
                    custom_endpoint: self.custom_endpoint.clone(),
                    ..Default::default()
                }),
            )
            .await?;
        Ok(connection as Arc<dyn AmqpConnectionApis + Send + Sync>)
    }

    pub(crate) async fn ensure_connection(&self) -> Result<()> {
        self.connections
            .get_or_try_init(|| self.create_connection())
            .await?;
        Ok(())
    }

    pub(crate) fn get_connection(&self) -> Result<Arc<dyn AmqpConnectionApis + Send + Sync>> {
        let connection = self
            .connections
            .get()
            .ok_or_else(|| EventHubsError::from(ErrorKind::MissingConnection))?;
        Ok(connection.clone())
    }

    pub(crate) fn get_connection_id(&self) -> &str {
        &self.connection_name
    }

    pub(crate) async fn close_connection(&self) -> Result<()> {
        let connection = self.get_connection()?;

        connection.close().await
    }

    pub(crate) async fn authorize_path(
        &self,
        connection: &Arc<dyn AmqpConnectionApis + Send + Sync>,
        path: &Url,
        credential: Arc<dyn azure_core::credentials::TokenCredential>,
    ) -> Result<AccessToken> {
        debug!("Authorizing path: {path}");
        let mut scopes = self.authorization_scopes.lock().await;

        if !scopes.contains_key(path) {
            // Create an ephemeral session to host the authentication.
            let session = AmqpSession::new();
            session.begin(connection, None).await?;

            let cbs = AmqpClaimsBasedSecurity::new(&session)?;
            cbs.attach().await?;

            debug!("Get Token.");
            let token = credential
                .get_token(&["https://eventhubs.azure.net/.default"])
                .await?;
            let expires_at = token.expires_on;
            cbs.authorize_path(
                path.to_string(),
                None,
                token.token.secret().to_string(),
                expires_at,
            )
            .await?;

            // insert returns some if it *fails* to insert, None if it succeeded.
            let present = scopes.insert(path.clone(), token);
            if present.is_some() {
                return Err(EventHubsError::from(ErrorKind::UnableToAddAuthenticationToken).into());
            }
            trace!("Token added.");
        }
        Ok(scopes
            .get(path)
            .ok_or_else(|| EventHubsError::from(ErrorKind::UnableToAddAuthenticationToken))?
            .clone())
    }
}

#[cfg(test)]
mod tests {
    // cspell: ignore rngs
    use super::*;
    use azure_core::credentials::{Secret, TokenCredential};
    use azure_core::error::ResultExt;
    use azure_core_amqp::{
        mocks::MockAmqpConnection,
        {AmqpConnectionOptions, AmqpSessionOptions},
    };
    use rand::rngs::mock;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use time::OffsetDateTime;

    // Mock implementation of TokenCredential for testing
    #[derive(Debug)]
    struct MockTokenCredential {
        token: String,
        expires_on: OffsetDateTime,
    }

    impl MockTokenCredential {
        fn new(token: &str, expires_on: OffsetDateTime) -> Self {
            Self {
                token: token.to_string(),
                expires_on,
            }
        }
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockTokenCredential {
        async fn get_token(&self, _scopes: &[&str]) -> azure_core::Result<AccessToken> {
            Ok(AccessToken {
                token: Secret::new(self.token.clone()),
                expires_on: self.expires_on,
            })
        }
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let url = Url::parse("amqps://example.com").unwrap();
        let application_id = Some("test_app".to_string());
        let custom_endpoint = Some(Url::parse("https://custom.endpoint").unwrap());

        let connection_manager = ConnectionManager::new(url, application_id, custom_endpoint);

        assert_eq!(connection_manager.get_connection_id(), "test_app");
    }

    #[tokio::test]
    async fn test_connection_manager_without_app_id() {
        let url = Url::parse("amqps://example.com").unwrap();
        let connection_manager = ConnectionManager::new(url, None, None);

        // Without app_id, it should generate a UUID for the connection name
        assert!(!connection_manager.get_connection_id().is_empty());
        // Verify it's a valid UUID
        assert!(Uuid::parse_str(connection_manager.get_connection_id()).is_ok());
    }

    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))] // Mock connections not supported in WASM
    async fn test_authorize_path() {
        // Setup
        let url = Url::parse("amqps://example.com").unwrap();
        let connection_manager = ConnectionManager::new(url.clone(), None, None);

        // Create a mock connection
        let connection = Arc::new(AmqpConnection::new());
        let connection_api: Arc<dyn AmqpConnectionApis + Send + Sync> = connection.clone();

        // Create a mock credential
        let expires_on = OffsetDateTime::now_utc() + Duration::from_secs(3600);
        let mock_credential = Arc::new(MockTokenCredential::new("mock_token", expires_on));

        // Create a path to authorize
        let path = Url::parse("amqps://example.com/path").unwrap();

        // This would fail in a real test because we can't open the connection
        // but we can test the error path
        let result = connection_manager
            .authorize_path(&connection_api, &path, mock_credential)
            .await;

        // The actual authorize_path will fail since we're not mocking the entire AMQP stack
        assert!(result.is_err());

        // Check that the error is related to AMQP connection/session rather than
        // authorization token insertion logic
        let err = result.unwrap_err();
        assert!(!format!("{:?}", err).contains("UnableToAddAuthenticationToken"));
    }

    #[test]
    fn test_get_connection_without_initialization() {
        let url = Url::parse("amqps://example.com").unwrap();
        let connection_manager = ConnectionManager::new(url, None, None);

        // Without calling ensure_connection first, get_connection should return an error
        let result = connection_manager.get_connection();
        assert!(result.is_err());

        // Verify it's the expected error kind
        let err = result.err();
        let error_str = format!("{:?}", err);
        assert!(error_str.contains("Connection is not yet open"));
    }

    #[tokio::test]
    async fn test_close_connection() {
        let url = Url::parse("amqps://example.com").unwrap();
        let connection_manager = ConnectionManager::new(url, None, None);

        // Create a mock connection
        let mock_connection = Arc::new(MockAmqpConnection::new());

        // Open the connection
        connection_manager
            .ensure_mock_connection(mock_connection.clone())
            .await
            .unwrap();

        assert!(connection_manager.connections.is_initialized());

        // Get the connection
        let connection = connection_manager.get_connection().unwrap();

        assert!(mock_connection.is_open());

        // Close the connection
        let result = connection_manager.close_connection().await;
        assert!(result.is_ok());

        // Verify that the connection is closed
        assert!(!mock_connection.is_open());
    }
}
