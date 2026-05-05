#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_dir="${repo_root}/rust/external-consumers/omena-checker-git-consumer"
dependency_line="$(grep '^omena-checker = ' "${fixture_dir}/Cargo.toml")"
default_repo_url="$(printf '%s\n' "${dependency_line}" | sed -E 's/.*git = "([^"]+)".*/\1/')"
default_repo_ref="$(printf '%s\n' "${dependency_line}" | sed -E 's/.*rev = "([^"]+)".*/\1/')"
repo_url="${OMENA_CHECKER_GIT_REPO:-${default_repo_url}}"
repo_ref="${OMENA_CHECKER_GIT_REF:-${default_repo_ref}}"
target_dir="${repo_root}/rust/target"

if [[ "${repo_url}" == "${default_repo_url}" && "${repo_ref}" == "${default_repo_ref}" ]]; then
  manifest_path="${fixture_dir}/Cargo.toml"
else
  temp_dir="$(mktemp -d)"
  trap 'rm -rf "${temp_dir}"' EXIT
  mkdir -p "${temp_dir}/src"
  cp -R "${fixture_dir}/src/." "${temp_dir}/src/"
  cat > "${temp_dir}/Cargo.toml" <<EOF
[workspace]
members = ["."]
resolver = "2"

[package]
name = "omena-checker-git-consumer-check"
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = false

[dependencies]
omena-abstract-value = "0.1.8"
omena-checker = { git = "${repo_url}", rev = "${repo_ref}" }
serde_json = "1.0"
EOF
  manifest_path="${temp_dir}/Cargo.toml"
fi

printf 'checking omena-checker repo via git dependency\n'
printf '  repo: %s\n' "${repo_url}"
printf '  ref:  %s\n' "${repo_ref}"
printf '  fixture: %s\n' "${manifest_path}"

CARGO_TARGET_DIR="${target_dir}" RUSTUP_TOOLCHAIN=stable cargo test --manifest-path "${manifest_path}" -p omena-checker-git-consumer-check
CARGO_TARGET_DIR="${target_dir}" RUSTUP_TOOLCHAIN=stable cargo check --manifest-path "${manifest_path}" -p omena-checker-git-consumer-check
CARGO_TARGET_DIR="${target_dir}" RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path "${manifest_path}" -p omena-checker-git-consumer-check --all-targets --all-features -- -D warnings
