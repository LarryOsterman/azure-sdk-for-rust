---
name: update-emitter
description: |-
    Update the TypeSpec emitter for Rust and optionally regenerate all clients. This skill handles regenerating clients, after regenerating. Provides comprehensive support for common use cases and edge cases.
    USE FOR: update-emitter, regenerating clients, after regenerating, update-emitter help, use update-emitter.
    DO NOT USE FOR: general coding questions unrelated to update-emitter, creating new projects from scratch.
---

Run `eng/scripts/Update-Emitter.ps1` to update `eng/emitter-package.json` to the latest `@azure-tools/typespec-rust` version and regenerate the lock file.

## Regenerating clients

After updating the emitter, service crates should be regenerated to pick up the new version.

Service owners should regenerate only their own service crates. To regenerate a single crate, run `tsp-client update` from within the crate directory:

```bash
cd sdk/{service-directory}/{crate-directory}
tsp-client update
```

To regenerate all crates under a service directory:

```bash
find sdk/{service-directory} -name tsp-location.yaml -execdir tsp-client update \;
```

## After regenerating

- Build and test each affected crate to verify correctness.
- Review any changes to public APIs and update each crate's `CHANGELOG.md` accordingly.
