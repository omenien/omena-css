# omena-transform-bundle

`omena-transform-bundle` turns parser facts into a transform bundle plan for
`@import` inlining, SCSS/Less module evaluation, CSS Modules hashing,
`composes` resolution, and `@value` resolution. It does not inline text yet; it
establishes the source-fact boundary that later mutation engines must consume.

## Bundle and source-map decisions

The current bundle/source-map baseline is the six-surface gate set reported by
`rust/omena-cli-bundle-origin-chain`:

```text
bundleSourceMapOriginChain+bundleAssetUrlRewrite+bundleCodeSplitEmission+bundleCodeSplitManifestEmission+bundleCodeSplitTreeShakeEmission+bundleCodeSplitSourceMapEmission
```

The following decisions keep the bundle pipeline in one product shape while
upstream source-map composition and split-policy work continue.

1. Crate ownership stays in place. `omena-transform-bundle` remains the bundle
   planning crate instead of splitting a new `omena-bundler` crate. Existing
   consumers already reach this planner through `omena-query-transform-runner`,
   `omena-query`, `omena-cli`, and `omena-umbrella`, and `omena-query` also uses
   the bundle entry points internally. Keeping the crate in place avoids another
   publishable crate and keeps the release train bounded.

2. Bundle source maps keep embedding `sourcesContent` by default. The emitted V3
   maps are the canonical provenance artifact, and always-embedded source
   content keeps CLI output debuggable without a second fetch path. A size-driven
   opt-out can be added later when there is a concrete consumer that needs it.
   Split manifests do not duplicate full composed original source lists; the
   sidecar `.map` files remain the authority for composed original sources. If a
   later split-policy lane needs manifest-level origin hints, it should add a
   compact field in the same schema change that adds chunk boundary metadata.

3. Omena provenance is recorded inside Source Map V3 output. The V3 map carries
   `x_omenaSchemaVersion`, `x_omenaProduct`, `x_omenaSegmentCount`, and
   `x_omenaPassIds`, so external JSON-only provenance is not the active product
   contract.

4. Bundle-time tree shaking uses the existing build path. The CLI sets closed
   style-world context and appends tree-shake transform passes for single-output
   and split-output builds; there is no separate graph-access subsystem for this
   feature.

5. Asset URL handling is resolve-only at this layer. The bundle planner and CLI
   rewrite resolved asset references while preserving data, fragment, external,
   and absolute references. Content hashing, CDN publication, and cache-key
   policy stay downstream of this crate.

6. Split policy evolves through explicit boundary metadata. Future entry/route
   split configuration and shared chunk detection should add new planner boundary
   kinds without changing the existing `entry`, `styleDependency`, and
   `assetDependency` meanings. The CLI split manifest must gain an explicit
   chunk kind or boundary field before emitting chunks that cannot be represented
   by the current `isEntry` boolean.

7. The Vite integration remains a thin adapter by design. It should pass through
   the CLI/query source-map result and avoid deeper HMR or virtual-module
   machinery until a concrete user-facing requirement justifies that surface.
