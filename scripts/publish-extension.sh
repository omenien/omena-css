#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Load local release credentials if present.
if [ -f .env ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
fi

CHANNEL="${RELEASE_CHANNEL:-stable}"
PUBLISH_MARKETPLACE="${PUBLISH_MARKETPLACE:-true}"
PUBLISH_OPENVSX="${PUBLISH_OPENVSX:-true}"

PACKAGE_ARGS=(--no-dependencies)
PUBLISH_ARGS=()

if [ "$CHANNEL" = "preview" ]; then
  PACKAGE_ARGS+=(--pre-release)
  PUBLISH_ARGS+=(--pre-release)
fi

./scripts/release.sh
pnpm check:release-m5-class-value-universe-matrix
pnpm check:release-m5-api-freeze-audit
pnpm check
if [ "${OMENA_RELEASE_NAPI_READY:-false}" != "true" ]; then
  pnpm omena-check run core/build/omena-napi
fi
pnpm test
pnpm build
node ./scripts/merge-engine-shadow-runner-artifacts.mjs
node ./scripts/restore-native-binary-permissions.mjs
pnpm check:packaged-engine-shadow-runner-matrix
pnpm check:packaged-tsgo-binary

rm -f ./*.vsix
node --import tsx ./scripts/package-extension-vsix.ts "${PACKAGE_ARGS[@]}"
VSIX_FILE="$(ls -1 ./*.vsix | head -n 1)"
pnpm check:packaged-selected-query-default
pnpm check:packaged-omena-lsp-server-type-fact-protocol

if [ "$PUBLISH_MARKETPLACE" = "true" ]; then
  pnpm exec vsce publish --packagePath "$VSIX_FILE" "${PUBLISH_ARGS[@]}"
fi

if [ "$PUBLISH_OPENVSX" = "true" ]; then
  pnpm exec ovsx publish "$VSIX_FILE" --skip-duplicate
fi
