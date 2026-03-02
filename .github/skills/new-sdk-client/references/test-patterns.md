# Test Patterns Reference

Detailed API patterns for writing tests in Azure SDK for Rust crates. Always verify signatures against source code (the authoritative source), but these patterns reflect the current API.

## Mock Credential (for unit tests)

```rust
use azure_core::credentials::{AccessToken, Secret, TokenRequestOptions};

#[derive(Debug)]
struct MockCredential;

#[async_trait::async_trait]
impl azure_core::credentials::TokenCredential for MockCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        Ok(AccessToken {
            token: Secret::new("mock-token"),
            expires_on: time::OffsetDateTime::now_utc()
                + std::time::Duration::from_secs(3600),
        })
    }
}
```

### Key details for credentials

- `TokenCredential` requires the `async_trait` macro.
- `get_token` takes 3 parameters: `&self`, `&[&str]` (scopes), and `Option<TokenRequestOptions<'_>>`.
- `AccessToken.expires_on` is `time::OffsetDateTime` — **not** `std::time::SystemTime`.
- `AccessToken.token` is `Secret<String>`, constructed via `Secret::new("...")`.
- Add `async-trait` and `time` as dev-dependencies.

## Integration Test Setup (for `#[recorded::test]`)

```rust
use azure_core::Result;
use azure_core_test::{recorded, TestContext, VarOptions};

fn get_client(ctx: &TestContext) -> Result<MyClient> {
    let recording = ctx.recording();
    let endpoint = recording.var(
        "MY_SERVICE_ENDPOINT",
        Some(VarOptions {
            default_value: Some("https://default-endpoint.example.com".into()),
            ..Default::default()
        }),
    );
    let credential = recording.credential();
    let mut options = MyClientOptions::default();
    recording.instrument(&mut options.client_options);
    MyClient::new(&endpoint, credential, Some(options))
}

#[recorded::test]
async fn my_test(ctx: TestContext) -> Result<()> {
    let client = get_client(&ctx)?;
    // ... test logic ...
    Ok(())
}
```

### Key details for test integration

- `recording.var(name, options)` takes `Option<VarOptions>` — **not** `Option<&str>`. Use `VarOptions { default_value: Some("...".into()), ..Default::default() }`.
- `recording.credential()` returns `Arc<dyn TokenCredential>` — a mock in playback, a real credential in record/live mode.
- `recording.instrument(&mut options.client_options)` configures the HTTP pipeline for recording/playback.
- `DeveloperToolsCredential::new(None)?` already returns `Arc<Self>` — do **not** wrap in `Arc::new()` again.
- `RequestContent::from(data.to_string().into_bytes())` converts JSON to a request body.

## Required dev-dependencies

```toml
[dev-dependencies]
async-trait = { workspace = true }
azure_core_test = { workspace = true, features = ["tracing"] }
azure_identity.workspace = true
serde_json = { workspace = true }
time = { workspace = true }
tokio.workspace = true
tracing.workspace = true
```
