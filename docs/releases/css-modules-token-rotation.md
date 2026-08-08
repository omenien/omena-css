---
title: CSS Modules token rotation
description: Compatibility notice for an upcoming CSS Modules token identity rotation.
kind: explanation
status: preview
products: [cli, sdk, napi, wasm, lsp]
owner: release
sourceOfTruth: authored
---

# CSS Modules Token Rotation

The CSS Modules token producer now identifies each emitted class by its module
and raw class-name bytes. This rotation is planned as a coordinated extension
`5.4.0` and Rust crate-train `0.4.0` release. It does not use extension `6.0.0`,
which remains reserved for the independently reviewed linked-emission default
switch.

<!-- omena-css-module-token-rotation-contract:start -->

```json
{
  "schemaVersion": "1",
  "product": "omena.css-modules-token-rotation",
  "previousRotationIdentity": null,
  "rotationIdentity": "omena.css-modules.module-and-class-hash.v1",
  "coordinatedRelease": {
    "extension": "5.4.0",
    "rustCrateTrain": "0.4.0"
  },
  "breakClasses": [
    {
      "id": "persisted-emitted-token",
      "warningChannel": "none"
    },
    {
      "id": "global-to-scoped-dependency-class",
      "warningChannel": "none"
    },
    {
      "id": "handwritten-emitted-token-selector",
      "warningChannel": "none"
    },
    {
      "id": "stale-interface-manifest",
      "warningChannel": "build-time"
    },
    {
      "id": "mixed-token-identity-versions",
      "warningChannel": "none"
    },
    {
      "id": "collision-retained-declaration-removal",
      "warningChannel": "none"
    },
    {
      "id": "required-workspace-root",
      "warningChannel": "build-time"
    }
  ]
}
```

<!-- omena-css-module-token-rotation-contract:end -->

The previous token format did not publish a rotation identity, so
`previousRotationIdentity` is `null`. The new identity names the byte-producing
algorithm independently of either release version and is the value tools and
operators can compare during a coordinated upgrade.

The emitted token is not a contract; `classMap`, `namedExports`, and the
generated `.d.ts` are. Hand-writing an emitted token into markup, tests, or CSS
is unsupported. Consumers should read the generated interface rather than
persisting or constructing emitted names.

## Compatibility classes

The seven entries in the contract block are separate compatibility classes,
not seven names for one symptom:

- `persisted-emitted-token`: downstream snapshots, visual baselines, and E2E
  selectors that stored old emitted tokens change. Omena has no warning channel
  for an opaque token copied into an external artifact.
- `global-to-scoped-dependency-class`: dependency classes that previously
  leaked as unscoped names are now module-scoped. Markup that directly used a
  leaked name such as `class="solo1"` no longer selects that declaration.
- `handwritten-emitted-token-selector`: handwritten CSS and `:global`
  overrides that target an emitted token stop matching after the rotation.
- `stale-interface-manifest`: a checked-in `--interface-file` manifest can
  describe the old token bytes. Repository builds detect local drift, but they
  cannot update a consumer's separately stored copy.
- `mixed-token-identity-versions`: an old prebuilt package and a newly built
  application can each contain valid CSS while disagreeing about the token
  named by the generated map.
- `collision-retained-declaration-removal`: a declaration retained only because
  two modules previously produced the same token is now removed when normal
  reachability is applied. This is a second-order byte change alongside the
  token rotation.
- `required-workspace-root`: bundler-host resolve requests now require
  `workspaceRoot`. Older request payloads that omit it fail deserialization;
  callers must send the same stable workspace boundary used to derive module
  identity.

Caller-supplied per-module class maps are part of the selected module context.
Strict verification, when explicitly enabled, validates emitted bytes against
that selected context. The default `Descriptive` profile continues to report
the census without turning it into an admission failure; Strict coverage is
therefore opt-in rather than evidence about every default build.

The number of consumers in each class is not measured.

## Migration runbook

1. Upgrade the extension and every Rust, NAPI, WASM, CLI, and build adapter used
   by one build together. Do not combine old prebuilt CSS with a new token map.
2. Rebuild CSS Modules and regenerate `classMap`, `namedExports`, generated
   `.d.ts` files, and checked-in `--interface-file` manifests.
3. Replace copied token strings in markup, snapshot tests, visual baselines,
   and E2E selectors with lookups through the generated interface.
4. Audit handwritten CSS and `:global` rules for selectors copied from emitted
   output; rewrite them against a supported source-level contract.
5. Check dependency styles for class names that were used as globals. Import
   those names through the module interface instead.
6. Invalidate cached and prebuilt CSS assets so all artifacts in a deployment
   carry `omena.css-modules.module-and-class-hash.v1`.
7. Review byte snapshots for declarations that disappear after collision-free
   reachability, then run the application's visual and E2E suites.
8. Add `workspaceRoot` to every bundler-host resolve request. Use a stable,
   canonical workspace boundary and keep it identical across relocated builds;
   do not derive it from a temporary output directory.
