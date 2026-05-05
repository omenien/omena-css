#!/usr/bin/env bash
set -euo pipefail

prefix="rust/crates/omena-checker"
branch="${1:-split/omena-checker}"

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage:
  ./scripts/prepare-omena-checker-subtree.sh [branch-name]

Default branch:
  split/omena-checker

This creates a local history-preserving subtree branch for:
  rust/crates/omena-checker

Equivalent raw command:
  git subtree split --prefix=rust/crates/omena-checker -b <branch-name>
EOF
  exit 0
fi

if git show-ref --verify --quiet "refs/heads/${branch}"; then
  printf 'refusing to overwrite existing local branch: %s\n' "${branch}" >&2
  exit 1
fi

git subtree split --prefix="${prefix}" -b "${branch}"

printf '\ncreated subtree branch: %s\n' "${branch}"
printf 'next step: inspect the branch and add standalone repo scaffolding before publishing it elsewhere.\n'
