---
name: new-sdk-client
description: Create a new Azure SDK for Rust client library crate from a TypeSpec specification. Use when asked to create, scaffold, or generate a new Azure service client SDK. Covers workspace registration, TypeSpec code generation, hand-written wrappers, Cargo.toml setup, README, CHANGELOG, unit tests, integration tests, examples, test infrastructure (Bicep), Azure resource provisioning, and CI validation.
metadata:
    author: azure-sdk
    version: "1.0"
---

# Creating a New Azure SDK Client Library

Follow every step below in order. Do not skip steps or make assumptions — verify at each stage.

## Prerequisites

1. **Check Azure access** — run `az account show` to verify Azure credentials are available. Do not assume Azure access is unavailable — always check. If the default subscription is not appropriate for test resource provisioning, identify the correct subscription (e.g., "Azure SDK Developer Playground") and set it with `az account set --subscription <id>`.

2. **Find the TypeSpec spec** — look for a `main.tsp` under `specification/<service>/` in [azure-rest-api-specs](https://github.com/Azure/azure-rest-api-specs). Check the `tspconfig.yaml` for `@azure-tools/typespec-rust` emitter configuration. If no Rust emitter is configured, stop and inform the user.

## Step 1: Register the crate in the workspace

Add the new crate path (e.g., `"sdk/<service>/<crate-name>"`) to the `members` list in the root `Cargo.toml`. This **must** be done before code generation so that `cargo fmt` succeeds during the generation step.

## Step 2: Generate client code from TypeSpec

Use the `azsdk_package_generate_code` MCP tool (or `tsp-client update` from the crate directory) to generate models, clients, and operations. The tool automatically creates the crate directory structure (`src/`, `src/generated/`, `Cargo.toml`, `tsp-location.yaml`). **Do not manually create these directories or files.**

Example MCP tool parameters:

```text
azsdk_package_generate_code(
  localSdkRepoPath: "/absolute/path/to/azure-sdk-for-rust",
  tspConfigPath: "https://github.com/Azure/azure-rest-api-specs/blob/<commit-sha>/specification/<service>/<TypeSpecDir>/tspconfig.yaml",
  tspLocationPath: "",
  emitterOptions: ""
)
```

- The `tspConfigPath` must use a **commit SHA** (not a branch name) for reproducibility. To find the latest commit SHA, query the repository's recent commits.
- Verify the build succeeds with `cargo build -p <crate-name>` after generation.

## Step 3: Add hand-written wrappers

Create `src/clients.rs` for re-exporting generated client types and adding any custom client constructors, authentication setup, or convenience methods on top of the generated code in `src/generated/`.

Update `src/lib.rs` to follow this pattern:

```rust
// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod clients;
mod generated;

pub use clients::{MyServiceClient, MyServiceClientOptions};
pub use generated::models;
```

## Step 4: Update Cargo.toml

Update the generated `Cargo.toml` with:

- Descriptive `description`, `readme`, `homepage`, `documentation`, `keywords`, `categories` fields.
- `serde` and `serde_json` in `[dependencies]` if needed.
- Dev-dependencies:

```toml
[dev-dependencies]
async-trait = { workspace = true }
azure_core_test = { workspace = true, features = ["tracing"] }
azure_identity.workspace = true
serde_json = { workspace = true }
time = { workspace = true }
tokio.workspace = true
tracing.workspace = true

[lints]
workspace = true
```

## Step 5: Write README.md

Follow the structure of `sdk/keyvault/azure_security_keyvault_secrets/README.md`:

- Brief service description with links to product docs
- Getting started (install, prerequisites, authenticate, create client)
- Key concepts
- Examples (one per major operation)
- Troubleshooting
- Contributing

Use ` ```rust no_run ` for examples with placeholder values.

## Step 6: Write CHANGELOG.md

```markdown
# Release History

## 0.1.0 (Unreleased)

### Features Added

- Initial public release.
- Built on <service> service version <api-version>.
```

## Step 7: Write unit tests

Add a `#[cfg(test)] mod tests` in `src/lib.rs` covering:

- Client creation (success, with options, invalid URL, non-http endpoint)
- Endpoint accessibility
- Default option values
- Parameter validation (empty required parameters should error)
- Upload/method options defaults

Use this mock credential pattern (see [references/test-patterns.md](references/test-patterns.md) for details):

```rust
use azure_core::credentials::{AccessToken, Secret, TokenRequestOptions};

#[derive(Debug)]
struct MockCredential;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
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

## Step 8: Write integration tests

Create `tests/<client_name>.rs` using `#[recorded::test]` (see [references/test-patterns.md](references/test-patterns.md) for the full setup pattern):

```rust
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
```

## Step 9: Write an example

Create `examples/<operation>.rs` demonstrating the primary use case. Use `DeveloperToolsCredential` for authentication and read configuration from environment variables.

## Step 10: Create test infrastructure

Create `sdk/<service>/test-resources.bicep` that provisions all Azure resources needed for integration tests. Reference existing Bicep files (e.g., `sdk/keyvault/test-resources.bicep`) and equivalent files in other Azure SDK language repos (Python, JS) for the same service.

The Bicep file must:

- Accept standard parameters: `baseName`, `testApplicationOid`, `location`.
- Create all required Azure resources.
- Assign the appropriate RBAC role to `testApplicationOid`.
- RBAC role assignment names must be deterministic GUIDs. Include `baseName` in the `guid()` seed to avoid `RoleAssignmentExists` errors when redeploying to a shared resource group.
- Output environment variables that map to `recording.var()` calls in integration tests.

Also create `sdk/<service>/assets.json`:

```json
{
    "AssetsRepo": "Azure/azure-sdk-assets",
    "AssetsRepoPrefixPath": "rust",
    "TagPrefix": "rust/<service-name>",
    "Tag": ""
}
```

The `Tag` field starts empty and is populated after recordings are pushed.

## Step 11: Provision resources and record tests

```powershell
# Provision test resources (writes a .env file to sdk/<service>/.env automatically)
eng/common/TestResources/New-TestResources.ps1 -ServiceDirectory <service> -Location <region>
```

The provisioning script writes a `.env` file (e.g., `sdk/<service>/.env`) containing the Bicep outputs as environment variables. The test framework reads this file automatically — **do not manually set environment variables**. Just run:

```powershell
# Record integration tests against live Azure (reads .env automatically)
$env:AZURE_TEST_MODE = "record"
cargo test -p <crate-name> --all-features

# Verify tests pass in playback mode (no env vars needed)
$env:AZURE_TEST_MODE = $null
cargo test -p <crate-name> --all-features
```

**Provisioning tips:**

- Use `-Location` to match any existing resource group location (check the error message if deployment fails with a location conflict).
- Use `-BaseName` to control resource naming and avoid collisions with previous deployments.
- Use `-Force` to skip confirmation prompts.

## Step 12: Validate

Run all checks to ensure the crate passes CI:

```powershell
cargo build -p <crate-name>
cargo clippy -p <crate-name> --all-features --all-targets
cargo fmt -p <crate-name> -- --check
cargo test -p <crate-name> --all-features
```

All unit tests and integration tests must pass before the task is considered complete.
