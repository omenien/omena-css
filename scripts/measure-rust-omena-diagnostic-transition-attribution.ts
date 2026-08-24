import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-custom-property-diagnostic-census.json");
const fromPin = "76574230a2e807f243f04e09944f6696d51541b8";
const toPin = "26450dbf146807be958b14547393dd57852dbc45";
const args = new Set(process.argv.slice(2).filter((argument) => argument !== "--"));

assert.deepEqual(
  [...args].filter((argument) => argument !== "--inject-location-substitution"),
  [],
  "unknown diagnostic transition attribution option",
);

const census = JSON.parse(readFileSync(censusPath, "utf8")) as DiagnosticCensus;
const locationOwner = "type-expanded-keyword-closure";
const replayOwner = "accepted-keyword-rejection-authority";
const declaredLocations = resolveLocations(census, locationOwner);
const replayedLocations = resolveLocations(census, replayOwner);

assert.deepEqual(
  locationKeys(replayedLocations),
  locationKeys(declaredLocations),
  "the replayed diagnostic transition must reference the measured location set",
);

for (const pin of [fromPin, toPin]) {
  ensureCommitAvailable(pin);
}

const scratch = mkdtempSync(path.join(tmpdir(), "omena-diagnostic-transition-"));
const worktree = path.join(scratch, "worktree");
const targetDirectory = path.join(repoRoot, "rust/target/diagnostic-transition-attribution");
const add = spawnSync("git", ["worktree", "add", "--detach", "--quiet", worktree, fromPin], {
  cwd: repoRoot,
  encoding: "utf8",
});
assert.equal(add.status, 0, `cannot create diagnostic measurement worktree: ${add.stderr}`);

try {
  installMeasurementProbe(worktree);
  const beforeLocations = measureInvalidPropertyValueLocations(worktree, targetDirectory);

  const checkout = spawnSync("git", ["checkout", "--detach", "--quiet", toPin], {
    cwd: worktree,
    encoding: "utf8",
  });
  assert.equal(checkout.status, 0, `cannot checkout the second diagnostic pin: ${checkout.stderr}`);
  installMeasurementProbe(worktree);
  const afterLocations = measureInvalidPropertyValueLocations(worktree, targetDirectory);

  const before = new Set(beforeLocations);
  const afterSet = new Set(afterLocations);
  const addedLocations = afterLocations.filter((location) => !before.has(location));
  const removedLocations = beforeLocations.filter((location) => !afterSet.has(location));

  assert.equal(beforeLocations.length, 1, "the earlier pin must measure one invalid value");
  assert.equal(afterLocations.length, 13, "the later pin must measure thirteen invalid values");
  assert.deepEqual(
    removedLocations,
    [],
    "the diagnostic transition unexpectedly removed a location",
  );
  assert.equal(addedLocations.length, 12, "the diagnostic transition must add twelve locations");

  let expectedLocations = locationKeys(declaredLocations);
  if (args.has("--inject-location-substitution")) {
    const correctLocation =
      "test/_fixtures/stylelint-plugin-smoke/src/ValueMissingModule.module.css\t4\t3";
    const staleLocation = "test/_fixtures/semantic-smoke/ValueSmoke.module.scss\t5\t3";
    const correctIndex = expectedLocations.indexOf(correctLocation);
    assert.ok(correctIndex >= 0, "the measured ValueMissingModule location is absent");
    expectedLocations[correctIndex] = staleLocation;
    expectedLocations = expectedLocations.toSorted(byteCompare);
  }
  assert.deepEqual(
    expectedLocations,
    addedLocations,
    "the declared per-input attribution diverged from the measured two-pin diagnostic diff",
  );

  const staleLocation = "test/_fixtures/semantic-smoke/ValueSmoke.module.scss\t5\t3";
  assert.ok(
    before.has(staleLocation),
    "the earlier-pin survivor must remain independently measured",
  );
  assert.ok(
    !addedLocations.includes(staleLocation),
    "the earlier-pin survivor cannot be attributed to the transition",
  );
  const substitutedLocations = [...locationKeys(declaredLocations)];
  const correctLocation =
    "test/_fixtures/stylelint-plugin-smoke/src/ValueMissingModule.module.css\t4\t3";
  const substitutedIndex = substitutedLocations.indexOf(correctLocation);
  assert.ok(substitutedIndex >= 0, "the measured ValueMissingModule location is absent");
  substitutedLocations[substitutedIndex] = staleLocation;
  const orderedSubstitutedLocations = substitutedLocations.toSorted(byteCompare);
  assert.throws(
    () =>
      assert.deepEqual(
        orderedSubstitutedLocations,
        addedLocations,
        "the declared per-input attribution diverged from the measured two-pin diagnostic diff",
      ),
    /declared per-input attribution diverged/u,
    "substituting the earlier-pin survivor must be RED",
  );

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "1",
      product: "omena-query.diagnostic-transition-attribution",
      fromPin,
      toPin,
      beforeInvalidPropertyValueCount: beforeLocations.length,
      afterInvalidPropertyValueCount: afterLocations.length,
      addedLocationCount: addedLocations.length,
      removedLocationCount: removedLocations.length,
      attribution: "measured-two-pin-diff:GREEN",
      locationSubstitutionMutation: "RED",
    })}\n`,
  );
} finally {
  spawnSync("git", ["worktree", "remove", "--force", worktree], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  rmSync(scratch, { recursive: true, force: true });
}

function ensureCommitAvailable(pin: string): void {
  const present = spawnSync("git", ["cat-file", "-e", `${pin}^{commit}`], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (present.status === 0) {
    return;
  }

  const fetch = spawnSync("git", ["fetch", "--no-tags", "--depth=1", "origin", pin], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(fetch.status, 0, `cannot fetch diagnostic measurement pin ${pin}: ${fetch.stderr}`);

  const fetched = spawnSync("git", ["cat-file", "-e", `${pin}^{commit}`], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(fetched.status, 0, `diagnostic measurement pin ${pin} is unavailable after fetch`);
}

function installMeasurementProbe(worktreeRoot: string): void {
  const probePath = path.join(
    worktreeRoot,
    "rust/crates/omena-query/tests/diagnostic_transition_attribution_probe.rs",
  );
  writeFileSync(
    probePath,
    `use omena_query::{
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_hover_candidates,
};
use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

