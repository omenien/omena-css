#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dest="${1:-"${repo_root}/../omena-css"}"

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage:
  ./scripts/prepare-omena-css-subtree.sh [destination]

Default destination:
  ../omena-css

This stages the multi-crate omena-css standalone workspace. Unlike the
single-crate subtree helpers, omena-css is assembled from the publish-target
crates into one workspace:
  omena-interner, omena-syntax, omena-parser, omena-incremental, omena-cascade,
  omena-transform-cst, omena-transform-passes, omena-transform-bundle,
  omena-transform-target, omena-transform-print, omena-transform-egg.

The generated workspace is verified with cargo fmt, test, and clippy.
EOF
  exit 0
fi

node "${repo_root}/scripts/prepare-omena-css-workspace.mjs" \
  --dest "${dest}" \
  --force \
  --preserve-git \
  --verify
