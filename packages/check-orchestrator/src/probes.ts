export type CiProbeProfileId =
  | "orchestrator"
  | "rust-cli"
  | "cross-platform-cli"
  | "rust-workspace"
  | "closure-diff"
  | "linux-benchmark"
  | "verify";

export interface CiProbeProfile {
  readonly id: CiProbeProfileId;
  readonly target: string;
  readonly description: string;
  readonly platforms: readonly NodeJS.Platform[];
}

const ALL_RUNNER_PLATFORMS = ["linux", "darwin", "win32"] as const;

export const CI_PROBE_PROFILES: readonly CiProbeProfile[] = [
  {
    id: "orchestrator",
    target: "tooling/ci-probe/orchestrator",
    description: "Check manifest, CLI, inventory, and core TypeScript hygiene.",
    platforms: ALL_RUNNER_PLATFORMS,
  },
  {
    id: "rust-cli",
    target: "rust/ci-probe/omena-cli",
    description: "Run the Rust CLI product tests and CLI-facing contract gates.",
    platforms: ALL_RUNNER_PLATFORMS,
  },
  {
    id: "cross-platform-cli",
    target: "rust/ci-probe/omena-cli",
    description: "Run the Rust CLI probe on Linux, macOS, and Windows runners.",
    platforms: ALL_RUNNER_PLATFORMS,
  },
  {
    id: "rust-workspace",
    target: "rust/ci-probe/workspace",
    description: "Run workspace formatting, checking, and strict clippy.",
    platforms: ALL_RUNNER_PLATFORMS,
  },
  {
    id: "closure-diff",
    target: "rust/ci-probe/closure-diff",
    description: "Run the differential-test shard of the closure suite.",
    platforms: ["linux"],
  },
  {
    id: "linux-benchmark",
    target: "rust/ci-probe/linux-benchmark",
    description: "Run the Linux instruction-count and performance contract lane.",
    platforms: ["linux"],
  },
  {
    id: "verify",
    target: "tooling/ci-probe/verify",
    description: "Run the main TypeScript build and test verification lane.",
    platforms: ["linux"],
  },
];

export function resolveCiProbeProfile(id: string): CiProbeProfile | null {
  return CI_PROBE_PROFILES.find((profile) => profile.id === id) ?? null;
}
