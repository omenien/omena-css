# Security policy

## Supported versions

Security fixes target the latest published editor and Rust release lines. Older
versions may be affected even when a report reproduces only on a newer build.

| Surface             | Supported line                           |
| ------------------- | ---------------------------------------- |
| VS Code extension   | `5.3.x`                                  |
| Rust crates and CLI | `0.3.x`                                  |
| npm packages        | latest published version of each package |

## Report a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/omenien/omena-css/security/advisories/new).
Do not open a public issue for a suspected vulnerability.

Include the affected package or binary, version, platform, impact, smallest
reproduction, and any known workaround. Avoid attaching credentials, private
source code, or production data that is not needed to reproduce the issue.

Maintainers will acknowledge a complete report, confirm the affected surface,
and coordinate disclosure after a fix or documented mitigation is available.
Registry publication is not atomic across every Omena surface, so disclosure
timing may differ by channel.