#[test]
fn prints_invalid_property_value_locations() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root");
    let tracked = Command::new("git")
        .args(["ls-files", "--", "*.css", "*.scss", "*.less"])
        .current_dir(&repo_root)
        .output()
        .expect("git ls-files for tracked style diagnostics corpus");
    assert!(tracked.status.success());

    let paths = String::from_utf8(tracked.stdout).expect("tracked paths are UTF-8");
    let mut locations = BTreeSet::<String>::new();
    for relative_path in paths.lines() {
        if relative_path.split('/').any(|component| {
            matches!(
                component,
                "node_modules" | "target" | "dist" | "build" | "coverage" | ".next" | ".turbo"
            )
        }) {
            continue;
        }
        let source = fs::read_to_string(repo_root.join(relative_path))
            .unwrap_or_else(|error| panic!("cannot read {relative_path}: {error}"));
        let candidates = summarize_omena_query_style_hover_candidates(relative_path, &source)
            .unwrap_or_else(|| panic!("cannot derive hover candidates for {relative_path}"));
        let diagnostics = summarize_omena_query_style_diagnostics_for_file(
            format!("file://{relative_path}").as_str(),
            &source,
            candidates.candidates.as_slice(),
        );
        for diagnostic in diagnostics.diagnostics {
            if diagnostic.code == "invalidPropertyValue" {
                locations.insert(format!(
                    "{relative_path}\\t{}\\t{}",
                    diagnostic.range.start.line + 1,
                    diagnostic.range.start.character + 1,
                ));
            }
        }
    }

    println!(
        "diagnosticTransitionLocations={}",
        serde_json::to_string(&locations.into_iter().collect::<Vec<_>>())
            .expect("serialize diagnostic locations")
    );
}
`,
  );
}

function measureInvalidPropertyValueLocations(worktreeRoot: string, targetRoot: string): string[] {
  const result = spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "--test",
      "diagnostic_transition_attribution_probe",
      "--",
      "--nocapture",
    ],
    {
      cwd: worktreeRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        CARGO_TARGET_DIR: targetRoot,
      },
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  assert.equal(result.status, 0, `diagnostic measurement failed:\n${output.slice(-8_000)}`);
  const match = output.match(/diagnosticTransitionLocations=(\[[^\n]+\])/u);
  assert.ok(match, "the exact diagnostic-location measurement receipt is absent");
  const locations = JSON.parse(match[1]) as string[];
  assert.deepEqual(
    locations,
    locations.toSorted(byteCompare),
    "diagnostic locations are unordered",
  );
  assert.equal(new Set(locations).size, locations.length, "diagnostic locations are not unique");
  return locations;
}

function resolveLocations(censusValue: DiagnosticCensus, owner: string): DiagnosticLocation[] {
  const delta = censusValue.declaredDeltas.find((entry) => entry.owner === owner);
  assert.ok(delta, `${owner} diagnostic delta owner is absent`);
  assert.ok(
    (delta.locations === undefined) !== (delta.locationSetOwner === undefined),
    `${owner} must either own or reference one diagnostic location set`,
  );
  if (delta.locations !== undefined) return delta.locations;
  assert.notEqual(delta.locationSetOwner, owner, `${owner} cannot reference itself`);
  const referenced = censusValue.declaredDeltas.find(
    (entry) => entry.owner === delta.locationSetOwner,
  );
  assert.ok(referenced?.locations, `${owner} references an absent diagnostic location set`);
  return referenced.locations;
}

function locationKeys(locations: DiagnosticLocation[]): string[] {
  const keys = locations
    .map(({ path: sourcePath, line, character }) => `${sourcePath}\t${line}\t${character}`)
    .toSorted(byteCompare);
  assert.equal(new Set(keys).size, keys.length, "declared diagnostic locations are not unique");
  return keys;
}

function byteCompare(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

type DiagnosticLocation = {
  path: string;
  line: number;
  character: number;
};

type DiagnosticCensus = {
  declaredDeltas: Array<{
    owner: string;
    locations?: DiagnosticLocation[];
    locationSetOwner?: string;
  }>;
};
