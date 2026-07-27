import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

export function summarizeRequiredJobResults(rawResults) {
  assert.equal(typeof rawResults, "object", "required CI results must be a JSON object");
  assert.notEqual(rawResults, null, "required CI results must not be null");
  assert.equal(Array.isArray(rawResults), false, "required CI results must be keyed by job id");

  const entries = Object.entries(rawResults);
  assert.ok(entries.length > 0, "required CI results must include at least one job");

  const failed = [];
  for (const [jobId, value] of entries) {
    assert.equal(typeof value, "object", `required CI result "${jobId}" must be an object`);
    assert.notEqual(value, null, `required CI result "${jobId}" must not be null`);

    const result = value.result;
    assert.equal(
      typeof result,
      "string",
      `required CI result "${jobId}" must expose a string result`,
    );
    if (result !== "success") {
      failed.push({ jobId, result });
    }
  }

  return {
    total: entries.length,
    failed,
  };
}

export function assertRequiredJobResults(rawJson) {
  const summary = summarizeRequiredJobResults(JSON.parse(rawJson));
  assert.deepEqual(
    summary.failed,
    [],
    `required CI jobs did not succeed: ${summary.failed
      .map(({ jobId, result }) => `${jobId}=${result}`)
      .join(", ")}`,
  );
  return summary;
}

const isEntrypoint =
  process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isEntrypoint) {
  const rawJson = process.env.OMENA_CI_REQUIRED_RESULTS;
  assert.ok(rawJson, "OMENA_CI_REQUIRED_RESULTS is required");
  const summary = assertRequiredJobResults(rawJson);
  console.log(`Required CI jobs passed: ${summary.total}/${summary.total}`);
}
