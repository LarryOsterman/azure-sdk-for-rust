# Contributing

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)

  When you run `cargo build`, toolchain version [1.80](https://releases.rs/docs/1.80.0/) and necessary components will be installed automatically.

- (Recommended) If you use [Visual Studio Code], install recommended extensions to improve your development experience.

## Generated code

If you want to contribute to a file that is generated (the file is located in a `generated` subdirectory), the best approach is to open a PR on the TypeSpec specification since we cannot replace generated code that will be replaced when regenerated. Please visit the [Azure/azure-rest-api-specs repo](https://github.com/Azure/azure-rest-api-specs/) to view and make changes to Azure service API specifications.

## Building

To build any library in the Azure SDK for Rust navigate to the library's project folder and run `cargo build`.

## Testing

[TODO] Add instructions on how to run tests for a specific project.
[TODO] Add instructions for write new tests.

### Debugging with Visual Studio Code

[Visual Studio Code] with recommended extensions installed can be used to run and debug tests for a module or individual tests.

If you need to debug a test, you can use the LLDB extension and set environment variables as needed. For example, to debug recording a specific test,
your `.vscode/launch.json` file might look something like:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Record secret_roundtrip",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--test=secret_client",
          "--package=azure_security_keyvault_secrets",
          "secret_roundtrip"
        ],
        "filter": {
          "name": "secret_client",
          "kind": "test"
        },
        "env": {
        }
      },
      "cwd": "${workspaceFolder}",
      "env": {
        "AZURE_KEYVAULT_URL": "https://my-vault.vault.azure.net/",
        "PROXY_MANUAL_START": "true",
        "RUST_LOG": "trace"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Play back secret_roundtrip",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--test=secret_client",
          "--package=azure_security_keyvault_secrets",
          "secret_roundtrip"
        ],
        "filter": {
          "name": "secret_client",
          "kind": "test"
        },
        "env": {
          "AZURE_TEST_MODE": "playback"
        }
      },
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "trace"
      }
    }
  ]
}
```

You can also start the [Test Proxy] manually, in which can you add to the outer `env` above to `"PROXY_MANUAL_START": "true"`.

To enable tracing, you can add the `RUST_LOG` environment variable as shown above using the [same format supported by `env_logger`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging).
The targets are the crate names if you want to trace more or less for specific targets e.g., `RUST_LOG=info,azure_core=trace` to trace information messages by default but detailed traces for the `azure_core` crate.

## Code Review Process

Before a pull request will be considered by the Azure SDK team, the following requirements must be met:

- Prior to issuing the pull request:
  - All code must have completed any necessary legal sign-off for being publicly viewable (Patent review, JSR review, etc.)
  - The changes cannot break any existing functional/unit tests that are part of the central repository.
    - This includes all tests, even those not associated with the given feature area
  - Code submitted must have basic unit test coverage, and have all unit tests pass. Testing is the full responsibility of person submitting the code review.
    - Functional tests are encouraged, and provide teams with a way to mitigate regressions cause by other code contributions.
  - Code should be commented.
  - Code should be fully code reviewed.
  - Code should be able to merge without any conflicts into the dev branch being targeted.
  - Code should pass all relevant static checks and coding guidelines set forth by the specific repository.
  - All build warnings and code analysis warnings should be fixed prior to submission.
- As part of the pull request (aka, in the text box on GitHub as part of submitting the pull request):
  - Proof of completion of the code review and test passes requirements above.
  - Identity of QA responsible for feature testing (can be conducted post-merging of the pull request).
  - Short description of the payload of pull request.
- After the pull request is submitted:
  - When your PR is submitted someone on our team will be auto assigned the PR for review. No need to email us.

Once all of the above steps are met, the following process will be followed:

- A member of the Azure SDK team will review the pull request on GitHub.
- If the pull request meets the repository's requirements, the individual will approve the pull request, merging the code into the appropriate branch of the source repository.
- If the request does not meet any of the requirements, the pull request will not be merged, and the necessary fixes for acceptance will be communicated back to the person who opened the PR.

### Pull Request Etiquette and Best Practices

#### Reviewers

- If you disagree with the overall approach of the PR, comment on the general PR rather than individual lines of code.
- Leaving [suggested changes](https://docs.github.com/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/commenting-on-a-pull-request#adding-line-comments-to-a-pull-request) is welcomed, but please never commit them for a PR you did not create.
- When you are seeking to understand something rather than request corrections, it is suggested to use language such as "I'm curious ..." as a prefix to comments.
- For comments that are just optional suggestions or are explicitly non-blocking, prefix them with "nit: " or "non-blocking: ".
- Avoid marking a PR as "Request Changes" ![Request Change Icon](https://user-images.githubusercontent.com/1279263/151379844-b9babb22-b0fe-4b9c-b749-eb7488a38d84.png) unless you have serious concerns that should block the PR from merging.
- When to mark a PR as "Approved"
  - You feel confident that the code meets a high quality bar, has adequate test coverage, is ready to merge.
  - You have left comments that are uncontroversial and there is a shared understanding with the author that the comments can be addressed or resolved prior to being merged without significant discussion or significant change to the design or approach.
- When to leave comments without approval
  - You do not feel confident that your review alone is sufficient to call the PR ready to merge.
  - You have feedback that may require detailed discussion or may indicate a need to change the current design or approach in a non-trivial way.
- When to mark a PR as "Request Changes"
  - You have significant concerns that must be addressed before this PR should be merged such as unintentional breaking changes, security issues, or potential data loss.

#### Pull Request Authors

- If you add significant changes to a PR after it has been marked approved, please confirm with reviewers that they still approve before merging.
- Please ensure that you have obtained an approval from at least one of the code owners before merging.
- If a reviewer marks your PR as approved along with specific comments, it is expected that those comments will be addressed or resolved prior to merging.
  - One exception is when a comment clearly states that the feedback is optional or just a nit
  - When in doubt, reach out to the commenter to confirm that they have no concerns with you merging without addressing a comment.

## Samples

### Third-party dependencies

Third party libraries should only be included in samples when necessary to demonstrate usage of an Azure SDK package; they should not be suggested or endorsed as alternatives to the Azure SDK.

When code samples take dependencies, readers should be able to use the material without significant license burden or research on terms. This goal requires restricting dependencies to certain types of open source or commercial licenses.

Samples may take the following categories of dependencies:

- **Open-source** : Open source offerings that use an [Open Source Initiative (OSI) approved license](https://opensource.org/licenses). Any component whose license isn't OSI-approved is considered a commercial offering. Prefer OSS projects that are members of any of the [OSS foundations that Microsoft is part of](https://opensource.microsoft.com/ecosystem/). Prefer permissive licenses for libraries, like [MIT](https://opensource.org/licenses/MIT) and [Apache 2](https://opensource.org/licenses/Apache-2.0). Copy-left licenses like [GPL](https://opensource.org/licenses/gpl-license) are acceptable for tools, and OSs. [Kubernetes](https://github.com/kubernetes/kubernetes), [Linux](https://github.com/torvalds/linux), and [Newtonsoft.Json](https://github.com/JamesNK/Newtonsoft.Json) are examples of this license type. Links to open source components should be to where the source is hosted, including any applicable license, such as a GitHub repository (or similar).

- **Commercial**: Commercial offerings that enable readers to learn from our content without unnecessary extra costs. Typically, the offering has some form of a community edition, or a free trial sufficient for its use in content. A commercial license may be a form of dual-license, or tiered license. Links to commercial components should be to the commercial site for the software, even if the source software is hosted publicly on GitHub (or similar).

- **Dual licensed**: Commercial offerings that enable readers to choose either license based on their needs. For example, if the offering has an OSS and commercial license, readers can  choose between them. [MySql](https://github.com/mysql/mysql-server) is an example of this license type.

- **Tiered licensed**: Offerings that enable readers to use the license tier that corresponds to their characteristics. For example, tiers may be available for students, hobbyists, or companies with defined revenue  thresholds. For offerings with tiered licenses, strive to limit our use in tutorials to the features available in the lowest tier. This policy enables the widest audience for the article. [Docker](https://www.docker.com/), [IdentityServer](https://duendesoftware.com/products/identityserver), [ImageSharp](https://sixlabors.com/products/imagesharp/), and [Visual Studio](https://visualstudio.com) are examples of this license type.

In general, we prefer taking dependencies on licensed components in the order of the listed categories. In cases where the category may not be well known, we'll document the category so that readers understand the choice that they're making by using that dependency.

[Test Proxy]: https://github.com/Azure/azure-sdk-tools/blob/main/tools/test-proxy/Azure.Sdk.Tools.TestProxy/README.md
[Visual Studio Code]: https://code.visualstudio.com
