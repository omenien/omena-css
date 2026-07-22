import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it, vi } from "vitest";
import {
  classifyCrateRegistryState,
  deriveSemverBaselinePlan,
  resolveCratePublishMode,
  semverCheckableLibraryPath,
} from "../../../scripts/crate-registry-state";
import {
  renderCargoPublishWorkspaceConfig,
  selectPublishableWorkspaceCrates,
} from "../../../scripts/generate-cargo-publish-workspace-config";

describe("crate publish registry contract", () => {
  it("separates semver baselines from first-publish crate names", async () => {
    const fetchRegistry = vi.fn(async (input: string | URL | Request) => {
      const name = String(input).split("/").at(-1);
      if (name === "omena-new") {
        return new Response("not found", { status: 404 });
      }
      return Response.json({
        versions: [{ num: name === "omena-current" ? "0.3.0" : "0.2.0" }],
      });
    });

    const state = await classifyCrateRegistryState({
      crateNames: ["omena-existing", "omena-new", "omena-current"],
      workspaceVersion: "0.3.0",
      fetchRegistry,
    });

    expect(state).toEqual({
      workspaceVersion: "0.3.0",
      publishable: ["omena-existing", "omena-new", "omena-current"],
      registered: ["omena-existing", "omena-current"],
      unregistered: ["omena-new"],
      alreadyPublished: ["omena-current"],
      remaining: ["omena-existing", "omena-new"],
      latestPublishedVersions: {
        "omena-existing": "0.2.0",
        "omena-current": "0.3.0",
      },
    });
  });

  it("derives semver eligibility from the baseline library surface", async () => {
    const plan = await deriveSemverBaselinePlan({
      registryState: {
        workspaceVersion: "0.3.0",
        publishable: ["omena-lib", "omena-bin", "omena-macro", "omena-current"],
        registered: ["omena-lib", "omena-bin", "omena-macro", "omena-current"],
        unregistered: [],
        alreadyPublished: ["omena-current"],
        remaining: ["omena-lib", "omena-bin", "omena-macro"],
        latestPublishedVersions: {
          "omena-lib": "0.2.0",
          "omena-bin": "0.2.0",
          "omena-macro": "0.2.0",
          "omena-current": "0.3.0",
        },
      },
      baselineHasCheckableLibraryTarget: async (name) =>
        name !== "omena-bin" && name !== "omena-macro",
    });

    expect(plan).toEqual({
      eligible: ["omena-lib"],
      noCheckableLibraryBaseline: ["omena-bin", "omena-macro"],
      alreadyPublished: ["omena-current"],
    });
  });

  it("excludes proc-macro and non-Rust library baselines from semver checks", () => {
    expect(semverCheckableLibraryPath('[package]\nname = "plain"\n')).toBe("src/lib.rs");
    expect(
      semverCheckableLibraryPath(
        '[package]\nname = "custom"\n\n[lib]\npath = "src/api.rs"\ncrate-type = ["rlib", "cdylib"]\n',
      ),
    ).toBe("src/api.rs");
    expect(
      semverCheckableLibraryPath('[lib]\nproc-macro = true\npath = "src/lib.rs"\n'),
    ).toBeUndefined();
    expect(
      semverCheckableLibraryPath('[lib]\ncrate-type = ["cdylib", "staticlib"]\n'),
    ).toBeUndefined();
  });

  it("fails closed when registry status cannot be established", async () => {
    await expect(
      classifyCrateRegistryState({
        crateNames: ["omena-existing"],
        workspaceVersion: "0.3.0",
        fetchRegistry: async () => new Response("unavailable", { status: 503 }),
      }),
    ).rejects.toThrow("crates.io returned 503 for omena-existing");
  });

  it("requires bootstrap auth for an irreversible mixed first-publish train", () => {
    expect(
      resolveCratePublishMode({
        requestedMode: "auto",
        dryRun: false,
        unregisteredCount: 2,
      }),
    ).toEqual({ effectiveMode: "bootstrap", authenticationRequired: true });

    expect(() =>
      resolveCratePublishMode({
        requestedMode: "oidc",
        dryRun: false,
        unregisteredCount: 2,
      }),
    ).toThrow("require bootstrap mode");

    expect(
      resolveCratePublishMode({
        requestedMode: "oidc",
        dryRun: true,
        unregisteredCount: 2,
      }),
    ).toEqual({ effectiveMode: "trusted", authenticationRequired: false });
  });

  it("derives a local publish resolution config from publishable workspace members", () => {
    const crates = selectPublishableWorkspaceCrates({
      workspace_members: [
        "omena-a 0.3.0 (path+file:///repo/a)",
        "omena-b 0.3.0 (path+file:///repo/b)",
      ],
      packages: [
        {
          id: "omena-b 0.3.0 (path+file:///repo/b)",
          name: "omena-b",
          manifest_path: "/repo/b/Cargo.toml",
          publish: [],
        },
        {
          id: "omena-a 0.3.0 (path+file:///repo/a)",
          name: "omena-a",
          manifest_path: "/repo/a/Cargo.toml",
          publish: null,
        },
        {
          id: "external 1.0.0 (registry+https://example.invalid)",
          name: "external",
          manifest_path: "/registry/external/Cargo.toml",
          publish: null,
        },
      ],
    });

    expect(crates).toEqual([{ name: "omena-a", crateRoot: "/repo/a" }]);
    expect(renderCargoPublishWorkspaceConfig(crates)).toBe(
      "# Generated from cargo metadata for local resolution during workspace publish verification.\n" +
        '[patch.crates-io]\n"omena-a" = { path = "/repo/a" }\n',
    );
  });

  it("wires registry-aware semver and authentication before cargo publish", () => {
    const repoRoot = process.cwd();
    const workflow = readFileSync(
      path.join(repoRoot, ".github/workflows/_publish-crate-train.yml"),
      "utf8",
    );
    const action = readFileSync(
      path.join(repoRoot, ".github/actions/cargo-publish-workspace-oidc/action.yml"),
      "utf8",
    );

    const registryIndex = workflow.indexOf("name: Resolve crates.io registry state");
    const semverIndex = workflow.indexOf("name: cargo-semver-checks steady-state gate");
    const publishIndex = workflow.indexOf("name: Publish crate train");

    expect(registryIndex).toBeGreaterThan(-1);
    expect(semverIndex).toBeGreaterThan(registryIndex);
    expect(publishIndex).toBeGreaterThan(semverIndex);
    expect(workflow).toContain("steps.registry.outputs.effective_mode");
    expect(workflow).toContain(".semverEligible | index($crate) != null");
    expect(workflow).toContain("startsWith(github.ref, 'refs/tags/release-v') && 'auto'");
    expect(workflow).toContain(
      "(env.RESUME != 'true' && !startsWith(github.ref, 'refs/tags/release-v')) || steps.resume.outputs.remaining_count != '0'",
    );
    expect(action).toContain("inputs.mode == 'trusted' && inputs.dry-run != 'true'");
    expect(action).toContain('if [ "${{ inputs.dry-run }}" != "true" ]');
    expect(action).toContain("scripts/generate-cargo-publish-workspace-config.ts");
    expect(action).toContain('args+=(--config "${patch_config}")');
    expect(action.indexOf("generate-cargo-publish-workspace-config.ts")).toBeLessThan(
      action.indexOf('if [ "${{ inputs.dry-run }}" = "true" ]'),
    );
  });
});
