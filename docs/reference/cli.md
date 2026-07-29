---
title: CLI reference
description: The complete generated Omena command surface and dispatch ownership.
kind: reference
status: stable
products: [cli]
owner: cli
sourceOfTruth: generated
---

<!-- Generated from product code. Do not edit by hand. -->

# CLI reference

## Product verbs

| Command         | Status              | Dispatch owner    |
| --------------- | ------------------- | ----------------- |
| `omena check`   | Compatibility alias | `facts_file`      |
| `omena lint`    | Product command     | `lint_workspace`  |
| `omena fmt`     | Product command     | `format_sources`  |
| `omena minify`  | Product command     | `minify_source`   |
| `omena bundle`  | Product command     | `bundle_command`  |
| `omena modules` | Product command     | `modules_command` |
| `omena sass`    | Product command     | `sass_command`    |
| `omena intel`   | Product command     | `intel_workspace` |
| `omena migrate` | Product command     | `migrate_command` |
| `omena verify`  | Product command     | `verify_command`  |
| `omena ci`      | Product command     | `ci_command`      |
| `omena explain` | Product command     | `explain_command` |

## Complete command surface

| Command                               | Role                | Availability             | Purpose                                                                            |
| ------------------------------------- | ------------------- | ------------------------ | ---------------------------------------------------------------------------------- |
| `omena check`                         | Compatibility alias | Default build            | Compatibility route through `facts_file`.                                          |
| `omena facts`                         | Specialized command | Default build            | Parse a CSS-family source and report parser-owned facts.                           |
| `omena lint`                          | Product command     | Default build            | Run semantic and compatibility lint rules.                                         |
| `omena fmt`                           | Product command     | Default build            | Format CSS-family sources through the typed CST formatter contract.                |
| `omena minify`                        | Product command     | Default build            | Minify a stylesheet with an explicit semantic profile and backend.                 |
| `omena bundle`                        | Product command     | Default build            | Bundle a source entry and emit CSS plus optional evidence.                         |
| `omena modules`                       | Product command     | Default build            | Emit or verify typed CSS Modules interfaces.                                       |
| `omena sass`                          | Product command     | Default build            | Inspect Sass module graphs and compatibility diagnostics.                          |
| `omena intel`                         | Product command     | Default build            | Query workspace style-intelligence providers.                                      |
| `omena migrate`                       | Product command     | Default build            | Plan a named source migration without applying unsafe edits.                       |
| `omena verify`                        | Product command     | Default build            | Verify user-workspace product contracts and evidence.                              |
| `omena ci`                            | Product command     | Default build            | Run the configured CI product workflow.                                            |
| `omena sdk`                           | Specialized command | Default build            | Execute a generated SDK workflow request against an ephemeral workspace runtime.   |
| `omena explain`                       | Product command     | Default build            | Explain a diagnostic, transform decision, or retained artifact.                    |
| `omena build`                         | Specialized command | Default build            | Run the conservative transform pipeline.                                           |
| `omena passes`                        | Specialized command | Default build            | List transform pass ids accepted by `omena build --pass`.                          |
| `omena compress`                      | Specialized command | Cargo feature `mdl`      | Estimate an MDL minimum-description summary for a style source.                    |
| `omena context`                       | Specialized command | Default build            | Derive transform context from EngineInputV2 semantic reachability.                 |
| `omena expression-flow`               | Specialized command | Default build            | Analyze cross-language class-value flow from EngineInputV2.                        |
| `omena selector-projection`           | Specialized command | Default build            | Project expression-domain flow values to target style selectors.                   |
| `omena cascade`                       | Specialized command | Default build            | Read cascade and custom-property LFP information at a source position.             |
| `omena context-index`                 | Specialized command | Default build            | Read @layer, @container, and @scope context indexes.                               |
| `omena style-diagnostics`             | Specialized command | Default build            | Read query-owned style diagnostics for a CSS-family file.                          |
| `omena style-hover-candidates`        | Specialized command | Default build            | Read query-owned style hover candidates for a CSS-family file.                     |
| `omena style-completion`              | Specialized command | Default build            | Read query-owned style completions at a source position.                           |
| `omena source-diagnostics`            | Specialized command | Default build            | Read query-owned source diagnostics from precomputed missing-selector candidates.  |
| `omena dynamic-classname-diagnostics` | Specialized command | Default build            | Read query-owned dynamic className M-tier diagnostics from an input JSON contract. |
| `omena perceptual-check`              | Specialized command | Default build            | Emit downstream perceptual-check JSON from Omena style facts.                      |
| `omena lock`                          | Specialized command | Default build            | Verify local Omena lockfile integrity.                                             |
| `omena sif`                           | Specialized command | Default build            | Generate local Sass Interface File artifacts.                                      |
| `omena provenance`                    | Specialized command | Default build            | Inspect deferred/advisory SIF provenance metadata without network access.          |
| `omena report`                        | Specialized command | Default build            | Report soundiness and diagnostic-noise visibility for a workspace slice.           |
| `omena audit`                         | Specialized command | Cargo feature `zk-audit` | Run feature-gated audit surfaces.                                                  |
